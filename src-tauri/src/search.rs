use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{cmp::Reverse, env, thread};

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchReply {
    pub results: Vec<SearchResult>,
    pub warning: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AppEntry {
    name: String,
    target: String,
    kind: AppKind,
}

#[derive(Clone, Debug)]
enum AppKind {
    Shortcut,
    Store(String),
}

#[derive(Clone, Default)]
pub struct SearchState {
    apps: Arc<RwLock<Vec<AppEntry>>>,
}

impl SearchState {
    pub fn new() -> Self {
        let state = Self::default();
        let apps = Arc::clone(&state.apps);
        if let Err(error) = thread::Builder::new()
            .name("spotlight-app-index".to_owned())
            .spawn(move || {
                let shortcuts = discover_shortcut_apps();
                if let Err(error) = merge_apps(&apps, shortcuts) {
                    log::error!("Unable to cache Start Menu applications: {error}");
                }
                let store_apps = discover_store_apps();
                if let Err(error) = merge_apps(&apps, store_apps) {
                    log::error!("Unable to cache Store applications: {error}");
                }
            })
        {
            log::error!("Unable to start application indexing: {error}");
        }
        state
    }

    pub fn search(&self, query: &str) -> SearchReply {
        let apps = match self.apps.read() {
            Ok(apps) => apps.clone(),
            Err(error) => {
                log::error!("Unable to read application index: {error}");
                Vec::new()
            }
        };
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut utf32_buffer = Vec::new();
        let mut results: Vec<(i64, SearchResult)> = apps
            .iter()
            .filter_map(|app| {
                let score = pattern
                    .score(Utf32Str::new(&app.name, &mut utf32_buffer), &mut matcher)?
                    as i64;
                let (kind, subtitle, id) = match &app.kind {
                    AppKind::Shortcut => (
                        "Application",
                        app.target.clone(),
                        format!("app:{}", app.target),
                    ),
                    AppKind::Store(app_id) => (
                        "Application",
                        "Microsoft Store app".to_owned(),
                        format!("store:{app_id}"),
                    ),
                };
                Some((
                    score,
                    SearchResult {
                        id,
                        name: app.name.clone(),
                        subtitle,
                        kind: kind.to_owned(),
                    },
                ))
            })
            .collect();

        #[cfg(windows)]
        let warning = match super::windows_search::search(query, 64) {
            Ok(files) => {
                results.extend(files.into_iter().map(|result| (result.0, result.1)));
                None
            }
            Err(error) => {
                log::warn!("Windows Search provider could not answer: {error}");
                Some(format!(
                    "Windows Search is unavailable, so indexed files and folders could not be searched: {error}"
                ))
            }
        };
        #[cfg(not(windows))]
        let warning = Some("Indexed file search is only available on Windows.".to_owned());

        results.sort_by_key(|result| Reverse(result.0));
        let mut applications = 0;
        let mut indexed_items = 0;
        let mut selected = Vec::new();

        for (_, result) in &results {
            let is_application = result.kind == "Application";
            if is_application && applications == 5 || !is_application && indexed_items == 5 {
                continue;
            }

            if is_application {
                applications += 1;
            } else {
                indexed_items += 1;
            }
            selected.push(result.clone());
            if selected.len() == 9 {
                break;
            }
        }

        if selected.len() < 9 {
            for (_, result) in results {
                if selected.iter().any(|selected| selected.id == result.id) {
                    continue;
                }
                selected.push(result);
                if selected.len() == 9 {
                    break;
                }
            }
        }

        SearchReply {
            results: selected,
            warning,
        }
    }

    pub fn launch(&self, id: &str) -> Result<(), String> {
        if let Some(app_id) = id.strip_prefix("store:") {
            return launch_store_app(app_id);
        }
        let Some(path) = id
            .strip_prefix("app:")
            .or_else(|| id.strip_prefix("file:"))
            .or_else(|| id.strip_prefix("folder:"))
        else {
            return Err("The search result has an unsupported launch target.".to_owned());
        };
        launch_path(path)
    }
}

#[cfg(test)]
fn fuzzy_score(query: &str, candidate: &str) -> Option<i64> {
    if query.trim().is_empty() || candidate.is_empty() {
        return None;
    }

    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut buffer = Vec::new();
    pattern
        .score(Utf32Str::new(candidate, &mut buffer), &mut matcher)
        .map(i64::from)
}

fn merge_apps(cache: &RwLock<Vec<AppEntry>>, new_apps: Vec<AppEntry>) -> Result<(), String> {
    let mut apps = cache
        .write()
        .map_err(|error| format!("application cache lock was poisoned: {error}"))?;
    let mut names: HashSet<String> = apps.iter().map(|app| app.name.to_lowercase()).collect();
    apps.extend(
        new_apps
            .into_iter()
            .filter(|app| names.insert(app.name.to_lowercase())),
    );
    log::info!("Cached {} application entries", apps.len());
    Ok(())
}

fn discover_shortcut_apps() -> Vec<AppEntry> {
    let mut entries = Vec::new();
    let mut names = HashSet::new();

    for root in start_menu_roots() {
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file()
                || !entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
            {
                continue;
            }

            let Some(name) = entry.path().file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let name = name.trim();
            if name.is_empty()
                || ["uninstall", "readme", "help"]
                    .iter()
                    .any(|ignored| name.to_lowercase().starts_with(ignored))
            {
                continue;
            }

            if names.insert(name.to_lowercase()) {
                entries.push(AppEntry {
                    name: name.to_owned(),
                    target: entry.path().to_string_lossy().into_owned(),
                    kind: AppKind::Shortcut,
                });
            }
        }
    }

    log::info!(
        "Discovered {} Start Menu application shortcuts",
        entries.len()
    );
    entries
}

fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for (variable, suffix) in [
        ("PROGRAMDATA", "Microsoft\\Windows\\Start Menu\\Programs"),
        ("APPDATA", "Microsoft\\Windows\\Start Menu\\Programs"),
    ] {
        if let Some(base) = env::var_os(variable) {
            roots.push(PathBuf::from(base).join(suffix));
        }
    }
    roots
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StoreApp {
    name: String,
    app_id: String,
}

fn discover_store_apps() -> Vec<AppEntry> {
    let output = match powershell_command() {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            log::warn!(
                "Get-StartApps returned exit code {:?}",
                output.status.code()
            );
            return Vec::new();
        }
        Err(error) => {
            log::warn!("Unable to enumerate Store apps: {error}");
            return Vec::new();
        }
    };
    let decoded: Result<Vec<StoreApp>, _> = serde_json::from_slice(&output.stdout);
    let items = match decoded {
        Ok(items) => items,
        Err(_) => match serde_json::from_slice::<StoreApp>(&output.stdout) {
            Ok(item) => vec![item],
            Err(error) => {
                log::warn!("Unable to decode Store app list: {error}");
                return Vec::new();
            }
        },
    };
    items
        .into_iter()
        .filter(|item| !item.name.trim().is_empty() && !item.app_id.trim().is_empty())
        .map(|item| AppEntry {
            name: item.name,
            target: item.app_id.clone(),
            kind: AppKind::Store(item.app_id),
        })
        .collect()
}

#[cfg(windows)]
fn powershell_command() -> std::io::Result<std::process::Output> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-StartApps | Select-Object Name,AppID | ConvertTo-Json -Compress",
        ])
        .creation_flags(0x08000000)
        .output()
}

#[cfg(not(windows))]
fn powershell_command() -> std::io::Result<std::process::Output> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Store app enumeration is only supported on Windows",
    ))
}

#[cfg(windows)]
fn launch_path(path: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let operation: Vec<u16> = "open\0".encode_utf16().collect();
    let target: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(Some(0))
        .collect();
    // SAFETY: both strings are NUL-terminated and live through the shell call.
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(format!("Windows could not open search result: {path}"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn launch_path(_path: &str) -> Result<(), String> {
    Err("Launching results is only supported on Windows.".to_owned())
}

#[cfg(windows)]
fn launch_store_app(app_id: &str) -> Result<(), String> {
    let target = format!("shell:AppsFolder\\{app_id}");
    launch_path(&target)
}

#[cfg(not(windows))]
fn launch_store_app(_app_id: &str) -> Result<(), String> {
    Err("Launching Store apps is only supported on Windows.".to_owned())
}

#[cfg(test)]
mod tests {
    use super::fuzzy_score;

    #[test]
    fn fuzzy_score_prefers_prefix_matches() {
        let prefix = fuzzy_score("chr", "Chrome").expect("prefix should match");
        let scattered =
            fuzzy_score("chr", "Census Historical Records").expect("subsequence matches");
        assert!(prefix > scattered);
    }

    #[test]
    fn fuzzy_score_matches_acronyms_by_subsequence() {
        assert!(fuzzy_score("vsc", "Visual Studio Code").is_some());
        assert!(fuzzy_score("xyz", "Visual Studio Code").is_none());
    }

    #[test]
    fn fuzzy_score_matches_query_words_independently() {
        assert!(fuzzy_score("visual code", "Visual Studio Code").is_some());
        assert!(fuzzy_score("studio visual", "Visual Studio Code").is_some());
        assert!(fuzzy_score("visual xyz", "Visual Studio Code").is_none());
    }
}
