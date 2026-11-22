use super::logic::coordinator::QuickFolders;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(QuickFolders)
}
