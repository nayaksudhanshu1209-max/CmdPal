//! Pure ranking/grouping: categories ordered by their best hit, items by score.
use super::super::api::Row;
use crate::core::contracts::{Category, ResultItem};
use std::collections::HashMap;

const MAX_TOTAL: usize = 40;

fn per_category_limit(c: Category) -> usize {
    match c {
        Category::Files => 8,
        Category::Commands | Category::Settings => 5,
        _ => 6,
    }
}

pub fn arrange(items: Vec<ResultItem>, excluded: &[String]) -> Vec<Row> {
    let mut groups: HashMap<Category, Vec<ResultItem>> = HashMap::new();
    for it in items {
        if excluded.iter().any(|k| k == &it.key) {
            continue;
        }
        groups.entry(it.category).or_default().push(it);
    }
    let mut order: Vec<(Category, i32)> = groups
        .iter_mut()
        .map(|(c, v)| {
            v.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
            v.dedup_by(|a, b| a.key == b.key);
            v.truncate(per_category_limit(*c));
            (*c, v.first().map(|i| i.score).unwrap_or(0))
        })
        .collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut rows = vec![];
    let mut count = 0;
    for (c, _) in order {
        let v = groups.remove(&c).unwrap_or_default();
        if v.is_empty() || count >= MAX_TOTAL {
            continue;
        }
        rows.push(Row::Header(c));
        for it in v {
            if count >= MAX_TOTAL {
                break;
            }
            count += 1;
            rows.push(Row::Item(it));
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::contracts::Action;

    fn it(key: &str, cat: Category, score: i32) -> ResultItem {
        ResultItem::new(key, key, cat, score, Action::OpenPath(key.into()))
    }

    #[test]
    fn groups_by_best_hit_and_sorts_within() {
        let rows = arrange(
            vec![it("f1", Category::Files, 100), it("a1", Category::Applications, 900), it("a2", Category::Applications, 950), it("f2", Category::Files, 300)],
            &[],
        );
        assert_eq!(rows[0], Row::Header(Category::Applications));
        assert!(matches!(&rows[1], Row::Item(i) if i.key == "a2"));
        assert_eq!(rows[3], Row::Header(Category::Files));
        assert!(matches!(&rows[4], Row::Item(i) if i.key == "f2"));
    }

    #[test]
    fn excluded_and_duplicates_removed() {
        let rows = arrange(vec![it("x", Category::Files, 1), it("x", Category::Files, 1), it("y", Category::Files, 2)], &["y".to_string()]);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn caps_total() {
        let items: Vec<_> = (0..200).map(|i| it(&format!("k{i}"), Category::Bookmarks, i)).collect();
        assert!(arrange(items, &[]).len() <= 8);
    }
}
