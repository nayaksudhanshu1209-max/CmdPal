use super::super::indexing::content::ContentIndex;
use super::super::indexing::control::Shared;
use super::super::indexing::store::ci_contains;
use crate::core::contracts::{Action, Category, Query, ResultItem, ResultProvider, SharedSettings};
use crate::core::fuzzy;
use std::sync::{Arc, RwLock};

pub struct FileProvider {
    pub shared: Arc<Shared>,
    pub content: Arc<RwLock<ContentIndex>>,
    pub settings: SharedSettings,
}

fn make_item(dir: &str, name: &str, is_dir: bool, score: i32, subtitle: &str) -> ResultItem {
    let full = format!("{}\\{}", dir.trim_end_matches('\\'), name);
    let mut it = ResultItem::new(format!("file:{}", full.to_lowercase()), name, Category::Files, score, Action::OpenPath(full.clone()));
    it.subtitle = if subtitle.is_empty() { dir.to_string() } else { format!("{subtitle} - {dir}") };
    it.icon_path = Some(full);
    it.hideable = true;
    let _ = is_dir;
    it
}

impl ResultProvider for FileProvider {
    fn id(&self) -> &'static str {
        "files"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        let text = q.text.trim();
        if q.bang.is_some() || text.chars().count() < 2 {
            return vec![];
        }
        let toks: Vec<String> = text.split_whitespace().map(|t| t.to_lowercase()).collect();
        let mut out: Vec<ResultItem> = vec![];
        if let Ok(recs) = self.shared.records.read() {
            'outer: for (dir, rec) in recs.iter() {
                for (names, is_dir) in [(&rec.files, false), (&rec.dirs, true)] {
                    for n in names {
                        if toks.iter().all(|t| ci_contains(n, t)) {
                            let s = fuzzy::score(text, n).unwrap_or(50);
                            out.push(make_item(dir, n, is_dir, s + if is_dir { 10 } else { 0 }, ""));
                            if out.len() >= 4000 {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| b.score.cmp(&a.score));
        out.truncate(30);
        if self.settings.borrow().files.content_index && text.chars().count() >= 3 {
            if let Ok(ci) = self.content.read() {
                for p in ci.search(text, 10) {
                    if let Some((dir, name)) = p.rsplit_once('\\') {
                        let mut it = make_item(dir, name, false, 30, "Content match");
                        it.key = format!("content:{}", p.to_lowercase());
                        out.push(it);
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::settings::Settings;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn finds_by_name_tokens() {
        let sh = super::super::super::indexing::control::init();
        {
            let mut r = sh.records.write().unwrap();
            r.clear();
            r.insert(
                r"C:\docs".into(),
                Arc::new(super::super::super::indexing::store::DirRecord { mtime: 1, files: vec!["Quarterly Report.xlsx".into(), "notes.txt".into()], dirs: vec!["Reports".into()] }),
            );
        }
        let p = FileProvider { shared: sh, content: Arc::new(RwLock::new(ContentIndex::default())), settings: Rc::new(RefCell::new(Settings::default())) };
        let r = p.query(&Query { text: "report".into(), bang: None });
        assert_eq!(r.len(), 2);
        assert!(p.query(&Query { text: "r".into(), bang: None }).is_empty());
        assert!(p.query(&Query { text: "report".into(), bang: Some("web:g".into()) }).is_empty());
    }
}
