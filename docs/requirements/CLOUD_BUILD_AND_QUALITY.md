# CmdPal — Cloud Build, Security, Testing, and Quality Standards

## Cloud build requirement

The user does not have a local compiler/toolchain. GitHub Actions is the primary build environment. The repository must contain Windows CI workflows under `.github/workflows/`.

## CI pipeline

On a clean Windows runner:
1. Checkout repository.
2. Provision the selected C++ or Rust toolchain and required Windows SDK/build tools.
3. Restore pinned dependencies; use lockfiles and reproducible versions.
4. Run formatter/linter and static checks.
5. Run unit and contract tests.
6. Build the Windows x64 application in the selected configuration.
7. Run smoke/integration checks that are practical in CI.
8. Package output with required assets/resources.
9. Generate SHA-256 checksum and version/build metadata.
10. Upload downloadable artifact with clear naming.

Include manual workflow dispatch, pull-request validation, and push triggers appropriate to the repository. Use least-privilege workflow permissions. Pin third-party actions to reviewed versions or commit SHAs according to practical maintenance policy. Do not expose secrets in logs.

## Artifacts

State clearly whether the artifact is:
- portable folder/ZIP;
- installer;
- debug or release;
- signed or unsigned;
- self-contained or dependent on separately installed components.

A GitHub Actions artifact is not automatically a signed installer. Do not imply otherwise. Include a README with run instructions and known prerequisites. Signing may be configured only when a legitimate certificate and secret handling are available.

## Performance measurement

Measure or report methodology for:
- cold launch / first palette appearance;
- warm activation;
- idle memory and CPU after startup settles;
- memory and CPU during file indexing;
- query-to-result latency;
- launching apps/files;
- note open/save responsiveness;
- package size and startup item behavior.

Use repeatable scenarios. Separate measurement data from estimates. Do not claim tests on the user's exact 4 GB / i3 / HDD PC unless it was actually tested on equivalent hardware. Avoid setting arbitrary absolute targets before measuring a baseline; define realistic budgets and iterate.

## Security and privacy

- Least privilege; no administrator requirement by default.
- No telemetry, analytics, ad calls, tracking, hidden update checks, or background network requests.
- No embedded web engine.
- External browser opening only on explicit user action.
- Validate URLs; HTTP/HTTPS only.
- Safe argument handling for user-defined commands.
- Avoid unsafe shell concatenation and injection.
- Clear confirmation for potentially destructive actions.
- Safe local settings/note writes and recovery.
- No process injection, evasion, anti-debugging, antivirus disabling, packers, or suspicious obfuscation.
- Do not claim guaranteed antivirus whitelist status. Maintain a software bill of materials/dependency inventory where practical and use reproducible transparent builds.

## Quality gates

A milestone is ready only when:
- code compiles on CI;
- relevant tests pass;
- no placeholder/stub is falsely marked complete;
- documentation matches implementation;
- no unintended feature coupling was added;
- keyboard and mouse interactions are considered;
- errors and cancellation are handled;
- resource usage is reasonable for the workload.

## Release checklist

- [ ] Clean Windows build succeeds.
- [ ] Unit/contract/integration tests pass as applicable.
- [ ] Excluded features are absent.
- [ ] Offline local workflows tested.
- [ ] No automatic network activity confirmed by code review/testing.
- [ ] Palette startup and activation measured.
- [ ] File indexing is optional-root-only and background/incremental.
- [ ] Notes autosave/recovery tested.
- [ ] Windows 10/11 compatibility scope documented.
- [ ] App icon/resources packaged.
- [ ] SHA-256 checksum included.
- [ ] Artifact type and prerequisites documented.
- [ ] Release notes and feature catalogue updated.
