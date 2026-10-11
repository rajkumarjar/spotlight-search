# Spotlight Search

A fast, local-first Windows launcher inspired by macOS Spotlight. Press a global shortcut, type an app, file, or folder name, and open a result without leaving the keyboard.

## Features

- Search installed desktop apps from Start Menu shortcuts and registered Microsoft Store apps.
- Search file and folder names using the Windows Search index (`SystemIndex`).
- Fuzzy, multiword application matching; results update as you type.
- Keyboard-first navigation, launch, and Escape-to-close.
- Translucent, Spotlight-inspired panel with Windows acrylic material.
- Search stays on the device. There are no web suggestions, telemetry, or query uploads.

## Download and install

Run the included [Spotlight Search setup installer](./release/Spotlight-Search-Setup.exe), or download the latest setup `.exe` from the repository's GitHub Releases page. The NSIS installer installs for the current Windows user and does not require administrator access.

The installer is generated at:

```text
release/Spotlight-Search-Setup.exe
```

The installer may need an internet connection to download the Microsoft Edge WebView2 bootstrapper if WebView2 Runtime is not already installed. Windows 10 and 11 commonly include this runtime. This project is not currently code-signed, so Windows SmartScreen may show an unknown-publisher warning for locally built or first-party installer downloads.

## Requirements

- Windows 10 or Windows 11, x64
- WebView2 Runtime
- For development: Node.js 20.19+ and npm
- Rust stable with the `x86_64-pc-windows-msvc` target
- Visual Studio Build Tools with the C++ workload

## Use

- Open the launcher with **Win+Space**. Windows may reserve this shortcut for input-language switching; if registration is unavailable, the app falls back to **Ctrl+Space**.
- Type to search applications, indexed files, and indexed folders.
- Use **Up/Down** to move through results, **Enter** to open the selected result, or **Alt+1** through **Alt+9** to open a numbered result.
- Press **Esc** to close the panel. The query is reset whenever the launcher opens, so a closed search is never restored.
- Clicking outside the panel closes it.

## Search coverage and privacy

### Applications

The app catalog is built from the per-user and common Start Menu folders and registered Store apps. Start Menu applications become searchable before the Store-app enumeration finishes. Matching uses the `nucleo-matcher` fuzzy matcher.

### Files and folders

File and folder names are queried from Windows Search's local `SystemIndex`. Results therefore depend on Windows Search being available and the searched location/file type being included in Windows indexing. Add locations or file types in **Windows Settings → Privacy & security → Searching Windows** (the exact page name may vary by Windows version). Content is not read or uploaded; the app searches names through the index.

If the Windows Search service is disabled, the index is unavailable, or an item is outside indexed locations, the corresponding results will not be available. The application search continues to work independently.

### Network behavior

Search is local. The application does not include web-content suggestions or a cloud search provider. The NSIS installer can download the WebView2 bootstrapper only when needed during installation.

## Build from source

Clone the repository and run:

```powershell
npm ci
npm run tauri dev
```

The app runs as a hidden background window in development and opens with its registered global shortcut.

## Quality checks and production installer

```powershell
npm run lint
npm run format:check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Build the Windows NSIS installer:

```powershell
npm run build:installer
```

The installer `.exe` is placed under `src-tauri/target/release/bundle/nsis/`. Building the installer invokes the frontend production build as part of Tauri's configured build process. Other operating systems cannot produce this Windows installer; build on Windows with the toolchain above.

## Troubleshooting

- **Win+Space does not open the launcher:** Windows may own the shortcut. Try **Ctrl+Space** or change the conflicting Windows input-language shortcut.
- **No files or folders appear:** Confirm Windows Search is running and that the desired folders/file types are indexed. The project does not crawl the entire disk as a fallback.
- **The installer reports a missing WebView:** Install the current Microsoft Edge WebView2 Runtime, then run the installer again.
- **SmartScreen warns about the installer:** Releases are currently unsigned. Verify that the installer came from this repository's Releases page; code signing is not configured.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development and pull request guidance. To report a vulnerability, see [SECURITY.md](./SECURITY.md).

## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE).
