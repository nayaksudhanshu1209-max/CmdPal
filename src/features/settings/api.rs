use super::logic::coordinator::SettingsFeature;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(SettingsFeature)
}
