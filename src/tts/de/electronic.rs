//! Electronic TN tagger for German.
//!
//! Converts written emails and URLs to spoken German form:
//! - "test@gmail.com" -> "t e s t at g m a i l punkt c o m"
//! - "http://www.example.com" -> "h t t p doppelpunkt schraegstrich schraegstrich w w w punkt e x a m p l e punkt c o m"

/// Parse an email or URL to spoken German form.
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

/// Parse an email address to spoken German form.
fn parse_email(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }

    let local = spell_domain(parts[0]);
    let domain = spell_domain(parts[1]);

    Some(format!("{} at {}", local, domain))
}

/// Parse a URL to spoken German form.
fn parse_url(input: &str) -> Option<String> {
    let mut result = String::new();
    let lower = input.to_lowercase();

    let rest = if lower.starts_with("https://") {
        result.push_str("h t t p s doppelpunkt schraegstrich schraegstrich");
        &input["https://".len()..]
    } else if lower.starts_with("http://") {
        result.push_str("h t t p doppelpunkt schraegstrich schraegstrich");
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

/// Spell out a domain name, using "punkt" for periods.
fn spell_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    let spelled: Vec<String> = parts.iter().map(|p| spell_electronic(p)).collect();
    spelled.join(" punkt ")
}

/// Spell out an electronic string in German.
///
/// Letters are spelled individually with spaces.
/// Digit runs are spelled individually using German digit words.
/// Special characters are mapped to German words.
fn spell_electronic(s: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for c in s.chars() {
        match c {
            '-' => parts.push("bindestrich".to_string()),
            '_' => parts.push("unterstrich".to_string()),
            '/' => parts.push("schraegstrich".to_string()),
            '~' => parts.push("tilde".to_string()),
            ':' => parts.push("doppelpunkt".to_string()),
            c if c.is_ascii_alphabetic() => {
                parts.push(c.to_lowercase().to_string());
            }
            c if c.is_ascii_digit() => {
                parts.push(digit_word_de(c));
            }
            _ => {
                // Skip unknown characters
            }
        }
    }

    parts.join(" ")
}

fn digit_word_de(c: char) -> String {
    match c {
        '0' => "null",
        '1' => "eins",
        '2' => "zwei",
        '3' => "drei",
        '4' => "vier",
        '5' => "fuenf",
        '6' => "sechs",
        '7' => "sieben",
        '8' => "acht",
        '9' => "neun",
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
            Some("t e s t at g m a i l punkt c o m".to_string())
        );
        assert_eq!(
            parse("info@example.de"),
            Some("i n f o at e x a m p l e punkt d e".to_string())
        );
    }

    #[test]
    fn test_url_http() {
        assert_eq!(
            parse("http://www.example.com"),
            Some(
                "h t t p doppelpunkt schraegstrich schraegstrich w w w punkt e x a m p l e punkt c o m"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_url_www() {
        assert_eq!(
            parse("www.example.de"),
            Some("w w w punkt e x a m p l e punkt d e".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("hallo"), None);
        assert_eq!(parse("12345"), None);
    }
}
