//! Electronic TN tagger for French.
//!
//! Converts written emails and URLs to spoken French form:
//! - "test@gmail.com" → "t e s t arobase g m a i l point c o m"
//! - "http://www.example.com" → "h t t p deux-points barre oblique barre oblique w w w point e x a m p l e point c o m"

/// Parse an email or URL to spoken French form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Email detection: contains @ with text on both sides
    if trimmed.contains('@') {
        return parse_email(trimmed);
    }

    // URL detection: starts with http://, https://, or www.
    let lower = trimmed.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.") {
        return parse_url(trimmed);
    }

    None
}

/// Parse an email address to spoken French form.
fn parse_email(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }

    let local = spell_domain(parts[0]);
    let domain = spell_domain(parts[1]);

    Some(format!("{} arobase {}", local, domain))
}

/// Parse a URL to spoken French form.
fn parse_url(input: &str) -> Option<String> {
    let mut result = String::new();
    let lower = input.to_lowercase();

    let rest = if lower.starts_with("https://") {
        result.push_str("h t t p s deux-points barre oblique barre oblique");
        &input["https://".len()..]
    } else if lower.starts_with("http://") {
        result.push_str("h t t p deux-points barre oblique barre oblique");
        &input["http://".len()..]
    } else {
        input
    };

    if !result.is_empty() && !rest.is_empty() {
        result.push(' ');
    }

    result.push_str(&spell_domain(rest));

    Some(result)
}

/// Spell out a domain name, using "point" for periods.
fn spell_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    let spelled: Vec<String> = parts.iter().map(|p| spell_electronic(p)).collect();
    spelled.join(" point ")
}

/// Spell out an electronic string in French.
///
/// Letters are spelled individually with spaces.
/// Digit runs are spelled individually using French words.
/// Special characters use French connector words.
fn spell_electronic(s: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for c in s.chars() {
        match c {
            '-' => parts.push("tiret".to_string()),
            '_' => parts.push("tiret bas".to_string()),
            '/' => parts.push("barre oblique".to_string()),
            '~' => parts.push("tilde".to_string()),
            ':' => parts.push("deux-points".to_string()),
            c if c.is_ascii_alphabetic() => {
                parts.push(c.to_lowercase().to_string());
            }
            c if c.is_ascii_digit() => {
                parts.push(digit_word_fr(c));
            }
            _ => {
                // Skip unknown characters
            }
        }
    }

    parts.join(" ")
}

fn digit_word_fr(c: char) -> String {
    match c {
        '0' => "zero",
        '1' => "un",
        '2' => "deux",
        '3' => "trois",
        '4' => "quatre",
        '5' => "cinq",
        '6' => "six",
        '7' => "sept",
        '8' => "huit",
        '9' => "neuf",
        _ => "",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert_eq!(
            parse("test@gmail.com"),
            Some("t e s t arobase g m a i l point c o m".to_string())
        );
        assert_eq!(
            parse("jean.dupont@example.fr"),
            Some("j e a n point d u p o n t arobase e x a m p l e point f r".to_string())
        );
    }

    #[test]
    fn test_url_http() {
        assert_eq!(
            parse("http://www.example.com"),
            Some(
                "h t t p deux-points barre oblique barre oblique w w w point e x a m p l e point c o m"
                    .to_string()
            )
        );
        assert_eq!(
            parse("https://google.fr"),
            Some(
                "h t t p s deux-points barre oblique barre oblique g o o g l e point f r".to_string()
            )
        );
    }

    #[test]
    fn test_www_url() {
        assert_eq!(
            parse("www.exemple.fr"),
            Some("w w w point e x e m p l e point f r".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("bonjour"), None);
        assert_eq!(parse("123"), None);
    }
}
