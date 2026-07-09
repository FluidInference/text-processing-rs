//! Ordinal TN tagger.
//!
//! Converts written ordinal numbers to spoken form:
//! - "1st" → "first"
//! - "2nd" → "second"
//! - "21st" → "twenty first"
//! - "100th" → "one hundredth"

use super::number_to_words;

/// Parse a written ordinal (e.g. "21st") to spoken words (e.g. "twenty first").
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Detect suffix: st, nd, rd, th
    let (num_str, suffix) = if let Some(s) = trimmed.strip_suffix("st") {
        (s, "st")
    } else if let Some(s) = trimmed.strip_suffix("nd") {
        (s, "nd")
    } else if let Some(s) = trimmed.strip_suffix("rd") {
        (s, "rd")
    } else if let Some(s) = trimmed.strip_suffix("th") {
        (s, "th")
    } else {
        return None;
    };

    // The part before the suffix must be digits (comma grouping allowed:
    // "1,000th" → "one thousandth").
    let digits: String = num_str.chars().filter(|c| *c != ',').collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: i64 = digits.parse().ok()?;
    if n < 0 {
        return None;
    }

    // Malformed ordinal — the written suffix does not match the number's true
    // ordinal suffix ("1th", "111st"). NeMo reads the number (cardinal below
    // 100, digit-by-digit at 100+) and appends the literal suffix as a word.
    if suffix != correct_suffix(n) {
        let number = if n < 100 {
            number_to_words(n)
        } else {
            super::spell_digits(&digits)
        };
        return Some(format!("{} {}", number, suffix));
    }

    // "0th" → "zeroth" (cardinal_to_ordinal maps the "zero" tail to "zeroth").
    let cardinal = number_to_words(n);
    Some(cardinal_to_ordinal(&cardinal))
}

/// The correct ordinal suffix for `n` (11–13 are always "th").
fn correct_suffix(n: i64) -> &'static str {
    if (11..=13).contains(&(n % 100)) {
        return "th";
    }
    match n % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

/// Spell an integer as ordinal words (`2640` → "two thousand six hundred
/// fortieth"). Shared with the fraction tagger for denominators.
pub(crate) fn number_to_ordinal_words(n: i64) -> String {
    cardinal_to_ordinal(&number_to_words(n))
}

/// Convert cardinal words to ordinal words by replacing the last word.
///
/// "twenty one" → "twenty first"
/// "one hundred" → "one hundredth"
fn cardinal_to_ordinal(cardinal: &str) -> String {
    let words: Vec<&str> = cardinal.split_whitespace().collect();
    if words.is_empty() {
        return cardinal.to_string();
    }

    let last = words[words.len() - 1];
    let ordinal_last = match last {
        "one" => "first",
        "two" => "second",
        "three" => "third",
        "four" => "fourth",
        "five" => "fifth",
        "six" => "sixth",
        "seven" => "seventh",
        "eight" => "eighth",
        "nine" => "ninth",
        "ten" => "tenth",
        "eleven" => "eleventh",
        "twelve" => "twelfth",
        "thirteen" => "thirteenth",
        "fourteen" => "fourteenth",
        "fifteen" => "fifteenth",
        "sixteen" => "sixteenth",
        "seventeen" => "seventeenth",
        "eighteen" => "eighteenth",
        "nineteen" => "nineteenth",
        "twenty" => "twentieth",
        "thirty" => "thirtieth",
        "forty" => "fortieth",
        "fifty" => "fiftieth",
        "sixty" => "sixtieth",
        "seventy" => "seventieth",
        "eighty" => "eightieth",
        "ninety" => "ninetieth",
        "hundred" => "hundredth",
        "thousand" => "thousandth",
        "million" => "millionth",
        "billion" => "billionth",
        "trillion" => "trillionth",
        _ => return format!("{}th", cardinal),
    };

    if words.len() == 1 {
        ordinal_last.to_string()
    } else {
        let prefix: Vec<&str> = words[..words.len() - 1].to_vec();
        format!("{} {}", prefix.join(" "), ordinal_last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ordinals() {
        assert_eq!(parse("1st"), Some("first".to_string()));
        assert_eq!(parse("2nd"), Some("second".to_string()));
        assert_eq!(parse("3rd"), Some("third".to_string()));
        assert_eq!(parse("4th"), Some("fourth".to_string()));
        assert_eq!(parse("5th"), Some("fifth".to_string()));
    }

    #[test]
    fn test_teens() {
        assert_eq!(parse("11th"), Some("eleventh".to_string()));
        assert_eq!(parse("12th"), Some("twelfth".to_string()));
        assert_eq!(parse("13th"), Some("thirteenth".to_string()));
    }

    #[test]
    fn test_compound() {
        assert_eq!(parse("21st"), Some("twenty first".to_string()));
        assert_eq!(parse("22nd"), Some("twenty second".to_string()));
        assert_eq!(parse("23rd"), Some("twenty third".to_string()));
        assert_eq!(parse("99th"), Some("ninety ninth".to_string()));
    }

    #[test]
    fn test_large() {
        assert_eq!(parse("100th"), Some("one hundredth".to_string()));
        assert_eq!(parse("1000th"), Some("one thousandth".to_string()));
        assert_eq!(parse("101st"), Some("one hundred first".to_string()));
    }

    #[test]
    fn test_zeroth_and_commas() {
        assert_eq!(parse("0th"), Some("zeroth".to_string()));
        assert_eq!(parse("1,000th"), Some("one thousandth".to_string()));
        assert_eq!(parse("9,000,000th"), Some("nine millionth".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("st"), None);
    }
}
