# commands
**Does:** built-in commands (settings, reload, refresh apps, rebuild index, pause index, quit) and user-defined commands (`[[commands]]` in settings; editable in Settings > Commands). Running requires Enter/click; commands with `confirm = true` (default) show the exact program + arguments first.
**Safety:** the program and each argument are passed separately to `std::process::Command` (no shell, no concatenated command line); `{input}` only fills inside one argument. Typing `<command name> <text>` supplies the input.
**Does not:** ship example destructive commands, run anything automatically.
Files: `api.rs`, `logic/coordinator.rs` (entry + handler), `provider.rs`, `exec.rs` (tested).
