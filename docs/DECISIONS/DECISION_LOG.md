# CmdPal Decision Log

### DEC-001 - Implementation language: Rust
- **Date:** 2026-11-21 - **Status:** Accepted (unverified until first CI build and measurements)
- **Context:** Native Windows; low startup/idle RAM/CPU/package size on ~4 GB / i3 / HDD PCs; no .NET; user cannot compile locally.
- **Options:** (A) C++ (Win32/Direct2D, CMake/vcpkg). (B) Rust (`windows` crate, Cargo).
- **Decision:** Rust.
- **Reasoning:** Both give small native executables with comparable startup. Rust wins on memory safety in code that parses untrusted input (URLs, Markdown, paths, config); Cargo + `Cargo.lock` gives pinned, reproducible dependencies with much less CI setup than CMake/vcpkg; static CRT is one flag; unit tests are built in. C++ wins on mature UI samples/tooling and no `unsafe` noise around Win32.
- **Trade-offs:** Win32 calls need `unsafe`; a rich Markdown editing surface must be built on DirectWrite by hand in either language.
- **Verification:** measure binary size, cold/warm start and idle RAM from the CI artifact (docs/PERFORMANCE.md once measurements exist).

### DEC-002 - UI toolkit: raw Win32 + GDI now; Direct2D/DirectWrite for custom surfaces later
- **Status:** Accepted for foundation; Proposed for later phases.
- **Reasoning:** No XAML/.NET/WebView; smallest footprint. Palette list and Notes editor are custom-drawn and will need DirectWrite. The Settings GUI (Phase 8) is expected to use native common controls; re-decide then.
- **Risk:** custom text editing (caret, IME, accessibility) is the largest technical risk; revisit in Phase 7.

### DEC-003 - Platform scope
- **Status:** Proposed. x64 only. Target Windows 10 1809+ and Windows 11 (Rust's MSVC target requires Windows 10+; per-monitor-v2 DPI needs 1703+). CI exercises only `windows-2022`, so other versions are unverified until manually tested.

### DEC-004 - Feature registration: static
- **Status:** Accepted. `src/bootstrap/feature_registry.rs`; `Feature` trait in `core/contracts.rs`. No dynamic plugin loader.

### DEC-005 - Settings: single TOML file with schema_version
- **Status:** Accepted. `%APPDATA%\CmdPal\settings.toml`; GUI and file share `core::settings::Settings`. An unparseable file is never overwritten (defaults used in memory, user told). Atomic save (temp + rename). Migrations key off `schema_version`.

### DEC-006 - File index storage and incremental updates
- **Status:** Open - decide at the start of Phase 6 (compact custom index + ReadDirectoryChangesW vs. embedded SQLite; weigh size and RAM).

### DEC-007 - Markdown single-surface editing
- **Status:** Open - decide at the start of Phase 7 (DirectWrite-rendered editor over a piece-table/rope buffer; parser choice).

### DEC-008 - Packaging: portable ZIP first
- **Status:** Accepted. Unsigned portable ZIP + SHA-256. Installer deferred.

### DEC-009 - CI: `windows-2022`, Rust 1.82.0 pinned via `rust-toolchain.toml`
- **Status:** Accepted. Actions pinned to major tags for now; pin to commit SHAs before first release. `windows` crate pinned to `=0.58.0`. Commit `Cargo.lock` after the first CI run.
