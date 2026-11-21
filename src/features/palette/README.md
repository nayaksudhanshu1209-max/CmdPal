# palette

**Does:** global hotkey, borderless always-on-top popup placed inside the primary work area,
hides on Esc / focus loss. **Does not (yet):** text input, results, multi-monitor/cursor placement,
animation, settings UI. The window currently shows placeholder text.

| File | Responsibility |
|---|---|
| `api.rs` | Public interface: `create(&Settings) -> Box<dyn Feature>` (read this first) |
| `logic/coordinator.rs` | Feature entry: validates hotkey, creates window, registers hotkey |
| `logic/visibility.rs` | Pure show/hide state machine (unit-tested) |
| `logic/placement.rs` | Pure work-area placement maths (unit-tested) |
| `ui/window.rs` | Win32 window + message handling; forwards events to `Visibility` |

Depends on: `core::contracts`, `core::settings`, `platform::windows::{hotkey, shell}`.
Settings owned: `[palette] hotkey` (currently modelled in `core/settings.rs`; becomes a feature-contributed section when more features need settings).
Tests: `cargo test` (inline unit tests; window code needs manual/smoke checks).
Register: one line in `src/bootstrap/feature_registry.rs`. Remove: delete that line, this folder, and `pub mod palette;` in `features/mod.rs`.
