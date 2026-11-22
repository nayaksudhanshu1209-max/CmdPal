//! Public interface: feature factory plus index controls/status used by the settings UI.
use super::indexing::control;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(super::logic::coordinator::FileSearch::default())
}

/// Human-readable index status (state, entry count, last scan, errors).
pub fn status_text() -> String {
    control::status_text()
}

pub fn rebuild() {
    control::rebuild();
}

pub fn toggle_pause() -> bool {
    control::toggle_pause()
}

pub fn clear() {
    control::clear();
}
