# Contributing

Thanks for helping improve Spotlight Search. Contributions should preserve the local-first search model and the existing Tauri 2, React, TypeScript, and Rust stack.

## Before opening a pull request

1. Create a focused branch from the default branch.
2. Keep changes scoped to one bug or feature and explain user-visible behavior changes.
3. Run the applicable checks from the repository root:

   ```powershell
   npm ci
   npm run lint
   npm run format:check
   npm run build
   cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
   cargo test --manifest-path src-tauri/Cargo.toml
   cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
   ```

4. Test Windows-specific changes on Windows 10 or 11, including the relevant shortcut, search provider, and installer behavior.
5. Describe the change, verification performed, and any known limitations in the pull request.

Do not include credentials, personal search data, generated build output, or unrelated formatting changes. Do not add online search or telemetry without explicit product approval and clear privacy documentation.
