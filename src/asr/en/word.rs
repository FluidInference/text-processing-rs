//! Word tagger.
//!
//! Handles special word patterns:
//! - Spelled letters with numbers: "e s three" → "es3"
//! - Numbers with trailing punctuation: "twenty!" → "20 !"

use super::cardinal;

/// Parse special word patterns.
pub fn parse(input: &str) -> Option<String> {
    // Try spelled letters with number pattern
    if let Some(result) = parse_spelled_with_number(input) {
        return Some(result);
    }

    // Try number with trailing punctuation
    if let Some(result) = parse_number_with_punctuation(input) {
        return Some(result);
    }

    None
}

/// Parse spelled letters followed by a number: "e s three" → "es3"
/// Pattern: one or more single letters, then a number word at the end
fn parse_spelled_with_number(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }

    // Last word must be a number
    let last_word = words.last()?;
    let num = cardinal::words_to_number(last_word)?;

    // All preceding words must be single letters
    let letter_words = &words[..words.len() - 1];
    if letter_words.is_empty() {
        return None;
    }

    let mut result = String::new();
    for word in letter_words {
        // Must be exactly one ASCII letter
        if word.len() != 1 {
            return None;
        }
        let c = word.chars().next()?;
        if !c.is_ascii_alphabetic() {
            return None;
        }
        result.push(c);
    }

    // Append the number
    result.push_str(&(num as i64).to_string());
    Some(result)
}

/// Parse number word with trailing punctuation: "twenty!" → "20 !"
fn parse_number_with_punctuation(input: &str) -> Option<String> {
    // Check for trailing punctuation
    let last_char = input.chars().last()?;
    if !last_char.is_ascii_punctuation() {
        return None;
    }

    // Don't match if it's just punctuation
    if input.len() == 1 {
        return None;
    }

    let text_part = &input[..input.len() - last_char.len_utf8()];

    // Try to parse as cardinal number
    if let Some(num) = cardinal::parse(text_part) {
        return Some(format!("{} {}", num, last_char));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spelled_with_number() {
        assert_eq!(parse("e s three"), Some("es3".to_string()));
    }

    #[test]
    fn test_number_with_punctuation() {
        assert_eq!(parse("twenty!"), Some("20 !".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("twenty"), None); // No punctuation
    }
}
