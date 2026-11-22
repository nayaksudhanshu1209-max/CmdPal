# CmdPal — Feature Catalogue and Acceptance Criteria

Use this file to track implementation of the new app. This is a forward-looking delivery checklist, not an audit of an existing application. Status values: `Not started`, `In progress`, `Implemented`, `Verified`.

## A. Core palette and lifecycle

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| CORE-01 | Global hotkey | User can configure a global shortcut; conflicts/errors are reported; hotkey opens palette. | P0 | Implemented |
| CORE-02 | Always-on-top palette | Palette remains above normal windows while active. | P0 | Implemented |
| CORE-03 | Auto-hide | Palette hides on focus loss, outside click, app/file launch, or command execution. | P0 | Implemented |
| CORE-04 | Screen-bounded placement | Palette remains within monitor work area; multiple monitors and DPI handled. | P0 | Implemented |
| CORE-05 | Cursor positioning | Optional placement near mouse cursor with safe screen-edge adjustment. | P1 | Implemented |
| CORE-06 | Keyboard navigation | Up, Down, Tab, Shift+Tab, Page Up, Page Down and Enter work consistently. | P0 | Implemented |
| CORE-07 | Mouse support | Click, scrolling, selection and controls work. | P0 | Implemented |
| CORE-08 | Input persistence | Query and caret never disappear during asynchronous updates. | P0 | Implemented |
| CORE-09 | Long input | Long queries/notes remain reachable and editable without clipping or lost caret. | P0 | Implemented |
| CORE-10 | Suggestions visibility | Setting controls suggestion visibility at startup. | P1 | Implemented |
| CORE-11 | Categorised suggestions | Results grouped and ranked with clear categories. | P0 | Implemented |
| CORE-12 | Exclusions | User can hide individual applications/results; preference persists. | P1 | Implemented |
| CORE-13 | Compact mode | User can configure compact density. | P1 | Implemented |
| CORE-14 | Animation controls | Restrained animation and reduced-motion/disable option where practical. | P2 | Implemented |
| CORE-15 | App icon | Original CmdPal icon used consistently. | P1 | Implemented |
| CORE-16 | Windows startup | Optional opt-in startup, with minimal startup work and clear disable control. | P1 | Implemented |

## B. Application search

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| APP-01 | App discovery | Safe discovery of installed apps; duplicates handled. | P0 | Implemented |
| APP-02 | App search | Search names/metadata with useful ranking. | P0 | Implemented |
| APP-03 | Launch | Enter/click launches selected app; palette hides. | P0 | Implemented |
| APP-04 | App exclusions | Hide selected applications from suggestions without uninstalling. | P1 | Implemented |
| APP-05 | Icons | Resolve/cache application icons without slowing startup. | P2 | Implemented |

## C. Commands and tasks

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| CMD-01 | Built-in commands | CmdPal core actions are discoverable and executable. | P1 | Implemented |
| CMD-02 | User-defined commands | GUI supports create/edit/enable/disable/reorder/delete. | P1 | Implemented |
| CMD-03 | Safe arguments | Parameterised arguments; no unsafe string concatenation; clear confirmation for high-impact actions. | P0 | Implemented |
| CMD-04 | Result presentation | Command descriptions/status/errors shown clearly. | P1 | Implemented |
| CMD-05 | Hide after execution | Palette hides on successful command execution where appropriate. | P0 | Implemented |

## D. Web bangs and external browser handoff — no embedded browser

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| WEB-01 | Configurable aliases | User can add/edit/remove aliases such as `w`, `g`, `yt`. | P1 | Implemented |
| WEB-02 | Bang chips | Alias+Space becomes a labelled chip without losing query text. | P1 | Implemented |
| WEB-03 | Google default | Google is initial default search provider. | P1 | Implemented |
| WEB-04 | Provider search | Bang creates a correctly encoded provider-specific search URL. | P1 | Implemented |
| WEB-05 | Empty bang suggestions | Empty bang shows relevant saved bookmarks. | P1 | Implemented |
| WEB-06 | Bookmarks | Add/edit/remove/reorder bookmarks; open in default browser. | P1 | Implemented |
| WEB-07 | Arbitrary URL detection | Valid HTTP/HTTPS deep paths, query strings, fragments and ports work; not limited to popular domains. | P1 | Implemented |
| WEB-08 | URL validation | Reject malformed URLs and unsafe schemes. | P0 | Implemented |
| WEB-09 | External open | Enter/click hands off to Windows default browser; no app-owned web rendering. | P1 | Implemented |

## E. File search and quick folders

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| FILE-01 | Custom roots | User chooses one or more indexing folders. | P1 | Implemented |
| FILE-02 | Recursive indexing | Index eligible paths beneath selected roots, not the whole drive. | P1 | Implemented |
| FILE-03 | Optional default folders | Desktop/Documents/Downloads offered individually and removable. | P1 | Implemented |
| FILE-04 | Incremental background index | Indexing never blocks palette startup; changed content updates incrementally. | P1 | Implemented |
| FILE-05 | Name/metadata search | Fast filename/folder/metadata search. | P1 | Implemented |
| FILE-06 | Content index option | Full-text content indexing optional and off by default. | P2 | Implemented |
| FILE-07 | Exclusions | Exclude subfolders, patterns and file types. | P1 | Implemented |
| FILE-08 | Index controls | Status, errors, pause, clear and rebuild. | P1 | Implemented |
| FILE-09 | Open result | File/folder opens via Windows; palette hides. | P1 | Implemented |
| FILE-10 | Quick folders | Add/remove/enable/disable/reorder separate from indexing roots. | P1 | Implemented |

## F. Notes and Markdown editor

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| NOTE-01 | Note bang | Configurable note alias opens note editor or selected note. | P1 | Implemented |
| NOTE-02 | Standalone window | Separate frameless note window; no top bar or window buttons. | P1 | Implemented |
| NOTE-03 | Focus behavior | Losing focus does not close note. | P0 | Implemented |
| NOTE-04 | Close shortcuts | Escape and Ctrl+W close after safe save. | P0 | Implemented |
| NOTE-05 | Unified live Markdown | Single editable surface with live formatting; no separate fixed preview pane. | P1 | Implemented |
| NOTE-06 | Caret-aware syntax | Syntax is dimly visible while caret is inside formatted construct; visually hidden when caret leaves. | P1 | Implemented |
| NOTE-07 | Markdown support | Headings, emphasis, strike, code, lists, tasks, tables, quotes, rules, links, images. | P1 | Implemented |
| NOTE-08 | Syntax assistance | Helpful, non-destructive syntax completion/continuation. | P2 | Implemented |
| NOTE-09 | Formatting bar | Bottom syntax/action bar scrolls horizontally. | P1 | Implemented |
| NOTE-10 | Insert at caret | Toolbar actions insert/wrap at caret and restore editor focus. | P1 | Implemented |
| NOTE-11 | Per-note toolbar order | Toolbar action order can be changed and saved per note. | P2 | Implemented |
| NOTE-12 | Note management | Create/open/rename/search/delete local Markdown notes. | P1 | Implemented |
| NOTE-13 | Autosave/recovery | Atomic/safe saves and recovery protect against data loss. | P0 | Implemented |
| NOTE-14 | Adaptive size/scroll | Editor adapts to content; long notes scroll without clipping. | P1 | Implemented |

## G. Settings and appearance

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| SET-01 | Settings GUI | Dedicated graphical settings interface. | P1 | Implemented |
| SET-02 | Canonical settings model | GUI and human-editable config share one source of truth. | P0 | Implemented |
| SET-03 | Config validation | Invalid edits are caught; safe defaults/recovery available. | P1 | Implemented |
| SET-04 | Settings search | Palette can search settings and open relevant settings section. | P1 | Implemented |
| SET-05 | Preference coverage | Hotkey, startup, cursor, suggestions, compact mode, exclusions, bangs, bookmarks, paths, commands, notes, index, animation. | P1 | Implemented |

## H. Quality, platform and delivery

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| QUAL-01 | Windows 10/11 | Exact minimum supported versions documented and tested in CI where practical. | P0 | In progress |
| QUAL-02 | No .NET prerequisite | Clean target Windows environment does not require user-installed .NET. | P0 | Implemented |
| QUAL-03 | C++ or Rust | Compare both and document selected language. | P0 | Implemented |
| QUAL-04 | Lightweight | Startup, idle RAM/CPU, search latency and package size measured. | P0 | In progress |
| QUAL-05 | Offline local core | Local functionality works offline; no automatic network calls. | P0 | Implemented |
| QUAL-06 | Safe transparent behavior | Least privilege, reproducible build, no evasion/telemetry. | P0 | In progress |
| QUAL-07 | GitHub Actions | CI builds, tests and uploads Windows artifact. | P0 | In progress |
| QUAL-08 | Modular structure | Independent feature UI, logic, interface, tests and docs. | P0 | Implemented |
| QUAL-09 | Tiny entry point | Entry point delegates to bootstrap only. | P0 | Implemented |

## Progress rules

- `Implemented` means code exists and is integrated.
- `Verified` means relevant tests passed or a documented manual check was completed.
- Never mark anything Verified based only on code review or an AI-generated claim.
- Update this catalogue whenever a feature changes state.


## Status notes (full first implementation)
- **Nothing is Verified.** All code was written without a compiler in the authoring environment; "Implemented" means the code exists and is wired in, not that it has built or been tested. Items become Verified only after CI is green and a test or documented manual check covers them.
- QUAL-01 (Windows scope documented but only `windows-2022` exercised), QUAL-04 (CI smoke run records startup time and peak memory, not measured on a 4 GB/i3/HDD PC), QUAL-06 (no telemetry/network by design; reproducible-build and dependency inventory still to do), QUAL-07 (workflows written, not yet run) are In progress.
- Known simplifications: CORE-14 animation is a short fade (setting turns it off); APP-01 finds apps from Start Menu shortcuts only (no Store/UWP apps without shortcuts); FILE-04 incremental = directory-modification-time based (no live file watcher); FILE-06 content search is whole-word over small text files; NOTE-11 toolbar reorder is by mouse drag; Settings pages use native controls with Tab navigation via IsDialogMessage.

