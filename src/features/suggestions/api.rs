//! Public interface: turn provider results into display rows.
use crate::core::contracts::{Category, Query, Registry, ResultItem};
use crate::core::settings::Settings;

#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    Header(Category),
    Item(ResultItem),
}

/// Collects results from every provider and returns categorised, ranked rows.
pub fn build_rows(registry: &Registry, settings: &Settings, q: &Query) -> Vec<Row> {
    if q.text.trim().is_empty() && q.bang.is_none() && !settings.palette.show_suggestions_at_startup {
        return vec![];
    }
    let items = registry.query_all(q);
    super::logic::engine::arrange(items, &settings.exclusions)
}
