# Architecture (Rust)
Authoritative rules: `docs/requirements/ARCHITECTURE_CONTEXT.md` and `CMDPAL_MODULE_MAP.md`.
Flow: `src/main.rs` -> `bootstrap/` (settings, `feature_registry`, lifecycle) -> `core/` (contracts, settings) -> `platform/windows/` (hotkey, shell) -> `features/<name>/{api.rs, logic/, ui/}`.
Rust idiom: `api.rs` is the public entry file, `logic/coordinator.rs` the logic entry; unit tests live inline with the code they test.
