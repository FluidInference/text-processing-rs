//! Roman-numeral TN tagger.
//!
//! Converts a roman numeral that follows a section keyword to a cardinal:
//! - "Chapter IV" → "Chapter four"
//! - "PART XL" → "PART forty"
//!
//! Only the keyword-anchored form is handled. NeMo also reads a numeral after
//! a personal name as an ordinal ("Sam II" → "Sam second"), but that relies on
//! a large name list; without it a bare-capitalized heuristic over-fires on
//! ordinary words, so it is intentionally left out.

use super::number_to_words;

/// Section words that mark a following roman numeral as a cardinal. Matched
/// case-insensitively; the prefix is echoed back with its original casing.
const KEYWORDS: &[&str] = &[
    "chapter",
    "class",
    "part",
    "article",
    "section",
    "paragraph",
];

/// Parse a `"<keyword> <roman>"` pair to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let (prefix, roman) = trimmed.split_once(' ')?;
    if !KEYWORDS.contains(&prefix.to_lowercase().as_str()) {
        return None;
    }
    let value = roman_to_int(roman.trim())?;
    Some(format!("{} {}", prefix, number_to_words(value)))
}

/// Convert a roman numeral to its value, or `None` if it is not a valid
/// numeral (empty, or containing a non-roman letter).
fn roman_to_int(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut total = 0i64;
    let mut prev = 0i64;
    for c in s.chars().rev() {
        let value = match c.to_ascii_uppercase() {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if value < prev {
            total -= value;
        } else {
            total += value;
            prev = value;
        }
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_cardinal() {
        assert_eq!(parse("Chapter IV"), Some("Chapter four".to_string()));
        assert_eq!(parse("PART XL"), Some("PART forty".to_string()));
        assert_eq!(parse("section iii"), Some("section three".to_string()));
        assert_eq!(parse("Article XII"), Some("Article twelve".to_string()));
    }

    #[test]
    fn test_not_roman() {
        assert_eq!(parse("Chapter Five"), None); // not a numeral
        assert_eq!(parse("Sam II"), None); // name path not handled
        assert_eq!(parse("hello world"), None);
        assert_eq!(parse("Chapter"), None);
    }

    #[test]
    fn test_roman_values() {
        assert_eq!(roman_to_int("IV"), Some(4));
        assert_eq!(roman_to_int("XL"), Some(40));
        assert_eq!(roman_to_int("MCMXciv"), Some(1994));
        assert_eq!(roman_to_int("IIII"), Some(4)); // lax: additive form allowed
        assert_eq!(roman_to_int("hi"), None);
    }
}
