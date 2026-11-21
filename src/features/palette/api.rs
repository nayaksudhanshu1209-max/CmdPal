//! Public interface of the palette feature.
use super::logic::coordinator::PaletteCoordinator;
use crate::core::contracts::Feature;
use crate::core::settings::Settings;

pub fn create(settings: &Settings) -> Box<dyn Feature> {
    Box::new(PaletteCoordinator::new(settings.palette.hotkey.clone()))
}
