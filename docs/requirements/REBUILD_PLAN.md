# CmdPal — Greenfield Implementation Plan

This plan describes how to create the entire new CmdPal application from an empty repository. It is not a migration, repair, or audit plan.

## Phase 0 — Establish the project

**Goal:** create a clean source repository and settle the technical foundation.

- Create a new empty GitHub repository named `CmdPal`.
- Read the product and architecture files in Project Knowledge.
- Compare C++ and Rust for the actual app constraints; choose one and document the decision.
- Choose a suitable native Windows UI toolkit without mandatory .NET or embedded browser runtime.
- Define exact Windows 10/11 support scope.
- Create root documentation, source layout, dependency policy, versioning, and feature registration approach.
- Create GitHub Actions Windows CI workflow.

**Exit criteria:** minimal project builds on a clean GitHub Windows runner and uploads a downloadable artifact.

## Phase 1 — Application foundation

**Goal:** establish a stable, minimal host.

- Tiny entry point.
- Bootstrap/composition root.
- Logging with low overhead and no sensitive content.
- Canonical settings model and safe persistence.
- Window lifecycle abstraction and Windows platform adapters.
- App icon/resource packaging.
- Crash/error handling and orderly shutdown.

**Exit criteria:** app launches, remains running, closes cleanly, and CI artifact can be downloaded and executed in a compatible Windows environment.

## Phase 2 — Main command palette

**Goal:** make the core launcher usable.

- Global hotkey registration/configuration.
- Palette window, always-on-top behavior, focus handling and auto-hide.
- Monitor work-area/DPI positioning and optional cursor positioning.
- Search input and result list.
- Keyboard/mouse navigation.
- Categorised suggestions framework and hide/exclusion mechanism.
- Preserve query/caret during asynchronous updates.

**Exit criteria:** palette opens reliably, stays responsive, navigates correctly and obeys its window/focus rules.

## Phase 3 — Application search

- Installed application discovery and caching.
- Search ranking and app icons.
- Launch by Enter/click.
- Hide applications from suggestions.

**Exit criteria:** app discovery/search/launch works without blocking activation; exclusions persist.

## Phase 4 — Commands and tasks

- Built-in CmdPal commands.
- User-defined command/task model and settings UI.
- Safe argument validation and execution policy.
- Result/error feedback.

**Exit criteria:** users can create, edit, disable, reorder, execute and delete commands safely.

## Phase 5 — Web bangs and external URLs

- Configurable aliases and labelled chips.
- Google default and YouTube provider.
- Search URL generation and correct URL encoding.
- Empty-bang bookmarks.
- Add/edit/remove/reorder bookmarks.
- Arbitrary HTTP/HTTPS URL detection including deep paths.
- Explicit handoff to Windows default browser.

**Exit criteria:** external searches and URLs open correctly only after Enter/click; CmdPal contains no web renderer or browser engine.

## Phase 6 — File search and quick folders

- Custom indexing root configuration.
- Optional removable known folders.
- Recursive, incremental, low-priority indexing limited to chosen roots.
- Name/metadata search.
- Exclusions, progress, errors, pause/rebuild/clear.
- Quick folders as a separate shortcut feature.

**Exit criteria:** selected roots are indexed without blocking startup; quick-folder and index-root settings remain independent.

## Phase 7 — Notes and Markdown editor

- Separate frameless note window.
- Focus-loss behavior, Escape/Ctrl+W close.
- Local Markdown note management.
- Single editable live-formatted surface.
- Caret-aware source delimiter visibility.
- Formatting toolbar with horizontal scrolling and per-note ordering.
- Safe autosave and recovery.

**Exit criteria:** editing, formatting, saving, reopening, keyboard/mouse use and recovery all work without losing content.

## Phase 8 — Settings and appearance completion

- Full settings GUI and settings search.
- Human-editable canonical settings file and validation.
- Compact mode, suggestions visibility, cursor placement, animation/reduced motion.
- Startup option, exclusions, paths, aliases, bookmarks, commands, notes preferences.
- UI polish, icons, accessibility and Windows conventions.

**Exit criteria:** every in-scope preference can be changed reliably and persists correctly.

## Phase 9 — Integration, performance, security, and release

- Full app integration and regression tests.
- Feature removal-boundary tests.
- Profile cold/warm startup, idle memory/CPU, query latency, indexing, notes and package size.
- Review offline behavior and absence of automatic network calls.
- Review dependencies, URL validation, command execution, local file permissions and update behavior.
- Verify Windows 10/11 scope and document unsupported versions.
- Create release workflow, versioned artifact, checksum, release notes and user instructions.

**Exit criteria:** all catalogue items are Verified, CI builds successfully, and final artifact/package is documented.

## Working rule

Build in increments, but do not confuse incremental delivery with stopping early. The overall task is to complete the app described by `APP_BRIEF.md`. Do not mark placeholder features complete. Update `FEATURE_CATALOGUE.md` at each milestone.
