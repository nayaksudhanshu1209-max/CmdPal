# CmdPal
Fast, lightweight Windows command palette (Rust, Win32, no .NET, no embedded browser).
Requirements: `docs/requirements/APP_BRIEF.md`. Hard exclusions: `docs/requirements/REMOVED_FEATURES_DO_NOT_IMPLEMENT.md`. Progress: `docs/FEATURE_CATALOGUE.md`. Decisions: `docs/DECISIONS/DECISION_LOG.md`.

## Current state: foundation shell (not yet built in CI)
Run `CmdPal.exe`; press **Ctrl+Alt+Space** to toggle a placeholder palette window. **Esc** or clicking away hides it; **Alt+F4** (while focused) quits. The hotkey is editable in `%APPDATA%\CmdPal\settings.toml` (`[palette] hotkey = "..."`).

## Getting a build
Push to GitHub, open Actions > "CI (Windows)" > the run > download the `CmdPal-<version>-win-x64-portable` artifact. It is an **unsigned portable ZIP (release build)** with a SHA-256 file. No installer; no prerequisites expected (static CRT). Windows may show a SmartScreen warning for unsigned apps.
