//! Electronic tagger for German.
//!
//! Converts spoken German email/URL descriptions to written form:
//! - "a b c at g mail punkt com" → "abc@gmail.com"
//! - "h t t p s doppelpunkt slash slash w w w punkt a b c punkt com" → "https://www.abc.com"

/// Parse spoken German electronic address to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Must contain "at" (email) or "doppelpunkt" or "punkt" (URL)
    if !input_trim.contains(" at ")
        && !input_trim.contains("doppelpunkt")
        && !input_trim.contains(" punkt ")
    {
        return None;
    }

    let tokens: Vec<&str> = input_trim.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    // Convert tokens to characters/symbols
    let mut result = String::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];
        match token {
            "at" => result.push('@'),
            "punkt" => result.push('.'),
            "bindestrich" => result.push('-'),
            "unterstrich" => result.push('_'),
            "doppelpunkt" => result.push(':'),
            "slash" => result.push('/'),
            "fragezeichen" => result.push('?'),
            "gleichheitszeichen" => result.push('='),
            "tilde" => result.push('~'),
            _ => {
                // Single letter
                if token.len() == 1 && token.chars().all(|c| c.is_ascii_alphabetic()) {
                    result.push_str(token);
                } else if let Some(digit) = word_to_digit(token) {
                    result.push_str(digit);
                } else {
                    // Multi-char token that's not a keyword - treat as literal
                    result.push_str(token);
                }
            }
        }
        i += 1;
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Convert German digit word to digit string
fn word_to_digit(word: &str) -> Option<&'static str> {
    match word {
        "null" => Some("0"),
        "eins" | "ein" | "eine" => Some("1"),
        "zwei" => Some("2"),
        "drei" => Some("3"),
        "vier" => Some("4"),
        "fünf" => Some("5"),
        "sechs" => Some("6"),
        "sieben" => Some("7"),
        "acht" => Some("8"),
        "neun" => Some("9"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert_eq!(
            parse("a b c at g mail punkt com"),
            Some("abc@gmail.com".to_string())
        );
        assert_eq!(
            parse("a b c at a b c punkt com"),
            Some("abc@abc.com".to_string())
        );
    }

    #[test]
    fn test_email_with_digits() {
        assert_eq!(
            parse("a eins b zwei at a b c punkt com"),
            Some("a1b2@abc.com".to_string())
        );
    }

    #[test]
    fn test_url() {
        assert_eq!(
            parse("h t t p s doppelpunkt slash slash w w w punkt a b c punkt com"),
            Some("https://www.abc.com".to_string())
        );
        assert_eq!(
            parse("w w w punkt a b c punkt com"),
            Some("www.abc.com".to_string())
        );
    }
}
