# Feature Module Template

Use this template for each meaningful CmdPal feature. Do not create a module for an excluded feature. Adapt names to the chosen language without weakening the structure.

## Module metadata

- **Feature name:**
- **Purpose:**
- **User-visible behavior:**
- **Priority:**
- **Public interface:**
- **Registration point:**
- **Owned settings/data:**
- **Dependencies:**

## Required module contents

```text
feature_name/
├── README.md
├── api/          # public contract used by bootstrap/other modules
├── logic/        # domain/application rules; no UI toolkit dependencies where practical
├── ui/           # feature-owned views, controls, state/presentation
├── platform/     # only if feature requires OS-specific adapter
├── data/         # only if feature owns persistence/schema
└── tests/        # unit and contract tests
```

Only create directories that are needed. Keep assets next to the feature that owns them unless they are truly shared design-system assets.

## Clear entry files

Where practical for the selected language, provide one obvious public interface entry file and one obvious feature-logic/coordinator entry file. These should make it easy to understand what the feature exposes and where its main behavior is coordinated. Split complex internals and each meaningful UI component into separate cohesive files. Do not put the whole feature into two giant files merely to meet this convention.

## Public contract requirements

- Minimal typed interface and explicit inputs/outputs.
- Clear lifecycle and cancellation semantics.
- No leakage of private implementation types.
- No circular dependency.
- Document events/commands/settings contributed by the feature.

## UI/logic requirements

- UI renders state and forwards actions.
- Logic owns behavior and validation.
- No long-running/blocking work in UI event handlers.
- Preserve focus, selection, caret and user-entered data across asynchronous work.
- Provide keyboard and mouse access for interactive UI.

## Tests

- Unit tests for core behavior.
- Public contract tests where useful.
- Integration tests for interactions with shared services.
- Regression tests for important user-visible bugs/edge cases.
- Tests must not depend on live external network access unless the external handoff itself is being mocked.

## README content

Every feature README must explain:
1. What it does and explicitly does not do.
2. File-by-file responsibility for important files.
3. Its public interface and registration location.
4. Dependencies and communication with other modules.
5. Settings/data it owns and storage location.
6. How to build/test it.
7. How to add, disable, and remove it safely.

## Removal test

Verify that removing the registration and module does not break unrelated features. Remove feature-only dependencies, settings, tests, assets, and documentation references. Record any unavoidable shared-contract change.
