//! Public interface of the palette feature.
use super::logic::coordinator::PaletteCoordinator;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(PaletteCoordinator::default())
}
