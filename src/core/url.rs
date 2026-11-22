//! Pure URL detection/validation/encoding. Only http/https are ever accepted (WEB-07, WEB-08).

struct Parsed<'a> {
    host: &'a str,
    explicit_scheme: bool,
}

fn strip_scheme(s: &str) -> Option<(&str, bool)> {
    let l = s.to_ascii_lowercase();
    if l.starts_with("https://") {
        Some((&s[8..], true))
    } else if l.starts_with("http://") {
        Some((&s[7..], true))
    } else {
        None
    }
}

fn parse(s: &str, explicit: bool) -> Option<Parsed<'_>> {
    // `s` = everything after the scheme
    if s.chars().any(|c| c.is_whitespace() || c.is_control() || c == '\\') {
        return None;
    }
    let end = s.find(['/', '?', '#']).unwrap_or(s.len());
    let authority = &s[..end];
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let host = if let Some(rest) = authority.strip_prefix('[') {
        let close = rest.find(']')?;
        let (h, after) = (&rest[..close], &rest[close + 1..]);
        if h.is_empty() || !h.chars().all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.') {
            return None;
        }
        if !after.is_empty() {
            valid_port(after.strip_prefix(':')?)?;
        }
        return Some(Parsed { host: h, explicit_scheme: explicit });
    } else if let Some((h, port)) = authority.rsplit_once(':') {
        valid_port(port)?;
        h
    } else {
        authority
    };
    valid_host(host, explicit)?;
    Some(Parsed { host, explicit_scheme: explicit })
}

fn valid_port(p: &str) -> Option<()> {
    if p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u32 = p.parse().ok()?;
    if (1..=65535).contains(&n) { Some(()) } else { None }
}

fn is_ipv4(h: &str) -> bool {
    let parts: Vec<&str> = h.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| !p.is_empty() && p.len() <= 3 && p.parse::<u16>().map(|n| n <= 255).unwrap_or(false))
}

fn valid_host(host: &str, explicit: bool) -> Option<()> {
    if host.is_empty() || host.len() > 253 {
        return None;
    }
    let labels: Vec<&str> = host.split('.').collect();
    for l in &labels {
        if l.is_empty()
            || l.len() > 63
            || l.starts_with('-')
            || l.ends_with('-')
            || !l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return None;
        }
    }
    if explicit || host.eq_ignore_ascii_case("localhost") || is_ipv4(host) {
        return Some(());
    }
    // schemeless: need a dot and an alphabetic TLD of 2+ letters
    let tld = labels.last()?;
    if labels.len() >= 2 && tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()) {
        Some(())
    } else {
        None
    }
}

/// True for a complete, valid http(s) URL.
pub fn is_valid_http_url(s: &str) -> bool {
    match strip_scheme(s.trim()) {
        Some((rest, _)) => parse(rest, true).is_some(),
        None => false,
    }
}

/// Detects a URL in typed text. Returns the URL to open (https:// added if no scheme).
pub fn detect(typed: &str) -> Option<String> {
    let t = typed.trim();
    if t.is_empty() {
        return None;
    }
    if let Some((rest, _)) = strip_scheme(t) {
        return parse(rest, true).map(|_| t.to_string());
    }
    let p = parse(t, false)?;
    let _ = (p.host, p.explicit_scheme);
    Some(format!("https://{t}"))
}

/// Percent-encodes a query component (UTF-8; unreserved characters kept).
pub fn encode_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Fills `{query}` in a provider template. The placeholder must not be in the authority.
pub fn build_search_url(template: &str, query: &str) -> Option<String> {
    if !template.contains("{query}") {
        return None;
    }
    let (rest, _) = strip_scheme(template.trim())?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if rest[..end].contains('{') {
        return None;
    }
    let url = template.trim().replace("{query}", &encode_component(query));
    if is_valid_http_url(&url) { Some(url) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_deep_urls() {
        assert_eq!(detect("https://example.com/a/b?x=1&y=2#frag").unwrap(), "https://example.com/a/b?x=1&y=2#frag");
        assert_eq!(detect("example.org/docs/page").unwrap(), "https://example.org/docs/page");
        assert_eq!(detect("http://localhost:8080/x").unwrap(), "http://localhost:8080/x");
        assert_eq!(detect("192.168.1.1:3000").unwrap(), "https://192.168.1.1:3000");
        assert!(detect("some.obscure-site.dev/path").is_some());
    }

    #[test]
    fn rejects_unsafe_and_malformed() {
        for bad in [
            "javascript:alert(1)", "file:///c:/x", "mailto:a@b.com", "ftp://x.com", "https://", "http://a b.com",
            "https://user@evil.com", "hello", "hello world", "https://exa_mple.com", "https://x.com:99999", "a.b",
            "https://-bad.com", "data:text/html,hi",
        ] {
            assert!(detect(bad).is_none(), "{bad}");
        }
        assert!(!is_valid_http_url("javascript:alert(1)"));
        assert!(is_valid_http_url("HTTPS://Example.com"));
    }

    #[test]
    fn encodes_and_builds() {
        assert_eq!(encode_component("a b&c/ü"), "a%20b%26c%2F%C3%BC");
        assert_eq!(
            build_search_url("https://www.google.com/search?q={query}", "rust lang").unwrap(),
            "https://www.google.com/search?q=rust%20lang"
        );
        assert!(build_search_url("https://{query}.com/", "x").is_none());
        assert!(build_search_url("javascript:{query}", "x").is_none());
        assert!(build_search_url("https://a.com/", "x").is_none());
    }
}
