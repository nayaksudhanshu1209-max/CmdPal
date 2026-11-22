# CmdPal
Fast, lightweight Windows command palette (Rust, native Win32, no .NET, no embedded browser). Requirements: `docs/requirements/APP_BRIEF.md`. Exclusions: `docs/requirements/REMOVED_FEATURES_DO_NOT_IMPLEMENT.md`. Progress: `docs/FEATURE_CATALOGUE.md`. Decisions: `docs/DECISIONS/DECISION_LOG.md`.

## Status
All planned phases have a first implementation, **not yet verified**: it has not been built or run by CI at the time of writing (see the catalogue notes). Expect compile fixes on the first CI runs.

## Using it
- **Ctrl+Alt+Space** (configurable) shows the palette; **Esc**, clicking away, or running something hides it. Up/Down/Tab/Shift+Tab/PageUp/PageDown move, **Enter** or click runs. **Right-click** (or Ctrl+Delete) hides an app/result permanently (undo in Settings > Hidden results).
- Type to search apps, commands, settings, bookmarks, quick folders, notes, files. `g <text>` / `yt <text>` / `w <text>`: alias + Space becomes a chip; an empty chip lists your bookmarks. Typing a URL (`example.com/path?x=1`) offers to open it in your default browser. `n <text>`: notes. "settings" opens the settings window; "quit" exits.
- Files: add folders under Settings > Files (nothing is indexed by default).
- Notes: frameless window, Esc / Ctrl+W save and close. See `src/features/notes/README.md`.
- Alt+F4 on the palette also quits.

## Data locations
`%APPDATA%\CmdPal\settings.toml`, `%APPDATA%\CmdPal\notes\*.md`, `%LOCALAPPDATA%\CmdPal\{apps.cache,file-index.tsv,cmdpal.log}`.

## Getting a build
Push to GitHub, open Actions > "CI (Windows)" > the run > download `CmdPal-<version>-win-x64-portable` (unsigned portable ZIP, release build, SHA-256 file included; no installer; no prerequisites expected - static CRT). Windows SmartScreen may warn because it is unsigned. Tag `vX.Y.Z` to run the release workflow.
