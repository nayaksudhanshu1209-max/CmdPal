# CmdPal — Features Explicitly Removed / Never Implement

This document is a hard scope boundary. Do not build these features now, later, behind a setting, as an optional module, as a plugin, or under a different name.

## 1. Numeric result shortcuts

Do not implement:
- Alt+1 through Alt+9 to launch numbered results.
- Numeric hints displayed beside results.

Keep the requested navigation keys: Up, Down, Tab, Shift+Tab, Page Up, Page Down, and Enter.

## 2. Windows switcher

Do not implement a Windows window/task switcher or a feature that lists and switches among open windows.

## 3. Converters and text utilities

Do not implement:
- Unit conversion.
- Number-base conversion (binary/decimal/hexadecimal conversion tools).
- Colour converter.
- General-purpose text tools or a text-transformation toolbox.

Do not recreate these under renamed categories.

## 4. Embedded browser — completely excluded

Do not implement any in-app browser or embedded web renderer, including:
- WebView2 or another embedded WebView.
- Chromium, CEF, Electron, browser shell, or browser runtime.
- Embedded page preview, browser window, browser tabs, browser chrome, browser profile, browser history, browser process manager, browser extension support, or in-app ad blocker.
- Loading a webpage inside the palette or a separate CmdPal-owned window.

Web bangs and URL detection remain in scope only as URL/search generation plus an explicit handoff to the Windows default browser. The external browser is owned by Windows/user settings, not CmdPal.

## 5. Music player

Do not invent a music player, song search, song streaming, or music playback feature. The earlier wording was ambiguous and does not provide a clear requirement.

## 6. Unapproved scope expansion

The user previously asked for "more useful features" in general. That is not approval to add arbitrary utilities. Keep the feature list to `APP_BRIEF.md` and `FEATURE_CATALOGUE.md`. Suggest optional additions separately and wait for explicit approval before implementing them.

## 7. Removal implementation rule

Do not create code, folders, UI, settings, commands, dependencies, tests, documentation sections, keyboard shortcuts, or CI assets for these excluded features. Do not leave placeholder files or dormant code for them.
