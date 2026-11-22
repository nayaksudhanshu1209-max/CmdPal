use crate::core::contracts::{Action, Category, Feature, FeatureContext, FeatureResult, Query, ResultItem, ResultProvider, SharedSettings};
use crate::core::fuzzy;
use std::rc::Rc;

pub struct QuickFolders;

struct Provider {
    settings: SharedSettings,
}

impl ResultProvider for Provider {
    fn id(&self) -> &'static str {
        "quick_folders"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        if q.bang.is_some() {
            return vec![];
        }
        let s = self.settings.borrow();
        let empty = q.text.trim().is_empty();
        let mut out = vec![];
        for (i, f) in s.quick_folders.iter().filter(|f| f.enabled).enumerate() {
            let score = if empty { Some(200 - i as i32) } else { fuzzy::score(&q.text, &f.name).map(|v| v + 40) };
            if let Some(sc) = score {
                let mut it = ResultItem::new(format!("qf:{}", f.path.to_lowercase()), f.name.clone(), Category::QuickFolders, sc, Action::OpenPath(f.path.clone()));
                it.subtitle = f.path.clone();
                it.icon_path = Some(f.path.clone());
                it.hideable = true;
                out.push(it);
            }
        }
        out
    }
}

impl Feature for QuickFolders {
    fn id(&self) -> &'static str {
        "quick_folders"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        ctx.registry.add_provider(Rc::new(Provider { settings: ctx.settings.clone() }));
        Ok(())
    }

    fn stop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::settings::{QuickFolder, Settings};
    use std::cell::RefCell;

    #[test]
    fn disabled_hidden_and_order_kept() {
        let mut s = Settings::default();
        s.quick_folders = vec![
            QuickFolder { name: "A".into(), path: "C:\\a".into(), enabled: true },
            QuickFolder { name: "B".into(), path: "C:\\b".into(), enabled: false },
            QuickFolder { name: "C".into(), path: "C:\\c".into(), enabled: true },
        ];
        let p = Provider { settings: Rc::new(RefCell::new(s)) };
        let r = p.query(&Query::default());
        assert_eq!(r.len(), 2);
        assert!(r[0].score > r[1].score);
    }
}
