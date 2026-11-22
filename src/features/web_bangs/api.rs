use super::logic::coordinator::WebBangs;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(WebBangs)
}
