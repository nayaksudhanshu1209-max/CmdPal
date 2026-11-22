//! Alias parsing: "g rust" -> (Google chip, "rust").
use crate::core::settings::BangAlias;

/// Returns (index of matching alias, rest of the text). Longest alias wins.
pub fn match_alias<'a>(bangs: &'a [BangAlias], text: &str) -> Option<(&'a BangAlias, String)> {
    let mut best: Option<(&BangAlias, usize)> = None;
    for b in bangs.iter().filter(|b| b.enabled && !b.alias.is_empty()) {
        let n = b.alias.chars().count();
        let head: String = text.chars().take(n).collect();
        let after = text.chars().nth(n);
        if head.eq_ignore_ascii_case(&b.alias) && after == Some(' ') && best.map(|(_, l)| n > l).unwrap_or(true) {
            best = Some((b, n));
        }
    }
    best.map(|(b, n)| (b, text.chars().skip(n + 1).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::settings::Settings;

    #[test]
    fn matches_alias_and_preserves_rest() {
        let s = Settings::default();
        let (b, rest) = match_alias(&s.bangs, "g rust lang").unwrap();
        assert_eq!((b.name.as_str(), rest.as_str()), ("Google", "rust lang"));
        let (b, rest) = match_alias(&s.bangs, "YT ").unwrap();
        assert_eq!((b.name.as_str(), rest.as_str()), ("YouTube", ""));
    }

    #[test]
    fn needs_space_and_enabled() {
        let mut s = Settings::default();
        assert!(match_alias(&s.bangs, "g").is_none());
        assert!(match_alias(&s.bangs, "google").is_none());
        s.bangs[0].enabled = false;
        assert!(match_alias(&s.bangs, "g x").is_none());
    }

    #[test]
    fn longest_alias_wins() {
        let mut s = Settings::default();
        s.bangs.push(BangAlias { alias: "g g".into(), name: "GG".into(), url: "https://x.com/?q={query}".into(), enabled: true });
        let (b, _) = match_alias(&s.bangs, "g g hello").unwrap();
        assert_eq!(b.name, "GG");
    }
}
