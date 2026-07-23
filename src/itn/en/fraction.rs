//! Fraction tagger for English (inverse text normalization).
//!
//! Converts spoken English fractions to written form:
//! - "one third" → "1/3"
//! - "two thirds" → "2/3"
//! - "three quarters" → "3/4"
//! - "one half" → "1/2"
//! - "twenty two thirds" → "22/3"
//!
//! Runs *before* the ordinal tagger. Without it, "one third" is read as a
//! compound ordinal (1 + 3 → "4th"); see issue #82. The plural/singular of the
//! denominator disambiguates fractions from compound ordinals:
//! - Plural denominator ("thirds") is always a fraction: "twenty two thirds" → 22/3.
//! - Singular denominator ("third") is a fraction only with numerator one
//!   ("one third" → 1/3); otherwise it is a compound ordinal
//!   ("twenty third" → 23rd) and this tagger defers to the ordinal tagger.

use super::cardinal;

/// Parse a spoken English fraction to written form, or `None` when the input
/// is not an unambiguous fraction (so higher-priority taggers can handle it).
pub fn parse(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();

    // A fraction needs at least a numerator word and a denominator word.
    if tokens.len() < 2 {
        return None;
    }

    let (denominator, denom_is_plural) = parse_denominator(tokens.last()?)?;

    // The numerator is everything before the denominator. `words_to_number`
    // rejects the article "a"/"an", so "a quarter of the pizza" is left alone.
    let numerator = cardinal::words_to_number(&tokens[..tokens.len() - 1].join(" "))?;
    if numerator < 1 {
        return None;
    }

    // A singular denominator only reads as a fraction with numerator one.
    // "twenty third" (numerator 20) is the ordinal 23rd, not 20/3.
    if !denom_is_plural && numerator != 1 {
        return None;
    }

    Some(format!("{}/{}", numerator, denominator))
}

/// Map a denominator word to `(value, is_plural)`, or `None` when the word is
/// not a fraction denominator.
///
/// Excludes "first"/"second" (and their plurals): "one second" is a duration,
/// not 1/2, and English never spells 1/2 as "second". Scale denominators
/// ("hundredth", "thousandth", ...) only count when plural, so singular
/// "one hundredth" stays the 100th ordinal.
fn parse_denominator(word: &str) -> Option<(i128, bool)> {
    // Irregular forms that don't follow the "-s" plural rule.
    match word {
        "half" => return Some((2, false)),
        "halves" => return Some((2, true)),
        "quarter" => return Some((4, false)),
        "quarters" => return Some((4, true)),
        _ => {}
    }

    let (singular, is_plural) = match word.strip_suffix('s') {
        Some(stem) => (stem, true),
        None => (word, false),
    };

    let value = match singular {
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        "tenth" => 10,
        "eleventh" => 11,
        "twelfth" => 12,
        "thirteenth" => 13,
        "fourteenth" => 14,
        "fifteenth" => 15,
        "sixteenth" => 16,
        "seventeenth" => 17,
        "eighteenth" => 18,
        "nineteenth" => 19,
        "twentieth" => 20,
        "thirtieth" => 30,
        "fortieth" => 40,
        "fiftieth" => 50,
        "sixtieth" => 60,
        "seventieth" => 70,
        "eightieth" => 80,
        "ninetieth" => 90,
        // Scale denominators are fractions only when plural ("two hundredths"),
        // leaving singular "one hundredth" to the ordinal tagger (100th).
        "hundredth" if is_plural => 100,
        "thousandth" if is_plural => 1000,
        "millionth" if is_plural => 1_000_000,
        "billionth" if is_plural => 1_000_000_000,
        _ => return None,
    };
    Some((value, is_plural))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_singular() {
        assert_eq!(parse("one third"), Some("1/3".to_string()));
        assert_eq!(parse("one half"), Some("1/2".to_string()));
        assert_eq!(parse("one quarter"), Some("1/4".to_string()));
        assert_eq!(parse("one fourth"), Some("1/4".to_string()));
        assert_eq!(parse("one fifth"), Some("1/5".to_string()));
        assert_eq!(parse("one tenth"), Some("1/10".to_string()));
    }

    #[test]
    fn test_plural() {
        assert_eq!(parse("two thirds"), Some("2/3".to_string()));
        assert_eq!(parse("three quarters"), Some("3/4".to_string()));
        assert_eq!(parse("five eighths"), Some("5/8".to_string()));
        assert_eq!(parse("twenty two thirds"), Some("22/3".to_string()));
        assert_eq!(parse("three halves"), Some("3/2".to_string()));
    }

    #[test]
    fn test_compound_ordinals_defer() {
        // Singular denominator with numerator != 1 is a compound ordinal.
        assert_eq!(parse("twenty third"), None);
        assert_eq!(parse("thirty first"), None);
        assert_eq!(parse("forty second"), None);
        assert_eq!(parse("one hundred third"), None);
    }

    #[test]
    fn test_excluded_denominators() {
        // "second"/"first" are never fraction denominators.
        assert_eq!(parse("one second"), None);
        assert_eq!(parse("two seconds"), None);
        assert_eq!(parse("one first"), None);
        // Singular scale words stay ordinals (100th, 1000th).
        assert_eq!(parse("one hundredth"), None);
        assert_eq!(parse("one thousandth"), None);
        // ...but plural scale words are fractions.
        assert_eq!(parse("two hundredths"), Some("2/100".to_string()));
    }

    #[test]
    fn test_article_not_a_numerator() {
        // "a quarter" must not become 1/4 (would break "a quarter past three"
        // and "a quarter of the pizza").
        assert_eq!(parse("a quarter"), None);
        assert_eq!(parse("a third"), None);
    }

    #[test]
    fn test_not_a_fraction() {
        assert_eq!(parse("third"), None);
        assert_eq!(parse("hello world"), None);
        assert_eq!(parse("one"), None);
    }
}
