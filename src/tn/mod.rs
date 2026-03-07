//! Text Normalization taggers for English.
//!
//! Converts written-form text to spoken form (the reverse of ITN):
//! - "200" → "two hundred"
//! - "$5.50" → "five dollars and fifty cents"
//! - "January 5, 2025" → "january fifth twenty twenty five"

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod electronic;
pub mod measure;
pub mod money;
pub mod ordinal;
pub mod telephone;
pub mod time;
pub mod whitelist;

/// Ones words indexed by value (0..20).
const ONES: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
    "nineteen",
];

/// Tens words indexed by tens digit (2..10 → index 0..8).
const TENS: [&str; 8] = [
    "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

/// Convert an integer to English words.
///
/// Examples:
/// - `0` → `"zero"`
/// - `21` → `"twenty one"`
/// - `123` → `"one hundred twenty three"`
/// - `1000` → `"one thousand"`
/// - `-42` → `"minus forty two"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    if n < 0 {
        return format!("minus {}", number_to_words(-n));
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n as u64;

    // Process scale groups from largest to smallest
    let scales: &[(u64, &str)] = &[
        (1_000_000_000_000_000_000, "quintillion"),
        (1_000_000_000_000_000, "quadrillion"),
        (1_000_000_000_000, "trillion"),
        (1_000_000_000, "billion"),
        (1_000_000, "million"),
        (1_000, "thousand"),
    ];

    for &(scale_value, scale_name) in scales {
        if remaining >= scale_value {
            let chunk = remaining / scale_value;
            remaining %= scale_value;
            parts.push(format!("{} {}", chunk_to_words(chunk as u32), scale_name));
        }
    }

    // Remainder (0..999)
    if remaining > 0 {
        parts.push(chunk_to_words(remaining as u32));
    }

    parts.join(" ")
}

/// Convert a number 1..999 to words.
fn chunk_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n < 1000);
    let mut parts: Vec<&str> = Vec::new();

    let hundreds = n / 100;
    let rest = n % 100;

    if hundreds > 0 {
        parts.push(ONES[hundreds as usize]);
        parts.push("hundred");
    }

    if rest >= 20 {
        let tens_idx = (rest / 10 - 2) as usize;
        parts.push(TENS[tens_idx]);
        let ones = rest % 10;
        if ones > 0 {
            parts.push(ONES[ones as usize]);
        }
    } else if rest > 0 {
        parts.push(ONES[rest as usize]);
    }

    parts.join(" ")
}

/// Spell each digit of a string individually.
///
/// "14" → "one four"
pub fn spell_digits(s: &str) -> String {
    s.chars()
        .filter_map(|c| c.to_digit(10).map(|d| ONES[d as usize]))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_words_basic() {
        assert_eq!(number_to_words(0), "zero");
        assert_eq!(number_to_words(1), "one");
        assert_eq!(number_to_words(10), "ten");
        assert_eq!(number_to_words(11), "eleven");
        assert_eq!(number_to_words(19), "nineteen");
        assert_eq!(number_to_words(20), "twenty");
        assert_eq!(number_to_words(21), "twenty one");
        assert_eq!(number_to_words(99), "ninety nine");
    }

    #[test]
    fn test_number_to_words_hundreds() {
        assert_eq!(number_to_words(100), "one hundred");
        assert_eq!(number_to_words(101), "one hundred one");
        assert_eq!(number_to_words(123), "one hundred twenty three");
        assert_eq!(number_to_words(999), "nine hundred ninety nine");
    }

    #[test]
    fn test_number_to_words_thousands() {
        assert_eq!(number_to_words(1000), "one thousand");
        assert_eq!(number_to_words(1001), "one thousand one");
        assert_eq!(number_to_words(1234), "one thousand two hundred thirty four");
        assert_eq!(number_to_words(10000), "ten thousand");
        assert_eq!(number_to_words(100000), "one hundred thousand");
    }

    #[test]
    fn test_number_to_words_millions() {
        assert_eq!(number_to_words(1000000), "one million");
        assert_eq!(number_to_words(2000003), "two million three");
    }

    #[test]
    fn test_number_to_words_negative() {
        assert_eq!(number_to_words(-42), "minus forty two");
        assert_eq!(number_to_words(-1000), "minus one thousand");
    }

    #[test]
    fn test_spell_digits() {
        assert_eq!(spell_digits("14"), "one four");
        assert_eq!(spell_digits("0"), "zero");
        assert_eq!(spell_digits("987"), "nine eight seven");
    }
}
