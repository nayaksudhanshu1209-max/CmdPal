//! Small relevance scorer shared by all providers. Higher is better; None = no match.
pub fn score(query: &str, text: &str) -> Option<i32> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(0);
    }
    let t = text.to_lowercase();
    let mut total = 0;
    for tok in q.split_whitespace() {
        total += score_token(tok, &t)?;
    }
    Some(total)
}

fn score_token(tok: &str, t: &str) -> Option<i32> {
    if t == tok {
        return Some(1000);
    }
    if t.starts_with(tok) {
        return Some(800 - (t.len() as i32 - tok.len() as i32).min(100));
    }
    if let Some(pos) = t.find(tok) {
        let word_start = t[..pos].chars().next_back().map(|c| !c.is_alphanumeric()).unwrap_or(true);
        return Some(if word_start { 600 } else { 400 } - (pos as i32).min(100));
    }
    // subsequence
    let mut it = t.chars();
    let mut gaps = 0;
    for qc in tok.chars() {
        let mut found = false;
        for tc in it.by_ref() {
            if tc == qc {
                found = true;
                break;
            }
            gaps += 1;
        }
        if !found {
            return None;
        }
    }
    Some((200 - gaps).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_exact_prefix_word_substring_subsequence() {
        let exact = score("code", "code").unwrap();
        let prefix = score("code", "codeblocks").unwrap();
        let word = score("code", "visual code").unwrap();
        let sub = score("code", "barcode").unwrap();
        let subseq = score("vsc", "visual studio code").unwrap();
        assert!(exact > prefix && prefix > word && word > sub && sub > subseq);
    }

    #[test]
    fn no_match_and_multi_token() {
        assert!(score("xyz", "notepad").is_none());
        assert!(score("note pad", "notepad plus").is_some());
        assert!(score("note zzz", "notepad").is_none());
        assert_eq!(score("", "anything"), Some(0));
    }
}
