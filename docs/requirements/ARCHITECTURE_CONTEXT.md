# CmdPal — Architecture Context

## Architecture objective

Build a small, modular Windows application whose core host connects independent features. A feature should be understandable by reading its own module folder and README. Adding/removing a feature should require one obvious composition/registration change in the normal case, not edits throughout unrelated code.

## Layer model

1. **Entry point:** minimal process startup; calls bootstrap and exits with its result.
2. **Bootstrap/composition root:** creates shared services, loads validated settings, registers feature modules, connects public contracts, and controls lifecycle.
3. **Shared core:** small stable contracts and shared primitives only (logging, settings interfaces, event bus if justified, result types, cancellation, clock, platform abstraction). Do not put feature-specific behavior here.
4. **Platform adapters:** Windows-specific hotkey, app discovery, shell launch, file-system notifications, startup registration, monitor/DPI, and window behavior.
5. **Feature modules:** palette, suggestions, app search, commands, web bangs, file search, quick folders, notes, settings. Each owns UI and logic.
6. **Presentation/design system:** genuinely shared visual primitives; feature-specific views remain inside feature folders.

## Dependency direction

- Entry point depends on bootstrap only.
- Bootstrap composes public module interfaces.
- Feature modules depend on shared contracts and required platform interfaces.
- A feature must not import another feature's private implementation.
- Cross-feature communication uses public contracts, application-level services, or typed messages/events where justified.
- Avoid circular dependencies, service-locator sprawl, and global mutable state.

## Feature module contract

Each module should expose a small public interface such as `IFeature`, `FeatureDescriptor`, or an equivalent idiom suitable for the selected language. The public contract should describe lifecycle, activation, commands/events, and settings contributions only where needed. Keep the contract minimal. Where practical, provide one obvious public interface entry file and one obvious feature-logic/coordinator entry file so the user has two clear places to begin reading. Split supporting implementation and UI components into additional focused files. Never make one giant UI or logic file.

Each module folder may contain:
- `README.md`
- `public/` or `api/` interface definitions
- `logic/` domain/application behavior
- `ui/` views, controls, view models, styles, interaction state
- `platform/` feature-specific OS adapters
- `data/` persistence schemas/mappers when required
- `tests/`
- `assets/`

Do not create empty folders merely to satisfy a template. Separate files by responsibility, not by individual trivial function.

## UI/logic separation

UI receives state and user actions. Logic decides behavior and returns explicit state/results. UI event handlers must not contain search algorithms, filesystem traversal, Markdown parsing, command execution policy, persistence transactions, or Windows API details.

For notes, the Markdown editor UI is a feature-owned component and the Markdown editing/rendering model is independent logic. Do not embed the whole application UI in one file.

## Main executable

The executable entry file should be extremely small, typically equivalent to:
- initialise minimal process/runtime settings;
- call `bootstrap::run()`;
- return its exit code.

No feature registration lists, app search, UI construction, file operations, or feature implementations belong in main.

## Add/remove workflow

Adding a feature:
1. Create a feature module folder from `FEATURE_MODULE_TEMPLATE.md`.
2. Define its public interface.
3. Implement its UI and logic separately.
4. Add focused tests and README.
5. Register it at the single composition point or through a simple static manifest.
6. Verify build and tests.

Removing a feature:
1. Remove the feature registration.
2. Remove the module folder and feature-only dependency/assets/config schema.
3. Remove only necessary public wiring.
4. Update docs/catalogue.
5. Build and run tests to prove unrelated modules still work.

Do not create a dynamic plugin loader unless the real product needs runtime third-party plugins. Static registration is preferred for a lightweight desktop app.

## Shared settings

Use one canonical settings model with schema versioning and validation. The GUI and human-editable config must not become separate competing sources of truth. Feature modules may contribute typed setting sections through a contract, but central persistence and migration remain controlled.

## Data and background work

- Launch the palette before optional tasks.
- File indexing must be incremental, cancellable, low priority, and outside the startup critical path.
- Use bounded caches; avoid caching large assets indefinitely.
- Keep user note data local and crash-safe.
- Respect cancellation and application shutdown.

## Maintainability

Every feature README explains purpose, files, public contract, dependencies, settings/data, tests, registration, and removal. Update docs when code changes. Avoid architecture astronautics and excessive one-function files.
