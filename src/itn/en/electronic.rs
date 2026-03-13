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
    if let Some(result) = parse_url(original, &input_lower) {
        return Some(result);
    }

    // Try domain pattern
    if let Some(result) = parse_domain(original, &input_lower) {
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

    // Find " at " position case-insensitively in original
    let at_pos = input.find(" at ")?;
    let orig_local = &original[..at_pos];
    let orig_domain = &original[at_pos + 4..];

    let local_part = parse_email_part_with_case(orig_local);
    let domain_part = parse_domain_part_with_case(orig_domain);

    Some(format!("{}@{}", local_part, domain_part))
}

/// Parse email local part preserving original casing
fn parse_email_part_with_case(original: &str) -> String {
    let mut result = String::new();
    let words: Vec<&str> = original.split_whitespace().collect();
    let mut prev_had_dot = false;

    for (i, word) in words.iter().enumerate() {
        let word_lower = word.to_lowercase();
        // "dot" at the start should be literal "dot", not "."
        // e.g., "dot three at gmail dot com" → "dot 3@gmail.com"
        if word_lower == "dot" && i == 0 {
            result.push_str(word);
            result.push(' ');
            prev_had_dot = false;
        } else if word_lower == "dot" {
            result.push('.');
            prev_had_dot = false;
        } else if word_lower == "underscore" {
            result.push('_');
            prev_had_dot = false;
        } else if word_lower == "dash" || word_lower == "hyphen" {
            result.push('-');
            prev_had_dot = false;
        } else if let Some(digit) = word_to_digit(&word_lower) {
            // Number word - convert to digit
            if prev_had_dot {
                result.push(' ');
                prev_had_dot = false;
            }
            result.push(digit);
        } else if word.len() == 1 {
            // Single letter - preserve original case
            // If previous token contained a dot, add space separator
            if prev_had_dot {
                result.push(' ');
                prev_had_dot = false;
            }
            result.push_str(word);
        } else {
            // Multi-char word — preserve original case
            // If it contains a dot (e.g., "WWW.A"), keep as-is and mark for spacing
            if prev_had_dot {
                result.push(' ');
            }
            result.push_str(word);
            prev_had_dot = word.contains('.');
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

/// Check if a word is a domain keyword (case-insensitive)
fn is_domain_keyword(word_lower: &str) -> bool {
    matches!(word_lower, "dot" | "slash" | "colon" | "hyphen" | "dash")
}

/// Parse URL with protocol, preserving original casing for domain parts
fn parse_url(original: &str, input: &str) -> Option<String> {
    // Check for protocol prefix
    let protocols = [
        ("h t t p s colon slash slash ", "https://"),
        ("h t t p colon slash slash ", "http://"),
        ("https colon slash slash ", "https://"),
        ("http colon slash slash ", "http://"),
    ];

    for (spoken, written) in &protocols {
        if input.starts_with(spoken) {
            let orig_rest = &original[spoken.len()..];
            let domain = parse_domain_part_with_case(orig_rest);
            return Some(format!("{}{}", written, domain));
        }
    }

    // Check for "w w w dot " prefix (case-insensitive) without protocol
    if input.starts_with("w w w dot ") {
        let orig_rest = &original[10..];
        let domain = parse_domain_part_with_case(orig_rest);
        return Some(format!("www.{}", domain));
    }

    // Check for "W W W. " pattern (individual letters followed by period already in text)
    if input.starts_with("w w w. ") {
        let orig_rest = &original[7..];
        let domain = parse_domain_part_with_case(orig_rest);
        return Some(format!("www.{}", domain));
    }

    // Check for "w w w . " pattern (individual letters, space, dot, space)
    if input.starts_with("w w w . ") {
        let orig_rest = &original[8..];
        let domain = parse_domain_part_with_case(orig_rest);
        return Some(format!("www.{}", domain));
    }

    None
}

/// Parse standalone domain, preserving case
fn parse_domain(original: &str, input: &str) -> Option<String> {
    // Must contain " dot " to be a domain
    if !input.contains(" dot ") {
        return None;
    }

    let result = parse_domain_part_with_case(original);

    // Must have at least one dot
    if result.contains('.') {
        Some(result)
    } else {
        None
    }
}

/// Check if a word is a common English word that should stop domain parsing
fn is_stop_word(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "and"
            | "or"
            | "the"
            | "a"
            | "an"
            | "is"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "of"
            | "with"
            | "from"
            | "by"
            | "as"
            | "it"
            | "this"
            | "that"
            | "but"
            | "not"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "has"
            | "have"
            | "had"
            | "do"
            | "does"
            | "did"
            | "will"
            | "would"
            | "could"
            | "should"
            | "may"
            | "might"
            | "can"
            | "shall"
    )
}

/// Parse domain part preserving original casing.
/// Stops parsing when encountering non-domain content and appends remainder as-is.
fn parse_domain_part_with_case(original: &str) -> String {
    let words: Vec<&str> = original.split_whitespace().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < words.len() {
        let word = words[i];
        let word_lower = word.to_lowercase();

        // Stop at common English multi-char words that are clearly not domain parts
        // (single-letter words like "a" should still be treated as domain letters)
        if word.len() > 1 && is_stop_word(&word_lower) {
            let remaining = &words[i..];
            let tail = remaining.join(" ");
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&tail);
            return result;
        }

        match word_lower.as_str() {
            "dot" => result.push('.'),
            "slash" => result.push('/'),
            "colon" => result.push(':'),
            "hyphen" | "dash" => result.push('-'),
            _ => {
                // Check for spelled out letters/numbers
                if let Some(c) = word_to_char_with_case(word) {
                    // If this single letter is followed by a multi-char word,
                    // join them as one word (lowercase the letter to match)
                    // e.g., "N vidia" → "nvidia"
                    if c.is_ascii_alphabetic()
                        && i + 1 < words.len()
                        && words[i + 1].len() > 1
                        && words[i + 1].chars().all(|ch| ch.is_ascii_alphabetic())
                        && !is_domain_keyword(&words[i + 1].to_lowercase())
                    {
                        result.push(c.to_ascii_lowercase());
                        result.push_str(words[i + 1]);
                        i += 2;
                        continue;
                    }
                    result.push(c);
                } else if word.len() > 1 && word.chars().all(|c| c.is_ascii_alphabetic()) {
                    // Multi-char alphabetic word — preserve original case
                    result.push_str(word);
                } else {
                    // Non-domain token (e.g., ".", digits, special chars)
                    // Stop parsing and append rest as-is
                    let remaining = &words[i..];
                    let tail = remaining.join(" ");
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&tail);
                    return result;
                }
            }
        }
        i += 1;
    }

    result
}

/// Convert single letter/number word to character, preserving original case for letters
fn word_to_char_with_case(word: &str) -> Option<char> {
    // Single letters - preserve case
    if word.len() == 1 {
        let c = word.chars().next()?;
        if c.is_ascii_alphabetic() || c.is_ascii_digit() {
            return Some(c);
        }
    }

    // Spelled out numbers (case-insensitive)
    let word_lower = word.to_lowercase();
    match word_lower.as_str() {
        "zero" | "oh" => Some('0'),
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
        assert_eq!(parse("a at gmail dot com"), Some("a@gmail.com".to_string()));
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

    #[test]
    fn test_case_preservation_domain() {
        assert_eq!(parse("NVIDIA dot com"), Some("NVIDIA.com".to_string()));
        assert_eq!(parse("NVIDIA dot COM"), Some("NVIDIA.COM".to_string()));
        assert_eq!(parse("Kore dot ai"), Some("Kore.ai".to_string()));
    }

    #[test]
    fn test_case_preservation_email() {
        assert_eq!(
            parse("Abc at gmail dot com"),
            Some("Abc@gmail.com".to_string())
        );
        assert_eq!(
            parse("Athreed at gmail dot com"),
            Some("Athreed@gmail.com".to_string())
        );
    }
}
