//! Minimal stable contract every feature module implements.
pub type FeatureResult = Result<(), String>;

pub trait Feature {
    /// Stable identifier, e.g. "palette".
    fn id(&self) -> &'static str;
    /// Called once on the UI thread before the message loop starts.
    fn start(&mut self) -> FeatureResult;
    /// Called once after the message loop ends. Must be safe if `start` never ran.
    fn stop(&mut self);
}
