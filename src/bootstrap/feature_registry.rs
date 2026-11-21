//! The single feature registration point. Add or remove one line per feature.
use crate::core::contracts::Feature;
use crate::core::settings::Settings;
use crate::features::palette;

pub fn build(settings: &Settings) -> Vec<Box<dyn Feature>> {
    vec![palette::api::create(settings)]
}
