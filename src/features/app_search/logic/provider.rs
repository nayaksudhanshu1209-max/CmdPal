use super::discovery::AppEntry;
use crate::core::contracts::{Action, Category, Query, ResultItem, ResultProvider};
use crate::core::fuzzy;
use std::sync::{Arc, RwLock};

pub struct AppProvider {
    pub apps: Arc<RwLock<Vec<AppEntry>>>,
}

impl ResultProvider for AppProvider {
    fn id(&self) -> &'static str {
        "apps"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        if q.bang.is_some() || q.text.trim().is_empty() {
            return vec![];
        }
        let Ok(apps) = self.apps.read() else { return vec![] };
        apps.iter()
            .filter_map(|a| {
                let s = fuzzy::score(&q.text, &a.name)?;
                let mut it = ResultItem::new(format!("app:{}", a.path.to_lowercase()), a.name.clone(), Category::Applications, s + 50, Action::LaunchPath(a.path.clone()));
                it.subtitle = "Application".into();
                it.icon_path = Some(a.path.clone());
                it.hideable = true;
                Some(it)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_and_ignores_bang_mode() {
        let p = AppProvider { apps: Arc::new(RwLock::new(vec![AppEntry { name: "Notepad".into(), path: "n.lnk".into() }])) };
        let q = Query { text: "note".into(), bang: None };
        assert_eq!(p.query(&q).len(), 1);
        assert!(p.query(&Query { text: "note".into(), bang: Some("web:g".into()) }).is_empty());
        assert!(p.query(&Query::default()).is_empty());
    }
}
