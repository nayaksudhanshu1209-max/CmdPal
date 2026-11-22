# CmdPal Decision Log

### DEC-001 - Implementation language: Rust
- **Date:** 2026-11-21 - **Status:** Accepted (unverified until first CI build and measurements)
- **Context:** Native Windows; low startup/idle RAM/CPU/package size on ~4 GB / i3 / HDD PCs; no .NET; user cannot compile locally.
- **Options:** (A) C++ (Win32/Direct2D, CMake/vcpkg). (B) Rust (`windows` crate, Cargo).
- **Decision:** Rust.
- **Reasoning:** Both give small native executables with comparable startup. Rust wins on memory safety in code that parses untrusted input (URLs, Markdown, paths, config); Cargo + `Cargo.lock` gives pinned, reproducible dependencies with much less CI setup than CMake/vcpkg; static CRT is one flag; unit tests are built in. C++ wins on mature UI samples/tooling and no `unsafe` noise around Win32.
- **Trade-offs:** Win32 calls need `unsafe`; a rich Markdown editing surface must be built on DirectWrite by hand in either language.
- **Verification:** measure binary size, cold/warm start and idle RAM from the CI artifact (docs/PERFORMANCE.md once measurements exist).

### DEC-002 - UI toolkit: raw Win32 (GDI drawing + native controls)
- **Status:** Accepted.
- **Reasoning:** No XAML/.NET/WebView; smallest footprint. Palette: native EDIT control for the query (caret/IME/long input handled by Windows) + GDI-drawn result list. Settings: native controls (BUTTON/EDIT/LISTBOX). Notes: see DEC-007. Direct2D/DirectWrite was dropped for now to limit API surface; revisit only if GDI text quality/perf is inadequate.
- **Risk:** GDI text is less crisp than DirectWrite on some displays; HiDPI handled via per-monitor-v2 and scaled fonts.

### DEC-003 - Platform scope
- **Status:** Proposed. x64 only. Target Windows 10 1809+ and Windows 11 (Rust's MSVC target requires Windows 10+; per-monitor-v2 DPI needs 1703+). CI exercises only `windows-2022`, so other versions are unverified until manually tested.

### DEC-004 - Feature registration: static
- **Status:** Accepted. `src/bootstrap/feature_registry.rs`; `Feature` trait in `core/contracts.rs`. No dynamic plugin loader.

### DEC-005 - Settings: single TOML file with schema_version
- **Status:** Accepted. `%APPDATA%\CmdPal\settings.toml`; GUI and file share `core::settings::Settings`. An unparseable file is never overwritten (defaults used in memory, user told). Atomic save (temp + rename). Migrations key off `schema_version`.

### DEC-006 - File index storage and incremental updates
- **Status:** Accepted. Per-directory records (name lists + directory mtime) held in memory and persisted as TSV (`%LOCALAPPDATA%\CmdPal\file-index.tsv`). Rescans re-read only directories whose mtime changed; no SQLite dependency, no live watcher (ReadDirectoryChangesW) to keep idle cost near zero. Trade-off: still stats every directory per scan, and file-content edits that do not change a directory's mtime are only seen by the optional content index's per-file mtime check.

### DEC-007 - Markdown single-surface editing
- **Status:** Accepted. Windows RichEdit 5.0 (system `Msftedit.dll`, not a browser/runtime) as the single editable surface; a pure Markdown analyser (`markdown_model`) produces spans and the UI applies character formats, using the RichEdit *hidden text* effect for delimiters outside the caret's construct and dim styling inside it. Avoids writing a text engine (caret, IME, undo, selection). Risks: formatting pass is O(note) per change (debounced 90 ms, disabled above ~200k characters); hidden-delimiter caret movement behaviour must be verified manually on Windows 10 and 11.

### DEC-008 - Packaging: portable ZIP first
- **Status:** Accepted. Unsigned portable ZIP + SHA-256. Installer deferred.

### DEC-009 - CI: `windows-2022`, Rust 1.88.0 pinned via `rust-toolchain.toml`
- **Status:** Accepted. Fmt/clippy are non-blocking until the first clean run. Actions pinned to major tags for now; pin to commit SHAs before first release. `windows` crate pinned to `=0.58.0`. Commit `Cargo.lock` after the first CI run.
