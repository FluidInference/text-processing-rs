//! Electronic TN tagger for Spanish.
//!
//! Converts written emails and URLs to spoken Spanish form:
//! - "test@gmail.com" → "t e s t arroba g m a i l punto c o m"
//! - "https://www.example.com" → "h t t p s dos puntos barra barra w w w punto e x a m p l e punto c o m"

/// Parse an email or URL to spoken Spanish form.
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

/// Parse an email address to spoken Spanish form.
fn parse_email(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }

    let local = spell_domain(parts[0]);
    let domain = spell_domain(parts[1]);

    Some(format!("{} arroba {}", local, domain))
}

/// Parse a URL to spoken Spanish form.
fn parse_url(input: &str) -> Option<String> {
    let mut result = String::new();
    let lower = input.to_lowercase();

    let rest = if lower.starts_with("https://") {
        result.push_str("h t t p s dos puntos barra barra");
        &input["https://".len()..]
    } else if lower.starts_with("http://") {
        result.push_str("h t t p dos puntos barra barra");
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

/// Spell out a domain name, using "punto" for periods.
fn spell_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    let spelled: Vec<String> = parts.iter().map(|p| spell_electronic(p)).collect();
    spelled.join(" punto ")
}

/// Spell out an electronic string in Spanish.
///
/// Letters are spelled individually with spaces.
/// Digit runs are spelled individually in Spanish.
/// Special characters use Spanish names.
fn spell_electronic(s: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for c in s.chars() {
        match c {
            '-' => parts.push("guion".to_string()),
            '_' => parts.push("guion bajo".to_string()),
            '/' => parts.push("barra".to_string()),
            '~' => parts.push("tilde".to_string()),
            ':' => parts.push("dos puntos".to_string()),
            c if c.is_ascii_alphabetic() => {
                parts.push(c.to_lowercase().to_string());
            }
            c if c.is_ascii_digit() => {
                parts.push(digit_word_es(c));
            }
            _ => {
                // Skip unknown characters
            }
        }
    }

    parts.join(" ")
}

fn digit_word_es(c: char) -> String {
    match c {
        '0' => "cero",
        '1' => "uno",
        '2' => "dos",
        '3' => "tres",
        '4' => "cuatro",
        '5' => "cinco",
        '6' => "seis",
        '7' => "siete",
        '8' => "ocho",
        '9' => "nueve",
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
            Some("t e s t arroba g m a i l punto c o m".to_string())
        );
        assert_eq!(
            parse("juan.perez@ejemplo.com"),
            Some("j u a n punto p e r e z arroba e j e m p l o punto c o m".to_string())
        );
    }

    #[test]
    fn test_url_http() {
        assert_eq!(
            parse("http://www.ejemplo.com"),
            Some(
                "h t t p dos puntos barra barra w w w punto e j e m p l o punto c o m".to_string()
            )
        );
        assert_eq!(
            parse("https://google.com"),
            Some("h t t p s dos puntos barra barra g o o g l e punto c o m".to_string())
        );
    }

    #[test]
    fn test_www_url() {
        assert_eq!(
            parse("www.ejemplo.com"),
            Some("w w w punto e j e m p l o punto c o m".to_string())
        );
    }

    #[test]
    fn test_non_electronic() {
        assert_eq!(parse("hola"), None);
        assert_eq!(parse("123"), None);
    }
}
