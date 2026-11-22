//! Non-destructive syntax assistance and the formatting-bar action definitions.

#[derive(Debug, Clone, PartialEq)]
pub enum Continue {
    /// Nothing special: let the editor insert a normal newline.
    None,
    /// Insert a newline followed by this prefix.
    Insert(String),
    /// The line is only an empty list/quote prefix: remove that many characters (end the list).
    ClearPrefix(usize),
}

/// Decides what Enter should do on `line` (text of the current line up to the caret).
pub fn continuation(line: &str) -> Continue {
    let ws_len = line.len() - line.trim_start().len();
    let (ws, rest) = line.split_at(ws_len);
    let (prefix, next_prefix): (String, String) = if let Some(c) = ["- [ ] ", "- [x] ", "- [X] ", "* [ ] ", "* [x] "].iter().find(|p| rest.starts_with(**p)) {
        (c.to_string(), format!("{}[ ] ", &c[..2]))
    } else if let Some(c) = ["- ", "* ", "+ ", "> "].iter().find(|p| rest.starts_with(**p)) {
        (c.to_string(), c.to_string())
    } else if let Some((num, _)) = rest.split_once(". ").filter(|(n, _)| !n.is_empty() && n.len() < 9 && n.chars().all(|c| c.is_ascii_digit())) {
        let n: u64 = num.parse().unwrap_or(0);
        (format!("{num}. "), format!("{}. ", n + 1))
    } else {
        return Continue::None;
    };
    if rest[prefix.len()..].trim().is_empty() {
        Continue::ClearPrefix(ws.chars().count() + prefix.chars().count())
    } else {
        Continue::Insert(format!("{ws}{next_prefix}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// Wrap the selection (or place the caret between) with these strings.
    Wrap(&'static str, &'static str),
    /// Insert at the start of the current line.
    LinePrefix(&'static str),
    /// Insert text at the caret ("\r" = newline).
    Insert(&'static str),
}

pub struct Tool {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: Kind,
}

pub const TOOLS: &[Tool] = &[
    Tool { id: "bold", label: "B", kind: Kind::Wrap("**", "**") },
    Tool { id: "italic", label: "I", kind: Kind::Wrap("*", "*") },
    Tool { id: "strike", label: "S", kind: Kind::Wrap("~~", "~~") },
    Tool { id: "code", label: "code", kind: Kind::Wrap("`", "`") },
    Tool { id: "h1", label: "H1", kind: Kind::LinePrefix("# ") },
    Tool { id: "h2", label: "H2", kind: Kind::LinePrefix("## ") },
    Tool { id: "h3", label: "H3", kind: Kind::LinePrefix("### ") },
    Tool { id: "bullet", label: "list", kind: Kind::LinePrefix("- ") },
    Tool { id: "number", label: "1.", kind: Kind::LinePrefix("1. ") },
    Tool { id: "task", label: "task", kind: Kind::LinePrefix("- [ ] ") },
    Tool { id: "quote", label: "quote", kind: Kind::LinePrefix("> ") },
    Tool { id: "rule", label: "rule", kind: Kind::Insert("\r---\r") },
    Tool { id: "link", label: "link", kind: Kind::Wrap("[", "](https://)") },
    Tool { id: "image", label: "image", kind: Kind::Wrap("![", "](path)") },
    Tool { id: "table", label: "table", kind: Kind::Insert("| Column | Column |\r| --- | --- |\r| | |\r") },
    Tool { id: "codeblock", label: "block", kind: Kind::Insert("```\r\r```") },
];

pub fn tool(id: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|t| t.id == id)
}

pub fn default_order() -> Vec<String> {
    TOOLS.iter().map(|t| t.id.to_string()).collect()
}

/// Saved order -> valid order: unknown ids and duplicates dropped, missing tools appended.
pub fn reconcile_order(saved: &[String]) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for s in saved {
        if tool(s).is_some() && !out.contains(s) {
            out.push(s.clone());
        }
    }
    for t in TOOLS {
        if !out.iter().any(|o| o == t.id) {
            out.push(t.id.to_string());
        }
    }
    out
}

pub fn move_item(order: &mut Vec<String>, from: usize, to: usize) {
    if from < order.len() && to < order.len() && from != to {
        let it = order.remove(from);
        order.insert(to, it);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continues_lists_and_numbers() {
        assert_eq!(continuation("- item"), Continue::Insert("- ".into()));
        assert_eq!(continuation("  * item"), Continue::Insert("  * ".into()));
        assert_eq!(continuation("9. nine"), Continue::Insert("10. ".into()));
        assert_eq!(continuation("> quote"), Continue::Insert("> ".into()));
        assert_eq!(continuation("- [x] done"), Continue::Insert("- [ ] ".into()));
    }

    #[test]
    fn empty_prefix_ends_list_and_plain_text_untouched() {
        assert_eq!(continuation("- "), Continue::ClearPrefix(2));
        assert_eq!(continuation("  1. "), Continue::ClearPrefix(5));
        assert_eq!(continuation("plain"), Continue::None);
        assert_eq!(continuation("-no space"), Continue::None);
        assert_eq!(continuation("3.14"), Continue::None);
    }

    #[test]
    fn order_reconcile_and_move() {
        let o = reconcile_order(&["italic".into(), "bogus".into(), "bold".into(), "bold".into()]);
        assert_eq!(&o[..2], &["italic".to_string(), "bold".to_string()]);
        assert_eq!(o.len(), TOOLS.len());
        let mut v = default_order();
        move_item(&mut v, 0, 2);
        assert_eq!(v[2], "bold");
        move_item(&mut v, 99, 0);
        assert_eq!(v.len(), TOOLS.len());
    }
}
