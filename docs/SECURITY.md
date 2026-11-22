# Security and privacy notes
- No telemetry, analytics, update checks or background network calls. The only network use is the default browser being opened by `ShellExecute` after an explicit Enter/click on an http(s) URL.
- URLs: only `http`/`https`; no user-info (`user@host`), no whitespace/control/backslash; validated when displayed and again before opening (`core/url.rs`, tested).
- User commands: program and arguments are separate values passed to `std::process::Command` (no shell); `{input}` fills inside one argument; confirmation dialog shows the exact command line (on by default).
- Data is local: settings and notes in `%APPDATA%\CmdPal`, caches/log in `%LOCALAPPDATA%\CmdPal`. The log never contains note text or command arguments.
- Saves are atomic (temp + rename); invalid settings are never overwritten (offer to move aside as `settings.toml.bad`).
- Runs as a normal user; the optional startup entry is a per-user `HKCU\...\Run` value, removable from Settings.
- No process injection, hiding, anti-debugging, packers or obfuscation. Builds are unsigned; antivirus classification cannot be guaranteed - report false positives through the vendor's process.
- Dependencies: `serde`, `toml`, `windows` (build: `winresource`). Run `cargo tree` for the full inventory; commit `Cargo.lock`.
- Known gaps: CI actions are pinned to major tags, not commit SHAs; no signing; no formal SBOM yet.
