# app_search
**Does:** discovers apps from Start Menu `.lnk` shortcuts (all-users + current user), caches the list in `%LOCALAPPDATA%\CmdPal\apps.cache`, rescans once per launch in a low-priority background thread (and on the "Refresh applications" command), fuzzy-searches names, launches via the Windows shell. Icons are resolved lazily by the palette while painting.
**Does not:** enumerate Microsoft Store/UWP apps that have no Start Menu shortcut (known limitation), scan per keystroke.
Files: `api.rs` (public `create()`), `logic/coordinator.rs` (entry), `discovery.rs`, `cache.rs`, `provider.rs`. Hiding an app = palette adds its key to `exclusions` in settings.
Register/remove: one line in `bootstrap/feature_registry.rs` + `pub mod app_search;` in `features/mod.rs`.
