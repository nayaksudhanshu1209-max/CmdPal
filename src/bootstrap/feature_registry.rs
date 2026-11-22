//! The single feature registration point. Add or remove one line per feature.
use crate::core::contracts::Feature;
use crate::features::{app_search, commands, file_search, notes, palette, quick_folders, settings, web_bangs};

/// Order only matters for start-up/shutdown sequencing; providers are looked up lazily.
pub fn build() -> Vec<Box<dyn Feature>> {
    vec![
        commands::api::create(),
        app_search::api::create(),
        web_bangs::api::create(),
        file_search::api::create(),
        quick_folders::api::create(),
        notes::api::create(),
        settings::api::create(),
        palette::api::create(),
    ]
}
