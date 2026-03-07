//! Electronic TN tagger.
//!
//! Converts written emails and URLs to spoken form:
//! - "test@gmail.com" → "test at gmail dot com"
//! - "http://www.example.com" → "h t t p colon slash slash w w w dot example dot com"

/// Parse an email or URL to spoken form.
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
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("www.")
    {
        return parse_url(trimmed);
    }

    None
}

/// Parse an email address to spoken form.
fn parse_email(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }

    // Local part: dots become "dot" just like in domain
    let local = spell_domain(parts[0]);
    let domain = spell_domain(parts[1]);

    Some(format!("{} at {}", local, domain))
}

/// Parse a URL to spoken form.
fn parse_url(input: &str) -> Option<String> {
    let mut result = String::new();

    let rest = if let Some(r) = input.strip_prefix("https://") {
        result.push_str("h t t p s colon slash slash");
        r
    } else if let Some(r) = input.strip_prefix("http://") {
        result.push_str("h t t p colon slash slash");
        r
    } else {
        input
    };

    if !result.is_empty() && !rest.is_empty() {
        result.push(' ');
    }

    result.push_str(&spell_domain(rest));

    Some(result)
}

/// Spell out a domain name, using "dot" for periods.
fn spell_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    let spelled: Vec<String> = parts.iter().map(|p| spell_electronic(p)).collect();
    spelled.join(" dot ")
}

/// Spell out an electronic string.
///
/// Letters are spelled individually with spaces.
/// Digit runs are spelled individually.
/// Hyphens become "dash", underscores become "underscore".
fn spell_electronic(s: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for c in s.chars() {
        match c {
            '-' => parts.push("dash".to_string()),
            '_' => parts.push("underscore".to_string()),
            '/' => parts.push("slash".to_string()),
            '~' => parts.push("tilde".to_string()),
            c if c.is_ascii_alphabetic() => {
                parts.push(c.to_lowercase().to_string());
            }
            c if c.is_ascii_digit() => {
                parts.push(digit_word(c));
            }
            _ => {
                // Skip unknown characters
            }
        }
    }

    parts.join(" ")
}

fn digit_word(c: char) -> String {
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
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert_eq!(
            parse("test@gmail.com"),
            Some("t e s t at g m a i l dot c o m".to_string())
        );
        assert_eq!(
            parse("john.doe@example.com"),
            Some("j o h n dot d o e at e x a m p l e dot c o m".to_string())
        );
    }

    #[test]
    fn test_url() {
        assert_eq!(
            parse("http://www.example.com"),
            Some("h t t p colon slash slash w w w dot e x a m p l e dot c o m".to_string())
        );
        assert_eq!(
            parse("https://google.com"),
            Some("h t t p s colon slash slash g o o g l e dot c o m".to_string())
        );
    }

    #[test]
    fn test_www_url() {
        assert_eq!(
            parse("www.example.com"),
            Some("w w w dot e x a m p l e dot c o m".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
