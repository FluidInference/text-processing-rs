//! Electronic TN tagger for Mandarin Chinese (pinyin output).
//!
//! Converts written emails and URLs to spoken form in pinyin:
//! - "test@gmail.com" -> "t e s t at g m a i l dian c o m"
//! - "http://www.example.com" -> "h t t p mao hao xie gang xie gang w w w dian e x a m p l e dian c o m"

/// Parse an email or URL to spoken form in Mandarin Chinese pinyin.
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

/// Parse an email address to spoken form in pinyin.
fn parse_email(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }

    let local = spell_domain(parts[0]);
    let domain = spell_domain(parts[1]);

    Some(format!("{} at {}", local, domain))
}

/// Parse a URL to spoken form in pinyin.
fn parse_url(input: &str) -> Option<String> {
    let mut result = String::new();
    let lower = input.to_lowercase();

    let rest = if lower.starts_with("https://") {
        result.push_str("h t t p s mao hao xie gang xie gang");
        &input["https://".len()..]
    } else if lower.starts_with("http://") {
        result.push_str("h t t p mao hao xie gang xie gang");
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

/// Spell out a domain name, using "dian" for periods.
fn spell_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    let spelled: Vec<String> = parts.iter().map(|p| spell_electronic(p)).collect();
    spelled.join(" dian ")
}

/// Spell out an electronic string in Mandarin Chinese pinyin.
///
/// Letters are spelled individually (lowercase).
/// Digits use Chinese pinyin words.
/// Special characters use Chinese names.
fn spell_electronic(s: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for c in s.chars() {
        match c {
            '-' => parts.push("gang".to_string()),
            '_' => parts.push("xia hua xian".to_string()),
            '/' => parts.push("xie gang".to_string()),
            '~' => parts.push("bo lang hao".to_string()),
            ':' => parts.push("mao hao".to_string()),
            c if c.is_ascii_alphabetic() => {
                parts.push(c.to_lowercase().to_string());
            }
            c if c.is_ascii_digit() => {
                parts.push(digit_pinyin(c));
            }
            _ => {
                // Skip unknown characters
            }
        }
    }

    parts.join(" ")
}

fn digit_pinyin(c: char) -> String {
    match c {
        '0' => "ling",
        '1' => "yi",
        '2' => "er",
        '3' => "san",
        '4' => "si",
        '5' => "wu",
        '6' => "liu",
        '7' => "qi",
        '8' => "ba",
        '9' => "jiu",
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
            Some("t e s t at g m a i l dian c o m".to_string())
        );
        assert_eq!(
            parse("user123@example.org"),
            Some("u s e r yi er san at e x a m p l e dian o r g".to_string())
        );
    }

    #[test]
    fn test_url_http() {
        assert_eq!(
            parse("http://www.example.com"),
            Some(
                "h t t p mao hao xie gang xie gang w w w dian e x a m p l e dian c o m"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_url_www() {
        assert_eq!(
            parse("www.baidu.com"),
            Some("w w w dian b a i d u dian c o m".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12345"), None);
    }
}
