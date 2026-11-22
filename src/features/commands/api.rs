use super::logic::coordinator::Commands;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(Commands)
}
