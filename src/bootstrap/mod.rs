//! Composition root: loads settings, registers features, runs the app, shuts down.
pub mod feature_registry;
mod handlers;

use crate::core::contracts::{FeatureContext, Registry};
use crate::core::{logging, settings};
use crate::platform::windows::{proc, shell};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn load_settings(interactive: bool) -> settings::Settings {
    let Some(path) = settings::settings_path() else { return settings::Settings::default() };
    match settings::load_from(&path) {
        Ok(s) => s,
        Err(e) => {
            logging::log(&format!("settings not loaded: {}", e.lines().next().unwrap_or("")));
            if interactive
                && shell::confirm(
                    ::windows::Win32::Foundation::HWND::default(),
                    "CmdPal - settings problem",
                    &format!("settings.toml has a problem:\n\n{e}\n\nReset to defaults? (Yes keeps the broken file as settings.toml.bad; No uses defaults for this session only and leaves the file untouched.)"),
                )
            {
                let _ = settings::quarantine(&path);
            }
            settings::Settings::default()
        }
    }
}

/// Runs the application and returns the process exit code.
pub fn run() -> i32 {
    let started = Instant::now();
    let smoke = std::env::args().any(|a| a == "--smoke");
    shell::enable_dpi_awareness();

    let ctx = FeatureContext { settings: Rc::new(RefCell::new(load_settings(!smoke))), registry: Rc::new(Registry::default()) };
    ctx.registry.add_handler(Rc::new(handlers::ShellHandler { settings: ctx.settings.clone(), registry: ctx.registry.clone() }));

    let mut features = feature_registry::build();
    for i in 0..features.len() {
        if let Err(e) = features[i].start(&ctx) {
            logging::log(&format!("feature {} failed: {e}", features[i].id()));
            shell::show_error("CmdPal - startup failed", &format!("Feature '{}' failed to start:\n{e}", features[i].id()));
            for f in features[..i].iter_mut().rev() {
                f.stop();
            }
            return 1;
        }
    }

    if smoke {
        let tid = shell::current_thread_id();
        let _ = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1500));
            shell::post_quit_to_thread(tid);
        });
    }
    let startup_ms = started.elapsed().as_millis();
    let code = shell::run_message_loop();
    if smoke {
        let report = format!("startup_ms={startup_ms}\npeak_working_set_kb={}\n", proc::peak_working_set_kb());
        let _ = std::fs::write("smoke-result.txt", report);
    }
    for f in features.iter_mut().rev() {
        f.stop();
    }
    code
}
