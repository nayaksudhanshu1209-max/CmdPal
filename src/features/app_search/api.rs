use super::logic::coordinator::AppSearch;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(AppSearch::default())
}
