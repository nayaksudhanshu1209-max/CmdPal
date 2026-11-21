# CmdPal — Feature Catalogue and Acceptance Criteria

Use this file to track implementation of the new app. This is a forward-looking delivery checklist, not an audit of an existing application. Status values: `Not started`, `In progress`, `Implemented`, `Verified`.

## A. Core palette and lifecycle

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| CORE-01 | Global hotkey | User can configure a global shortcut; conflicts/errors are reported; hotkey opens palette. | P0 | In progress |
| CORE-02 | Always-on-top palette | Palette remains above normal windows while active. | P0 | In progress |
| CORE-03 | Auto-hide | Palette hides on focus loss, outside click, app/file launch, or command execution. | P0 | In progress |
| CORE-04 | Screen-bounded placement | Palette remains within monitor work area; multiple monitors and DPI handled. | P0 | In progress |
| CORE-05 | Cursor positioning | Optional placement near mouse cursor with safe screen-edge adjustment. | P1 | Not started |
| CORE-06 | Keyboard navigation | Up, Down, Tab, Shift+Tab, Page Up, Page Down and Enter work consistently. | P0 | Not started |
| CORE-07 | Mouse support | Click, scrolling, selection and controls work. | P0 | Not started |
| CORE-08 | Input persistence | Query and caret never disappear during asynchronous updates. | P0 | Not started |
| CORE-09 | Long input | Long queries/notes remain reachable and editable without clipping or lost caret. | P0 | Not started |
| CORE-10 | Suggestions visibility | Setting controls suggestion visibility at startup. | P1 | Not started |
| CORE-11 | Categorised suggestions | Results grouped and ranked with clear categories. | P0 | Not started |
| CORE-12 | Exclusions | User can hide individual applications/results; preference persists. | P1 | Not started |
| CORE-13 | Compact mode | User can configure compact density. | P1 | Not started |
| CORE-14 | Animation controls | Restrained animation and reduced-motion/disable option where practical. | P2 | Not started |
| CORE-15 | App icon | Original CmdPal icon used consistently. | P1 | Not started |
| CORE-16 | Windows startup | Optional opt-in startup, with minimal startup work and clear disable control. | P1 | Not started |

## B. Application search

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| APP-01 | App discovery | Safe discovery of installed apps; duplicates handled. | P0 | Not started |
| APP-02 | App search | Search names/metadata with useful ranking. | P0 | Not started |
| APP-03 | Launch | Enter/click launches selected app; palette hides. | P0 | Not started |
| APP-04 | App exclusions | Hide selected applications from suggestions without uninstalling. | P1 | Not started |
| APP-05 | Icons | Resolve/cache application icons without slowing startup. | P2 | Not started |

## C. Commands and tasks

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| CMD-01 | Built-in commands | CmdPal core actions are discoverable and executable. | P1 | Not started |
| CMD-02 | User-defined commands | GUI supports create/edit/enable/disable/reorder/delete. | P1 | Not started |
| CMD-03 | Safe arguments | Parameterised arguments; no unsafe string concatenation; clear confirmation for high-impact actions. | P0 | Not started |
| CMD-04 | Result presentation | Command descriptions/status/errors shown clearly. | P1 | Not started |
| CMD-05 | Hide after execution | Palette hides on successful command execution where appropriate. | P0 | Not started |

## D. Web bangs and external browser handoff — no embedded browser

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| WEB-01 | Configurable aliases | User can add/edit/remove aliases such as `w`, `g`, `yt`. | P1 | Not started |
| WEB-02 | Bang chips | Alias+Space becomes a labelled chip without losing query text. | P1 | Not started |
| WEB-03 | Google default | Google is initial default search provider. | P1 | Not started |
| WEB-04 | Provider search | Bang creates a correctly encoded provider-specific search URL. | P1 | Not started |
| WEB-05 | Empty bang suggestions | Empty bang shows relevant saved bookmarks. | P1 | Not started |
| WEB-06 | Bookmarks | Add/edit/remove/reorder bookmarks; open in default browser. | P1 | Not started |
| WEB-07 | Arbitrary URL detection | Valid HTTP/HTTPS deep paths, query strings, fragments and ports work; not limited to popular domains. | P1 | Not started |
| WEB-08 | URL validation | Reject malformed URLs and unsafe schemes. | P0 | Not started |
| WEB-09 | External open | Enter/click hands off to Windows default browser; no app-owned web rendering. | P1 | Not started |

## E. File search and quick folders

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| FILE-01 | Custom roots | User chooses one or more indexing folders. | P1 | Not started |
| FILE-02 | Recursive indexing | Index eligible paths beneath selected roots, not the whole drive. | P1 | Not started |
| FILE-03 | Optional default folders | Desktop/Documents/Downloads offered individually and removable. | P1 | Not started |
| FILE-04 | Incremental background index | Indexing never blocks palette startup; changed content updates incrementally. | P1 | Not started |
| FILE-05 | Name/metadata search | Fast filename/folder/metadata search. | P1 | Not started |
| FILE-06 | Content index option | Full-text content indexing optional and off by default. | P2 | Not started |
| FILE-07 | Exclusions | Exclude subfolders, patterns and file types. | P1 | Not started |
| FILE-08 | Index controls | Status, errors, pause, clear and rebuild. | P1 | Not started |
| FILE-09 | Open result | File/folder opens via Windows; palette hides. | P1 | Not started |
| FILE-10 | Quick folders | Add/remove/enable/disable/reorder separate from indexing roots. | P1 | Not started |

## F. Notes and Markdown editor

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| NOTE-01 | Note bang | Configurable note alias opens note editor or selected note. | P1 | Not started |
| NOTE-02 | Standalone window | Separate frameless note window; no top bar or window buttons. | P1 | Not started |
| NOTE-03 | Focus behavior | Losing focus does not close note. | P0 | Not started |
| NOTE-04 | Close shortcuts | Escape and Ctrl+W close after safe save. | P0 | Not started |
| NOTE-05 | Unified live Markdown | Single editable surface with live formatting; no separate fixed preview pane. | P1 | Not started |
| NOTE-06 | Caret-aware syntax | Syntax is dimly visible while caret is inside formatted construct; visually hidden when caret leaves. | P1 | Not started |
| NOTE-07 | Markdown support | Headings, emphasis, strike, code, lists, tasks, tables, quotes, rules, links, images. | P1 | Not started |
| NOTE-08 | Syntax assistance | Helpful, non-destructive syntax completion/continuation. | P2 | Not started |
| NOTE-09 | Formatting bar | Bottom syntax/action bar scrolls horizontally. | P1 | Not started |
| NOTE-10 | Insert at caret | Toolbar actions insert/wrap at caret and restore editor focus. | P1 | Not started |
| NOTE-11 | Per-note toolbar order | Toolbar action order can be changed and saved per note. | P2 | Not started |
| NOTE-12 | Note management | Create/open/rename/search/delete local Markdown notes. | P1 | Not started |
| NOTE-13 | Autosave/recovery | Atomic/safe saves and recovery protect against data loss. | P0 | Not started |
| NOTE-14 | Adaptive size/scroll | Editor adapts to content; long notes scroll without clipping. | P1 | Not started |

## G. Settings and appearance

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| SET-01 | Settings GUI | Dedicated graphical settings interface. | P1 | Not started |
| SET-02 | Canonical settings model | GUI and human-editable config share one source of truth. | P0 | In progress |
| SET-03 | Config validation | Invalid edits are caught; safe defaults/recovery available. | P1 | Not started |
| SET-04 | Settings search | Palette can search settings and open relevant settings section. | P1 | Not started |
| SET-05 | Preference coverage | Hotkey, startup, cursor, suggestions, compact mode, exclusions, bangs, bookmarks, paths, commands, notes, index, animation. | P1 | Not started |

## H. Quality, platform and delivery

| ID | Feature | Acceptance criteria | Priority | Status |
|---|---|---|---|---|
| QUAL-01 | Windows 10/11 | Exact minimum supported versions documented and tested in CI where practical. | P0 | Not started |
| QUAL-02 | No .NET prerequisite | Clean target Windows environment does not require user-installed .NET. | P0 | In progress |
| QUAL-03 | C++ or Rust | Compare both and document selected language. | P0 | Implemented |
| QUAL-04 | Lightweight | Startup, idle RAM/CPU, search latency and package size measured. | P0 | Not started |
| QUAL-05 | Offline local core | Local functionality works offline; no automatic network calls. | P0 | Not started |
| QUAL-06 | Safe transparent behavior | Least privilege, reproducible build, no evasion/telemetry. | P0 | Not started |
| QUAL-07 | GitHub Actions | CI builds, tests and uploads Windows artifact. | P0 | In progress |
| QUAL-08 | Modular structure | Independent feature UI, logic, interface, tests and docs. | P0 | In progress |
| QUAL-09 | Tiny entry point | Entry point delegates to bootstrap only. | P0 | In progress |

## Progress rules

- `Implemented` means code exists and is integrated.
- `Verified` means relevant tests passed or a documented manual check was completed.
- Never mark anything Verified based only on code review or an AI-generated claim.
- Update this catalogue whenever a feature changes state.

## Status notes (foundation milestone)
- Nothing is Verified: no CI run has happened yet and the code has not been compiled.
- QUAL-03 is "Implemented" only in the sense that the decision is documented (DEC-001).
- CORE-01..04 are partial: hotkey, topmost popup, Esc/focus-loss hide, primary-monitor work-area placement. Missing: multi-monitor/cursor placement, hide on launch/command, conflict reporting beyond an error dialog.
