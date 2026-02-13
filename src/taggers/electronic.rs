//! Electronic address tagger.
//!
//! Converts spoken emails and URLs to written form:
//! - "a at gmail dot com" → "a@gmail.com"
//! - "w w w dot example dot com" → "www.example.com"
//! - "h t t p colon slash slash..." → "http://..."

/// Parse spoken electronic address to written form.
pub fn parse(input: &str) -> Option<String> {
    let original = input.trim();
    let input_lower = original.to_lowercase();

    // Try email pattern
    if let Some(result) = parse_email(original, &input_lower) {
        return Some(result);
    }

    // Try URL pattern
    if let Some(result) = parse_url(&input_lower) {
        return Some(result);
    }

    // Try domain pattern
    if let Some(result) = parse_domain(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse email address (contains " at ")
fn parse_email(original: &str, input: &str) -> Option<String> {
    if !input.contains(" at ") {
        return None;
    }

    let parts: Vec<&str> = input.splitn(2, " at ").collect();
    if parts.len() != 2 {
        return None;
    }

    // Domain part must contain " dot " to be a valid email domain
    // This prevents "set alarm at ten" from being parsed as email
    if !parts[1].contains(" dot ") {
        return None;
    }

    // Get the original local part to preserve casing
    let orig_parts: Vec<&str> = original.splitn(2, " at ").collect();
    let orig_local = if orig_parts.len() == 2 {
        orig_parts[0]
    } else {
        // Try case-insensitive split
        let at_pos = original.to_lowercase().find(" at ")?;
        &original[..at_pos]
    };

    let local_part = parse_email_part_with_case(orig_local, parts[0]);
    let domain_part = parse_domain_part(parts[1]);

    Some(format!("{}@{}", local_part, domain_part))
}

/// Parse email local part preserving original casing
fn parse_email_part_with_case(original: &str, _input: &str) -> String {
    let mut result = String::new();
    let words: Vec<&str> = original.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        let word_lower = word.to_lowercase();
        // "dot" at the start should be literal "dot", not "."
        // e.g., "dot three at gmail dot com" → "dot 3@gmail.com"
        if word_lower == "dot" && i == 0 {
            result.push_str(word);
            result.push(' ');
        } else if word_lower == "dot" {
            result.push('.');
        } else if word_lower == "underscore" {
            result.push('_');
        } else if word_lower == "dash" || word_lower == "hyphen" {
            result.push('-');
        } else if let Some(digit) = word_to_digit(&word_lower) {
            // Number word - convert to digit
            result.push(digit);
        } else if word.len() == 1 {
            // Single letter - preserve original case
            result.push_str(word);
        } else {
            result.push_str(&word.to_lowercase());
        }
    }

    result
}

/// Convert word to single digit
fn word_to_digit(word: &str) -> Option<char> {
    match word {
        "zero" | "o" | "oh" => Some('0'),
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        _ => None,
    }
}

/// Parse URL with protocol
fn parse_url(input: &str) -> Option<String> {
    // Check for protocol prefix
    let protocols = [
        ("h t t p s colon slash slash ", "https://"),
        ("h t t p colon slash slash ", "http://"),
        ("https colon slash slash ", "https://"),
        ("http colon slash slash ", "http://"),
    ];

    for (spoken, written) in &protocols {
        if input.starts_with(spoken) {
            let rest = &input[spoken.len()..];
            let domain = parse_domain_part(rest);
            return Some(format!("{}{}", written, domain));
        }
    }

    // Check for www prefix without protocol
    if input.starts_with("w w w dot ") {
        let rest = &input[10..];
        let domain = parse_domain_part(rest);
        return Some(format!("www.{}", domain));
    }

    None
}

/// Parse standalone domain
fn parse_domain(input: &str) -> Option<String> {
    // Must contain " dot " to be a domain
    if !input.contains(" dot ") {
        return None;
    }

    let result = parse_domain_part(input);

    // Must have at least one dot
    if result.contains('.') {
        Some(result)
    } else {
        None
    }
}

/// Parse email local part (before @)
fn parse_email_part(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();

    for (i, word) in words.iter().enumerate() {
        match *word {
            // "dot" at the start should be literal "dot", not "."
            // e.g., "dot three at gmail dot com" → "dot 3@gmail.com"
            "dot" if i == 0 => {
                result.push_str("dot ");
            }
            "dot" => result.push('.'),
            "hyphen" | "dash" => result.push('-'),
            "underscore" => result.push('_'),
            _ => {
                // Check for spelled out letters/numbers
                if let Some(c) = word_to_char(word) {
                    result.push(c);
                } else {
                    // Use word as-is (for things like "gmail", "abc")
                    result.push_str(word);
                }
            }
        }
    }

    result
}

/// Parse domain part (after @ or entire URL domain)
fn parse_domain_part(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();

    for word in words {
        match word {
            "dot" => result.push('.'),
            "slash" => result.push('/'),
            "colon" => result.push(':'),
            "hyphen" | "dash" => result.push('-'),
            _ => {
                // Check for spelled out letters/numbers
                if let Some(c) = word_to_char(word) {
                    result.push(c);
                } else {
                    // Use word as-is
                    result.push_str(word);
                }
            }
        }
    }

    result
}

/// Convert single letter/number word to character
fn word_to_char(word: &str) -> Option<char> {
    // Single letters
    if word.len() == 1 {
        let c = word.chars().next()?;
        if c.is_ascii_alphabetic() || c.is_ascii_digit() {
            return Some(c);
        }
    }

    // Spelled out numbers
    match word {
        "zero" | "o" | "oh" => Some('0'),
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_email() {
        assert_eq!(
            parse("a at gmail dot com"),
            Some("a@gmail.com".to_string())
        );
    }

    #[test]
    fn test_email_with_dots() {
        assert_eq!(
            parse("a dot b c at gmail dot com"),
            Some("a.bc@gmail.com".to_string())
        );
    }

    #[test]
    fn test_email_with_numbers() {
        assert_eq!(
            parse("a one b two at a b c dot com"),
            Some("a1b2@abc.com".to_string())
        );
    }

    #[test]
    fn test_url_with_protocol() {
        assert_eq!(
            parse("h t t p colon slash slash w w w dot example dot com"),
            Some("http://www.example.com".to_string())
        );
    }

    #[test]
    fn test_www_domain() {
        assert_eq!(
            parse("w w w dot example dot com"),
            Some("www.example.com".to_string())
        );
    }

    #[test]
    fn test_simple_domain() {
        assert_eq!(parse("nvidia dot com"), Some("nvidia.com".to_string()));
    }
}
