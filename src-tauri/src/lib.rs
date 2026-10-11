use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod search;
#[cfg(windows)]
mod windows_search;

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    let hide_result = window.hide().map_err(|error| {
        log::error!("Failed to hide the Spotlight Search window: {error}");
        error.to_string()
    });
    unregister_escape_shortcut(window.app_handle().clone());
    hide_result
}

fn unregister_escape_shortcut(app: AppHandle) {
    let shortcut = Shortcut::new(None, Code::Escape);
    let app_for_worker = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = app_for_worker.global_shortcut().unregister(shortcut) {
            log::debug!("Unable to unregister the temporary Escape shortcut: {error}");
        }
    });
}

fn register_escape_shortcut(app: &AppHandle) {
    let shortcut = Shortcut::new(None, Code::Escape);
    let app_for_worker = (*app).clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = app_for_worker.global_shortcut().register(shortcut) {
            log::error!("Unable to register the temporary Escape shortcut: {error}");
        }
    });
}

#[tauri::command]
async fn search(
    state: tauri::State<'_, search::SearchState>,
    query: String,
) -> Result<search::SearchReply, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.search(&query))
        .await
        .map_err(|error| format!("Search worker failed: {error}"))
}

#[tauri::command]
fn is_window_visible(window: tauri::WebviewWindow) -> Result<bool, String> {
    window.is_visible().map_err(|error| {
        log::error!("Failed to check Spotlight Search visibility: {error}");
        error.to_string()
    })
}

#[tauri::command]
async fn launch(state: tauri::State<'_, search::SearchState>, id: String) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.launch(&id))
        .await
        .map_err(|error| format!("Launch worker failed: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    if shortcut.matches(Modifiers::empty(), Code::Escape) {
                        if let Some(window) = app.get_webview_window("main") {
                            match window.is_visible() {
                                Ok(true) => {
                                    if let Err(error) = app.emit("spotlight-close", ()) {
                                        log::error!(
                                            "Unable to notify Spotlight Search that it closed: {error}"
                                        );
                                    }
                                    if let Err(error) = window.hide() {
                                        log::error!(
                                            "Unable to hide Spotlight Search after Escape: {error}"
                                        );
                                    }
                                }
                                Ok(false) => {}
                                Err(error) => log::error!(
                                    "Unable to check Spotlight Search visibility: {error}"
                                ),
                            }
                        }
                        unregister_escape_shortcut(app.clone());
                        return;
                    }

                    if let Some(window) = app.get_webview_window("main") {
                        match window.is_visible() {
                            Ok(true) => return,
                            Ok(false) => {}
                            Err(error) => {
                                log::error!(
                                    "Unable to check Spotlight Search visibility before opening: {error}"
                                );
                                return;
                            }
                        }
                        if let Err(error) = window.show() {
                            log::error!("Unable to show Spotlight Search: {error}");
                            return;
                        }
                        register_escape_shortcut(app);
                        if let Err(error) = window.set_focus() {
                            log::error!("Unable to focus Spotlight Search: {error}");
                            return;
                        }
                        if let Err(error) = app.emit("spotlight-open", ()) {
                            log::error!(
                                "Unable to notify Spotlight Search that it opened: {error}"
                            );
                        }
                    }
                })
                .build(),
        )
        .manage(search::SearchState::new())
        .invoke_handler(tauri::generate_handler![
            hide_window,
            is_window_visible,
            search,
            launch
        ])
        .setup(|app| {
            let _ = env_logger::try_init();
            if let Err(error) = app
                .global_shortcut()
                .register(Shortcut::new(Some(Modifiers::SUPER), Code::Space))
            {
                log::warn!(
                    "Unable to register Win+Space (Windows may reserve it for input-language switching): {error}"
                );
                if let Err(fallback_error) = app
                    .global_shortcut()
                    .register(Shortcut::new(Some(Modifiers::CONTROL), Code::Space))
                {
                    log::error!("Unable to register Ctrl+Space fallback: {fallback_error}");
                    if let Some(window) = app.get_webview_window("main") {
                        if let Err(show_error) = window.show() {
                            log::error!(
                                "Unable to show Spotlight Search after shortcut registration failed: {show_error}"
                            );
                        }
                    }
                } else {
                    log::info!("Registered Ctrl+Space fallback to open Spotlight Search.");
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                match window.current_monitor() {
                    Ok(Some(monitor)) => {
                        let monitor_position = monitor.position();
                        let monitor_size = monitor.size();
                        let scale: f64 = match window.scale_factor() {
                            Ok(scale) => scale,
                            Err(error) => {
                                log::warn!("Unable to read window scale factor: {error}");
                                1.0
                            }
                        };
                        let width = (760.0 * scale).round() as i32;
                        let padding = (40.0 * scale).round() as i32;
                        let x = monitor_position.x + ((monitor_size.width as i32 - width) / 2);
                        let y = monitor_position.y
                            + (monitor_size.height as f64 * 0.22).round() as i32
                            - padding;

                        if let Err(error) = window.set_position(tauri::PhysicalPosition::new(x, y))
                        {
                            log::warn!("Unable to position the Spotlight demo window: {error}");
                        }
                    }
                    Ok(None) => log::warn!("No monitor found for the Spotlight demo window."),
                    Err(error) => {
                        log::warn!("Unable to determine the Spotlight demo monitor: {error}")
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("Failed to run Spotlight Search demo: {error}");
    }
}
