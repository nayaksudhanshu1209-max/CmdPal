use super::bang::match_alias;
use crate::core::contracts::{Action, Category, Query, ResultItem, ResultProvider, SharedSettings};
use crate::core::{fuzzy, url};

pub struct WebProvider {
    pub settings: SharedSettings,
}

fn bookmark_item(name: &str, u: &str, score: i32) -> ResultItem {
    let mut it = ResultItem::new(format!("bm:{u}"), name, Category::Bookmarks, score, Action::OpenUrl(u.to_string()));
    it.subtitle = u.to_string();
    it.hideable = true;
    it
}

impl ResultProvider for WebProvider {
    fn id(&self) -> &'static str {
        "web"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        let s = self.settings.borrow();
        let text = q.text.trim();
        let mut out = vec![];

        if let Some(id) = &q.bang {
            let Some(alias) = id.strip_prefix("web:") else { return out };
            let Some(b) = s.bangs.iter().find(|b| b.enabled && b.alias.eq_ignore_ascii_case(alias)) else { return out };
            if text.is_empty() {
                // empty bang: show saved bookmarks in their saved order
                for (i, bm) in s.bookmarks.iter().filter(|b| b.enabled).enumerate() {
                    out.push(bookmark_item(&bm.name, &bm.url, 500 - i as i32));
                }
            } else {
                if let Some(u) = url::build_search_url(&b.url, text) {
                    let mut it = ResultItem::new(format!("web:search:{}:{text}", b.alias), format!("Search {} for \"{text}\"", b.name), Category::Web, 900, Action::OpenUrl(u.clone()));
                    it.subtitle = u;
                    out.push(it);
                }
                for bm in s.bookmarks.iter().filter(|b| b.enabled) {
                    if let Some(sc) = fuzzy::score(text, &bm.name) {
                        out.push(bookmark_item(&bm.name, &bm.url, sc));
                    }
                }
            }
            return out;
        }

        if text.is_empty() {
            for (i, bm) in s.bookmarks.iter().filter(|b| b.enabled).take(3).enumerate() {
                out.push(bookmark_item(&bm.name, &bm.url, 40 - i as i32));
            }
            return out;
        }
        if let Some(u) = url::detect(text) {
            let mut it = ResultItem::new(format!("web:url:{u}"), format!("Open {u}"), Category::Web, 990, Action::OpenUrl(u.clone()));
            it.subtitle = "Opens in your default browser".into();
            out.push(it);
        }
        for bm in s.bookmarks.iter().filter(|b| b.enabled) {
            let sc = fuzzy::score(text, &bm.name).or_else(|| fuzzy::score(text, &bm.url).map(|v| v / 2));
            if let Some(sc) = sc {
                out.push(bookmark_item(&bm.name, &bm.url, sc + 20));
            }
        }
        // default search provider fallback (low rank)
        if let Some(b) = s.bangs.iter().find(|b| b.enabled && b.alias.eq_ignore_ascii_case(&s.default_search_alias)) {
            if let Some((_, _)) = match_alias(&s.bangs, &format!("{} x", b.alias)).map(|(a, r)| (a, r)) {
                if let Some(u) = url::build_search_url(&b.url, text) {
                    let mut it = ResultItem::new(format!("web:default:{text}"), format!("Search {} for \"{text}\"", b.name), Category::Web, 5, Action::OpenUrl(u.clone()));
                    it.subtitle = u;
                    out.push(it);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::settings::{Bookmark, Settings};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn provider() -> WebProvider {
        let mut s = Settings::default();
        s.bookmarks.push(Bookmark { name: "Rust docs".into(), url: "https://doc.rust-lang.org/std/".into(), enabled: true });
        WebProvider { settings: Rc::new(RefCell::new(s)) }
    }

    #[test]
    fn empty_bang_lists_bookmarks() {
        let r = provider().query(&Query { text: "".into(), bang: Some("web:g".into()) });
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].category, Category::Bookmarks);
    }

    #[test]
    fn bang_builds_encoded_url() {
        let r = provider().query(&Query { text: "a b".into(), bang: Some("web:g".into()) });
        assert_eq!(r[0].action, Action::OpenUrl("https://www.google.com/search?q=a%20b".into()));
    }

    #[test]
    fn url_detected_and_default_search_offered() {
        let r = provider().query(&Query { text: "example.com/x/y?z=1".into(), bang: None });
        assert!(r.iter().any(|i| i.action == Action::OpenUrl("https://example.com/x/y?z=1".into())));
        let r = provider().query(&Query { text: "hello".into(), bang: None });
        assert!(r.iter().any(|i| i.key.starts_with("web:default")));
        assert!(!r.iter().any(|i| i.key.starts_with("web:url")));
    }
}
