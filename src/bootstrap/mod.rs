//! Composition root: loads settings, registers features, runs the app, shuts down.
pub mod feature_registry;

use crate::core::settings;
use crate::platform::windows::shell;

/// Runs the application and returns the process exit code.
pub fn run() -> i32 {
    let settings = match settings::settings_path() {
        Some(path) => match settings::load_from(&path) {
            Ok(s) => s,
            Err(e) => {
                // Never overwrite a file we could not parse; fall back to defaults in memory.
                shell::show_error(
                    "CmdPal - settings problem",
                    &format!(
                        "Could not read {}:\n{e}\n\nUsing defaults for this session.",
                        path.display()
                    ),
                );
                settings::Settings::default()
            }
        },
        None => settings::Settings::default(),
    };

    let mut features = feature_registry::build(&settings);
    for i in 0..features.len() {
        if let Err(e) = features[i].start() {
            shell::show_error(
                "CmdPal - startup failed",
                &format!("Feature '{}' failed to start:\n{e}", features[i].id()),
            );
            for f in features[..i].iter_mut().rev() {
                f.stop();
            }
            return 1;
        }
    }

    let code = shell::run_message_loop();

    for f in features.iter_mut().rev() {
        f.stop();
    }
    code
}
