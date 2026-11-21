# CmdPal — Product Brief and Authoritative Requirements

## 1. Product identity

**Name:** CmdPal  
**Product:** A fast, lightweight Windows command palette.  
**Design inspiration:** Keypirinha's low resource footprint and speed, combined with the modern visual quality and useful command-discovery experience associated with Microsoft PowerToys Command Palette. Do not copy proprietary artwork or code.

## 2. Greenfield status

This is a brand-new application. The user has not built CmdPal before. There is no existing implementation, source repository, old executable, or previous build to audit or migrate. The supplied conversations are requirements only. Create the complete application from scratch based on this brief.

## 3. Core goals

- Very fast activation, startup, search, and interaction.
- Very low RAM, CPU, disk, and package footprint.
- Remain responsive on older PCs, including approximately 4 GB RAM, an i3-class CPU, and an HDD.
- Modern, polished, compact Windows interface.
- Windows 10 and Windows 11 compatibility, with exact supported Windows versions documented and verified.
- Primary language selected between C++ and Rust by engineering evaluation.
- No mandatory .NET runtime installation.
- Local core features work fully offline.
- GitHub Actions performs cloud builds because the user cannot compile locally.
- Every feature has its own UI and logic files in a cohesive module.
- Main entry point is minimal and only delegates to bootstrap.

## 4. Main command palette

### Activation and lifecycle
- User-configurable global keyboard shortcut.
- Palette appears quickly and remains responsive.
- Palette is always-on-top while active.
- Palette auto-hides on focus loss, outside click, launching an application, opening a file, or executing a command, except where a specific window behavior says otherwise.
- Optional setting to position the palette near the mouse cursor.
- Palette must always remain fully inside the usable monitor work area, including multiple monitors and DPI scaling.
- Palette remains invokable while the Notes window is open.
- Optional launch-at-Windows-startup setting, disabled or clearly opt-in by default; startup must be minimal.

### Search and suggestions
- Search installed applications, commands, settings, files, folders, quick folders, bookmarks, and registered bang providers as appropriate.
- Suggestions are beautifully organised and categorised, with relevance ranking and clear visual hierarchy.
- Setting to show/hide suggestions at startup and customise suggestion behavior.
- Users can hide individual installed applications and unwanted suggestions/results; exclusions persist.
- Query text and caret position must never be lost during asynchronous initialization, search-provider changes, or result updates.
- Long input must remain editable and visible; the input area should grow sensibly or scroll horizontally/vertically without hiding entered text or losing the caret.

### Navigation
- Full mouse and keyboard support.
- Keep Up, Down, Tab, Shift+Tab, Page Up, Page Down, and Enter navigation.
- Enter launches/opens/runs the selected result.
- Do not implement Alt+1 through Alt+9 result shortcuts or numeric result hints.

## 5. Application search and launch

- Discover installed Windows applications using safe, documented Windows mechanisms.
- Search application names and useful metadata with relevant ranking.
- Launch an application only after explicit Enter or click.
- Hide the palette after successful launch.
- Let users hide selected applications from all suggestions without uninstalling them.
- Avoid repeated full discovery scans during each keystroke; cache and refresh appropriately without delaying initial palette display.
- Provide useful app icons where available without bloating startup.

## 6. Commands and user-defined tasks

- Include a small set of useful built-in CmdPal commands for its own functions.
- Support user-defined commands/tasks: add, edit, enable/disable, reorder, delete, and search.
- Use clear command names, descriptions, input/argument definitions, and result presentation.
- Execute only on explicit user confirmation (Enter/click).
- Handle arguments safely; do not build commands by unsafe concatenation of untrusted text.
- Clearly identify commands that may change files/system state and require confirmation when appropriate.
- Do not add arbitrary dangerous command examples or hidden actions.

## 7. Web bangs and external URL opening — no embedded browser

- Support user-configurable web/search bangs and aliases.
- Provide sensible initial aliases, including Google as the default search provider and YouTube as a selectable provider. Alias examples include `w`, `g`, and `yt`; users can edit aliases and names.
- When the user types a configured bang and presses Space, convert the alias into a labelled chip while preserving the rest of the query. Example: `g ` can become a `Google` chip; `yt ` can become a `YouTube` chip. The display name is configurable.
- When a bang is active with no query, show relevant saved bookmarks/suggestions.
- Users can add, edit, remove, reorder, and launch bookmarks.
- Detect arbitrary valid HTTP/HTTPS URLs, including deep paths and page routes, not only famous domains. Preserve valid paths, query strings, ports, and fragments while validating safely.
- On Enter/click, open the generated search URL or direct URL using the Windows default browser.
- Do not open a website automatically just because a bang was typed. Do not lose query text while switching bang modes.
- CmdPal itself must not fetch, render, embed, preview, or host web pages.

## 8. File search and quick folders

### Custom path indexing
- User selects one or more root folder paths.
- Recursively index eligible files and folders under each selected root; never scan the entire drive by default.
- Search file/folder names and useful metadata quickly.
- Index incrementally in the background. Do not block palette startup.
- Support add/remove/enable/disable indexing roots.
- Optionally offer common Windows folders such as Desktop, Documents, and Downloads, but each must be individually removable and not mandatory.
- Exclusions for subfolders, patterns, and file types.
- Show indexing status, errors, last scan, and allow pause/rebuild/clear index.
- Keep content/full-text indexing optional and clearly explain resource cost. Name/path indexing is the lightweight default.
- Open selected file/folder using Windows shell mechanisms and hide the palette.

### Quick folders
- Keep a distinct Quick Folders feature for fast navigation/opening.
- Users can add, remove, enable/disable, and reorder individual quick folders.
- Explain the difference between Quick Folders shortcuts and indexing roots. Removing a quick folder must not silently remove its indexing root unless the user explicitly chooses that action.

## 9. Notes and Markdown editor

### Window behavior
- Notes open in a separate standalone, frameless window.
- No title bar, no built-in minimize/maximize/close controls.
- Losing focus must not close the note window.
- Escape and Ctrl+W close the notes window after ensuring pending edits are safely saved.
- The note window is not always-on-top. Other windows may overlap it.
- The main palette can be invoked while Notes is open.

### Editor behavior
- Provide local Markdown notes with create, open, rename, search, and delete operations.
- One editable surface with live Markdown rendering/formatting; do not force users into separate read-only preview and edit panes.
- Common Markdown/GFM support: headings, bold, italic, strikethrough, inline code, fenced code blocks, ordered/unordered lists, task lists, tables, blockquotes, horizontal rules, links, and images (local/reference-safe behavior; no automatic remote fetch in the background).
- When the caret is inside a formatted construct, show the relevant Markdown delimiters/syntax in a dimmed style so the user can edit source. When the caret moves outside, visually render the formatted text without showing delimiters, while preserving editable source semantics.
- Helpful syntax completion/continuation must be non-destructive and must not unexpectedly duplicate or corrupt text.
- A bottom formatting/status bar shows commonly used Markdown syntax/actions.
- The formatting bar must scroll horizontally when its contents exceed available width.
- Clicking a formatting action inserts/wraps the syntax at the current caret position and returns focus to the editor.
- Users can reorder formatting actions. Remember the preferred order separately for each note/document as requested.
- The editor expands sensibly for short notes and scrolls for longer notes. No text may disappear when content exceeds the initial input area.
- Autosave and crash-safe recovery are mandatory. Save locally and avoid losing text on Escape/Ctrl+W.
- Keep editor opening fast; initialise it only when invoked.

## 10. Settings

- Provide a dedicated graphical settings window/page; it is the normal way to configure CmdPal.
- Also retain an advanced human-editable configuration file.
- The GUI and file must represent one canonical settings model. Do not maintain two conflicting sources of truth.
- GUI edits should validate values and persist safely.
- Search settings from the main palette and jump to the relevant settings area.
- Settings should cover at minimum: global shortcut, Windows startup, cursor positioning, suggestions visibility, compact mode, animation/reduced motion, app/result exclusions, bang aliases/providers, bookmarks, indexing roots, quick folders, exclusions, custom commands, note preferences, and file-index controls.

## 11. Appearance and interaction

- Modern Windows 10/11 visual design, clean typography, good spacing, consistent icons, clear focus states, and polished suggestion cards/rows.
- Compact mode and configurable density.
- Mouse and keyboard workflows should be equally usable.
- Restrained short animations only; never sacrifice responsiveness. Offer reduced-motion/animation controls where practical.
- Respect Windows DPI scaling, screen bounds, multiple monitors, keyboard focus, and accessibility basics.
- Include an original CmdPal application icon and consistent window/tray icon where applicable.

## 12. Removed features — never implement

See `REMOVED_FEATURES_DO_NOT_IMPLEMENT.md`. The exclusions include numeric Alt+1–Alt+9 result launching/hints, Windows switcher, unit conversion, number-base conversion, colour converter, general text tools, and the embedded browser/browser-engine/extension system.

Do not add music playback or unrelated extra utilities based on ambiguous wording. Do not add features merely because the user previously said "many more useful features"; propose them separately for approval.

## 13. Offline and security expectations

- App launcher, search over local data, file indexing, quick folders, commands, notes, and settings must work offline.
- No telemetry, analytics, tracking, advertisements, background network calls, silent update checks, or automatic web requests.
- The only intended network interaction is through a user-initiated external browser handoff for web search/URL opening.
- Do not require administrator privileges by default.
- Do not hide processes, inject code, evade detection, disable antivirus, obfuscate behavior, or use suspicious packers.
- Explain any required OS permissions and keep them minimal.
- Build transparently and reproducibly. Do not promise that every antivirus product will always classify every build as safe.

## 14. Platform, stack, and delivery

- Target Windows 10 and Windows 11; identify and document the exact minimum supported Windows release/build.
- Compare C++ and Rust and select one primary implementation language.
- No mandatory .NET runtime prerequisite.
- No embedded browser/WebView2/Chromium/CEF/Electron.
- GitHub Actions builds and tests the application on a Windows runner.
- Produce downloadable Windows x64 artifacts, with clear package type, prerequisites, checksum, and build version.
- The user should be able to download a build without compiling locally.

## 15. Acceptance standard

CmdPal is not complete merely because a window opens. Every in-scope feature must be implemented, connected, tested, documented, and represented in the feature progress catalogue. The whole application must build in CI. Performance, security, keyboard/mouse behavior, Windows compatibility, and removal boundaries must be checked and reported honestly.
