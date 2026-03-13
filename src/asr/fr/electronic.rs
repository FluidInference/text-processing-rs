//! Electronic tagger for French.
//!
//! Converts spoken French electronic addresses to written form:
//! - "test arobase gmail point com" → "test@gmail.com"
//! - "a b c at g mail point com" → "abc@gmail.com"
//! - Handles both "arobase" (French) and "at" (English) for @
//! - Converts digit words to digits: "un" → "1", "trois" → "3"

/// Parse spoken French electronic address to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    parse_email(&input_lower)
}

/// Parse email address pattern
fn parse_email(input: &str) -> Option<String> {
    // Look for "arobase" or "at" as the @ indicator
    let (local_raw, domain_raw) = if input.contains(" arobase ") {
        let parts: Vec<&str> = input.splitn(2, " arobase ").collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0].trim(), parts[1].trim())
    } else if input.contains(" at ") {
        let parts: Vec<&str> = input.splitn(2, " at ").collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0].trim(), parts[1].trim())
    } else {
        return None;
    };

    let local_part = convert_email_part(local_raw);
    let domain_part = convert_email_part(domain_raw);

    if local_part.is_empty() || domain_part.is_empty() {
        return None;
    }

    Some(format!("{}@{}", local_part, domain_part))
}

/// Convert email part:
/// - "point" → "."
/// - "tiret" → "-"
/// - single letter words are concatenated: "a b c" → "abc"
/// - digit words are converted: "un" → "1", "deux" → "2"
/// - multi-letter words are kept as-is and concatenated
fn convert_email_part(input: &str) -> String {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();
    let mut need_concat = true; // letters/words are concatenated

    for token in tokens {
        if token == "point" {
            result.push('.');
            need_concat = true;
        } else if token == "tiret" {
            result.push('-');
            need_concat = true;
        } else if token == "tiret du bas" || token == "sous-tiret" || token == "underscore" {
            result.push('_');
            need_concat = true;
        } else if let Some(d) = word_to_digit(token) {
            result.push(char::from(b'0' + d));
        } else {
            // Regular word or letter — concatenate directly
            if need_concat {
                result.push_str(token);
                need_concat = false;
            } else {
                result.push_str(token);
            }
        }
    }

    result
}

/// Convert digit word to digit
fn word_to_digit(word: &str) -> Option<u8> {
    match word {
        "zéro" | "zero" => Some(0),
        "un" | "une" => Some(1),
        "deux" => Some(2),
        "trois" => Some(3),
        "quatre" => Some(4),
        "cinq" => Some(5),
        "six" => Some(6),
        "sept" => Some(7),
        "huit" => Some(8),
        "neuf" => Some(9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_email_arobase() {
        assert_eq!(
            parse("test arobase gmail point com"),
            Some("test@gmail.com".to_string())
        );
    }

    #[test]
    fn test_email_with_at() {
        assert_eq!(
            parse("a b c at g mail point com"),
            Some("abc@gmail.com".to_string())
        );
    }

    #[test]
    fn test_email_with_digits() {
        assert_eq!(
            parse("a un b deux arobase a b c point com"),
            Some("a1b2@abc.com".to_string())
        );
    }

    #[test]
    fn test_email_with_dots() {
        assert_eq!(
            parse("a b trois point s d d point trois arobase g mail point com"),
            Some("ab3.sdd.3@gmail.com".to_string())
        );
    }

    #[test]
    fn test_email_with_dash() {
        assert_eq!(
            parse("jean tiret luc arobase example point com"),
            Some("jean-luc@example.com".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("test gmail dot com"), None); // No arobase or at
        assert_eq!(parse("arobase"), None); // Missing parts
    }
}
