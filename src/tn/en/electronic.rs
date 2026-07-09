//! Electronic TN tagger.
//!
//! Converts written emails and URLs to spoken form, keeping pronounceable
//! labels as words (not letter-by-letter) and applying NeMo's casing:
//! - "a.bc@gmail.com" → "a dot bc at gmail dot com"
//! - "a@hotmail.de" → "a at hotmail dot DE" (country-code TLDs upper-cased)
//! - "http://www.example.com" → "HTTP colon slash slash WWW dot example dot com"
//!
//! Casing is driven by small curated sets rather than NeMo's full data lists,
//! so a few brand and word-segmentation cases ("rtxprohelp" → "RTX pro help")
//! are not reproduced; see the vendored `tn_electronic.txt` header.

use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    /// Protocol / subdomain keywords, always upper-cased.
    static ref PROTOCOL_UPPER: HashSet<&'static str> =
        ["http", "https", "www", "ftp"].into_iter().collect();

    /// Generic TLDs read in lower case.
    static ref GTLD: HashSet<&'static str> = [
        "com", "org", "net", "edu", "gov", "mil", "int", "info", "biz", "io",
    ]
    .into_iter()
    .collect();

    /// Two-letter country-code TLDs read in upper case.
    static ref CCTLD: HashSet<&'static str> = [
        "de", "fr", "it", "sm", "uk", "us", "ca", "au", "jp", "cn", "ru", "es",
        "nl", "se", "no", "fi", "dk", "pl", "br", "in", "mx", "kr", "ch", "at",
        "be", "pt", "gr", "ie", "cz", "hu", "ro", "tr", "za", "nz", "sg", "hk",
    ]
    .into_iter()
    .collect();

    /// Product acronyms upper-cased in email/URL context.
    static ref BRAND: HashSet<&'static str> =
        ["nvidia", "cuda", "dgx", "rtx", "basepod"].into_iter().collect();
}

/// Parse an email or URL to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // "word / word" (spaces optional) reads the slash literally. Handled first
    // so the whitespace guard below can reject spaces in the URL/email forms.
    if let Some(result) = parse_slash_words(trimmed) {
        return Some(result);
    }

    // Emails and URLs never contain whitespace.
    if trimmed.contains(char::is_whitespace) {
        return None;
    }

    if trimmed.contains('@') {
        return parse_email(trimmed);
    }

    let lower = trimmed.to_ascii_lowercase();
    for scheme in ["https", "http", "ftp", "file"] {
        if lower.starts_with(scheme) && trimmed[scheme.len()..].starts_with(':') {
            let word = if scheme == "file" {
                "file".to_string()
            } else {
                scheme.to_ascii_uppercase()
            };
            // The remainder starts at ':' — render_remainder emits colon/slash.
            return Some(format!(
                "{} {}",
                word,
                render_remainder(&trimmed[scheme.len()..], true)
            ));
        }
    }

    if lower.starts_with("www.") {
        return Some(render_remainder(trimmed, true));
    }

    if is_bare_domain(trimmed) {
        return Some(render_domain_path(trimmed, false));
    }

    None
}

/// "word/word" or "word / word" (2+ alphabetic words joined by slashes) reads
/// the slash literally: "upgrade / update" → "upgrade slash update".
fn parse_slash_words(input: &str) -> Option<String> {
    if !input.contains('/') || input.contains('.') || input.contains('@') {
        return None;
    }
    // "and/or" is kept literal by NeMo's whitelist, not read as a slash.
    if input.eq_ignore_ascii_case("and/or") {
        return None;
    }
    let segs: Vec<&str> = input.split('/').map(str::trim).collect();
    if segs.len() < 2
        || segs
            .iter()
            .any(|s| s.is_empty() || !s.chars().all(|c| c.is_ascii_alphabetic()))
    {
        return None;
    }
    Some(
        segs.iter()
            .map(|s| s.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(" slash "),
    )
}

/// Parse an email address to spoken form.
fn parse_email(input: &str) -> Option<String> {
    let (local, rhs) = input.split_once('@')?;
    if local.is_empty() || rhs.is_empty() || rhs.contains('@') {
        return None;
    }
    let local_spoken = local
        .split('.')
        .map(|l| render_label(l, true, false))
        .collect::<Vec<_>>()
        .join(" dot ");
    Some(format!(
        "{} at {}",
        local_spoken,
        render_domain_path(rhs, true)
    ))
}

/// Render a `domain[/path]` string with TLD casing on the final domain label.
fn render_domain_path(s: &str, context: bool) -> String {
    let (domain, path) = match s.split_once('/') {
        Some((d, p)) => (d, Some(p)),
        None => (s, None),
    };
    let labels: Vec<&str> = domain.split('.').collect();
    let last = labels.len().saturating_sub(1);
    let domain_spoken = labels
        .iter()
        .enumerate()
        .map(|(i, l)| render_label(l, context, i == last))
        .collect::<Vec<_>>()
        .join(" dot ");

    match path {
        Some(p) => format!("{} slash {}", domain_spoken, render_remainder(p, context)),
        None => domain_spoken,
    }
}

/// Render an arbitrary remainder (path / URL tail), emitting "slash", "dot",
/// and "colon" for structural characters and rendering the labels between.
fn render_remainder(s: &str, context: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let flush = |buf: &mut String, out: &mut Vec<String>| {
        if !buf.is_empty() {
            out.push(render_label(buf, context, false));
            buf.clear();
        }
    };
    for c in s.chars() {
        match c {
            '/' => {
                flush(&mut buf, &mut out);
                out.push("slash".to_string());
            }
            '.' => {
                flush(&mut buf, &mut out);
                out.push("dot".to_string());
            }
            ':' => {
                flush(&mut buf, &mut out);
                out.push("colon".to_string());
            }
            _ => buf.push(c),
        }
    }
    flush(&mut buf, &mut out);
    out.join(" ")
}

/// Render a single label: pure-alpha labels are cased as a word; mixed labels
/// split into letter runs (words), digit runs (spelled), and symbols.
fn render_label(label: &str, context: bool, is_final_tld: bool) -> String {
    if label.is_empty() {
        return String::new();
    }
    if label.chars().all(|c| c.is_ascii_alphabetic()) {
        return case_word(label, context, is_final_tld);
    }

    let mut out: Vec<String> = Vec::new();
    let mut chars = label.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            let mut run = String::new();
            while matches!(chars.peek(), Some(d) if d.is_ascii_alphabetic()) {
                run.push(chars.next().unwrap());
            }
            out.push(case_word(&run, context, false));
        } else if c.is_ascii_digit() {
            while matches!(chars.peek(), Some(d) if d.is_ascii_digit()) {
                out.push(digit_word(chars.next().unwrap()).to_string());
            }
        } else {
            chars.next();
            if let Some(w) = sym_word(c) {
                out.push(w.to_string());
            }
        }
    }
    out.join(" ")
}

/// Case a pure-alpha token by role.
fn case_word(word: &str, context: bool, is_final_tld: bool) -> String {
    let lw = word.to_ascii_lowercase();

    // Common file-extension expansion (lower case).
    if lw == "jpg" {
        return "jpeg".to_string();
    }
    if PROTOCOL_UPPER.contains(lw.as_str()) {
        return lw.to_ascii_uppercase();
    }
    if context && BRAND.contains(lw.as_str()) {
        return lw.to_ascii_uppercase();
    }
    if CCTLD.contains(lw.as_str()) {
        // Final-position country codes upper-case even in bare domains; in a
        // path they upper-case only in email/URL context.
        if is_final_tld || context {
            return lw.to_ascii_uppercase();
        }
        return lw;
    }
    if is_final_tld {
        if GTLD.contains(lw.as_str()) {
            return lw;
        }
        // Unknown TLD: upper-cased in email/URL context, left alone when bare.
        if context {
            return lw.to_ascii_uppercase();
        }
    }
    lw
}

fn sym_word(c: char) -> Option<&'static str> {
    match c {
        '-' => Some("dash"),
        '_' => Some("underscore"),
        '&' => Some("ampersand"),
        '~' => Some("tilde"),
        '+' => Some("plus"),
        _ => None,
    }
}

/// A bare domain has ≥2 dot-separated labels, a plausible alphabetic TLD, and
/// is not a decimal number. A trailing `/path` is allowed.
fn is_bare_domain(s: &str) -> bool {
    let domain = s.split('/').next().unwrap_or(s);
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 || labels.iter().any(|l| l.is_empty()) {
        return false;
    }
    let tld = labels[labels.len() - 1];
    let tld_ok = (2..=4).contains(&tld.len()) && tld.chars().all(|c| c.is_ascii_alphabetic());
    // Require an alphabetic character somewhere before the TLD so pure-numeric
    // dotted forms ("5.4", IP addresses) are left to other taggers.
    let has_alpha_body = labels[..labels.len() - 1]
        .iter()
        .any(|l| l.chars().any(|c| c.is_ascii_alphabetic()));
    tld_ok && has_alpha_body
}

fn digit_word(c: char) -> &'static str {
    match c {
        '0' => "zero",
        '1' => "one",
        '2' => "two",
        '3' => "three",
        '4' => "four",
        '5' => "five",
        '6' => "six",
        '7' => "seven",
        '8' => "eight",
        '9' => "nine",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_words() {
        assert_eq!(
            parse("a.bc@gmail.com"),
            Some("a dot bc at gmail dot com".to_string())
        );
        assert_eq!(
            parse("asdf123@abc.com"),
            Some("asdf one two three at abc dot com".to_string())
        );
    }

    #[test]
    fn test_email_tld_casing() {
        assert_eq!(
            parse("a@hotmail.de"),
            Some("a at hotmail dot DE".to_string())
        );
        assert_eq!(
            parse("abc@gmail.abc"),
            Some("abc at gmail dot ABC".to_string())
        );
    }

    #[test]
    fn test_url() {
        assert_eq!(
            parse("http://www.ourdailynews.com.sm"),
            Some("HTTP colon slash slash WWW dot ourdailynews dot com dot SM".to_string())
        );
        assert_eq!(
            parse("www.ourdailynews.com/123-sm"),
            Some("WWW dot ourdailynews dot com slash one two three dash SM".to_string())
        );
    }

    #[test]
    fn test_bare_domain() {
        assert_eq!(parse("test.com"), Some("test dot com".to_string()));
        assert_eq!(parse("test.abc"), Some("test dot abc".to_string()));
        assert_eq!(parse("test2.uk"), Some("test two dot UK".to_string()));
        // Bare path segments stay lower case.
        assert_eq!(
            parse("ourdailynews.com/12-sm"),
            Some("ourdailynews dot com slash one two dash sm".to_string())
        );
    }

    #[test]
    fn test_slash_words() {
        assert_eq!(
            parse("upgrade/update"),
            Some("upgrade slash update".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("5.4"), None);
        assert_eq!(parse("123"), None);
    }
}
