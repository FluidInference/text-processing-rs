//! Electronic TN tagger for Japanese (romaji output).
//!
//! Converts written emails and URLs to spoken Japanese romaji form:
//! - "test@gmail.com" → "t e s t atto g m a i l dotto c o m"
//! - "https://example.com" → "h t t p s koron surasshu surasshu e x a m p l e dotto c o m"

/// Parse an email or URL to spoken Japanese romaji form.
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

/// Parse an email address to spoken Japanese romaji form.
fn parse_email(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }

    let local = spell_domain(parts[0]);
    let domain = spell_domain(parts[1]);

    Some(format!("{} atto {}", local, domain))
}

/// Parse a URL to spoken Japanese romaji form.
fn parse_url(input: &str) -> Option<String> {
    let mut result = String::new();
    let lower = input.to_lowercase();

    let rest = if lower.starts_with("https://") {
        result.push_str("h t t p s koron surasshu surasshu");
        &input["https://".len()..]
    } else if lower.starts_with("http://") {
        result.push_str("h t t p koron surasshu surasshu");
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

/// Spell out a domain name, using "dotto" for periods.
fn spell_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    let spelled: Vec<String> = parts.iter().map(|p| spell_electronic(p)).collect();
    spelled.join(" dotto ")
}

/// Spell out an electronic string in Japanese romaji.
///
/// Letters are spelled individually (lowercase).
/// Digits use Japanese romaji words.
/// Special characters use Japanese connector words.
fn spell_electronic(s: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for c in s.chars() {
        match c {
            '-' => parts.push("haifen".to_string()),
            '_' => parts.push("anda baa".to_string()),
            '/' => parts.push("surasshu".to_string()),
            '~' => parts.push("chiruda".to_string()),
            ':' => parts.push("koron".to_string()),
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
        '1' => "ichi",
        '2' => "ni",
        '3' => "san",
        '4' => "yon",
        '5' => "go",
        '6' => "roku",
        '7' => "nana",
        '8' => "hachi",
        '9' => "kyuu",
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
            Some("t e s t atto g m a i l dotto c o m".to_string())
        );
        assert_eq!(
            parse("user123@example.co.jp"),
            Some("u s e r ichi ni san atto e x a m p l e dotto c o dotto j p".to_string())
        );
    }

    #[test]
    fn test_url_http() {
        assert_eq!(
            parse("http://www.example.com"),
            Some(
                "h t t p koron surasshu surasshu w w w dotto e x a m p l e dotto c o m".to_string()
            )
        );
        assert_eq!(
            parse("https://google.com"),
            Some("h t t p s koron surasshu surasshu g o o g l e dotto c o m".to_string())
        );
    }

    #[test]
    fn test_www_url() {
        assert_eq!(
            parse("www.example.com"),
            Some("w w w dotto e x a m p l e dotto c o m".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
