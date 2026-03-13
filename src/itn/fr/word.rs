//! Word tagger for French.
//!
//! Converts spoken French letter sequences to written form:
//! - "a b c" → "ABC"
//! - Handles spelled-out words and acronyms

/// Parse spoken French letter sequence to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Try parsing as a sequence of letters
    if let Some(result) = parse_letter_sequence(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse sequence of letter words into uppercase letters
fn parse_letter_sequence(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    // Need at least 2 letters to be considered a sequence
    if tokens.len() < 2 {
        return None;
    }

    let mut letters = Vec::new();

    for token in tokens {
        if let Some(letter) = parse_letter(token) {
            letters.push(letter);
        } else {
            // If any token is not a letter, this isn't a letter sequence
            return None;
        }
    }

    Some(letters.join(""))
}

/// Parse single letter word to uppercase letter
fn parse_letter(word: &str) -> Option<String> {
    // French letter names
    let letter_map = [
        ("a", "A"),
        ("bé", "B"),
        ("cé", "C"),
        ("dé", "D"),
        ("e", "E"),
        ("effe", "F"),
        ("gé", "G"),
        ("hache", "H"),
        ("i", "I"),
        ("ji", "J"),
        ("ka", "K"),
        ("elle", "L"),
        ("emme", "M"),
        ("enne", "N"),
        ("o", "O"),
        ("pé", "P"),
        ("ku", "Q"),
        ("erre", "R"),
        ("esse", "S"),
        ("té", "T"),
        ("u", "U"),
        ("vé", "V"),
        ("double vé", "W"),
        ("ixe", "X"),
        ("i grec", "Y"),
        ("zède", "Z"),
    ];

    for (spoken, letter) in &letter_map {
        if word == *spoken {
            return Some(letter.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence() {
        assert_eq!(parse("a bé cé"), Some("ABC".to_string()));
    }

    #[test]
    fn test_longer_sequence() {
        assert_eq!(parse("u esse a"), Some("USA".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("a"), None); // Single letter
        assert_eq!(parse("hello world"), None); // Not letters
    }
}
