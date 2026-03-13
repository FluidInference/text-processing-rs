//! Money tagger.
//!
//! Converts spoken currency expressions to written form:
//! - "five dollars" → "$5"
//! - "five dollars and fifty cents" → "$5.50"
//! - "one cent" → "$0.01"
//! - "fifteen hundred dollars" → "$1500"

use super::cardinal::words_to_number;

/// Parse spoken money expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let original = input.trim();
    let input_lower = original.to_lowercase();

    // "one dollars" is grammatically incorrect - pass through
    if input_lower == "one dollars" {
        return None;
    }

    // Try other currencies (won, yen, yuan)
    if let Some(result) = parse_other_currency(&input_lower) {
        return Some(result);
    }

    // Try large currency first (most specific - contains scale words)
    if let Some(result) = parse_large_currency(original, &input_lower) {
        return Some(result);
    }

    // Try decimal dollar patterns (twenty point five o six dollars)
    if let Some(result) = parse_decimal_dollars(&input_lower) {
        return Some(result);
    }

    // Try dollars and cents
    if let Some(result) = parse_dollars_and_cents(&input_lower) {
        return Some(result);
    }

    if let Some(result) = parse_dollars(&input_lower) {
        return Some(result);
    }

    if let Some(result) = parse_cents(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse other currencies (won, yen, yuan)
fn parse_other_currency(input: &str) -> Option<String> {
    // Korean won: "X billion won" → "₩X billion"
    for scale in &["trillion", "billion", "million"] {
        let pattern = format!(" {} won", scale);
        if input.ends_with(&pattern) {
            let num_part = input.trim_end_matches(&pattern);
            let num = words_to_number(num_part)? as i64;
            return Some(format!("₩{} {}", num, scale));
        }
    }

    // Japanese yen: "X billion yen" → "¥X billion"
    for scale in &["trillion", "billion", "million"] {
        let pattern = format!(" {} yen", scale);
        if input.ends_with(&pattern) {
            let num_part = input.trim_end_matches(&pattern);
            let num = words_to_number(num_part)? as i64;
            return Some(format!("¥{} {}", num, scale));
        }
    }

    // Chinese yuan: "X billion yuan" → "X billion yuan" (no symbol)
    for scale in &["trillion", "billion", "million"] {
        let pattern = format!(" {} yuan", scale);
        if input.ends_with(&pattern) {
            let num_part = input.trim_end_matches(&pattern);
            // Handle decimal like "one point six nine billion yuan"
            if num_part.contains(" point ") {
                let parts: Vec<&str> = num_part.split(" point ").collect();
                if parts.len() == 2 {
                    let integer = words_to_number(parts[0])? as i64;
                    let decimal = parse_decimal_digits(parts[1])?;
                    return Some(format!("{}.{} {} yuan", integer, decimal, scale));
                }
            }
            let num = words_to_number(num_part)? as i64;
            return Some(format!("{} {} yuan", num, scale));
        }
    }

    None
}

/// Parse decimal dollar patterns like "twenty point five o six dollars"
fn parse_decimal_dollars(input: &str) -> Option<String> {
    // Pattern: "X point Y dollars" where Y can contain "o"
    if input.ends_with(" dollars") && input.contains(" point ") {
        let num_part = input.trim_end_matches(" dollars");
        let parts: Vec<&str> = num_part.splitn(2, " point ").collect();
        if parts.len() == 2 {
            let integer = if parts[0].is_empty() {
                String::new()
            } else {
                (words_to_number(parts[0])? as i64).to_string()
            };
            let decimal = parse_decimal_digits(parts[1])?;
            if integer.is_empty() {
                return Some(format!("$.{}", decimal));
            }
            return Some(format!("${}.{}", integer, decimal));
        }
    }

    // Pattern: "point X dollars" (no integer part)
    if input.starts_with("point ") && input.ends_with(" dollars") {
        let decimal_part = input.strip_prefix("point ")?.strip_suffix(" dollars")?;
        let decimal = parse_decimal_digits(decimal_part)?;
        return Some(format!("$.{}", decimal));
    }

    None
}

/// Parse "X dollars and Y cents" pattern
fn parse_dollars_and_cents(input: &str) -> Option<String> {
    // Pattern: "X united states dollars and Y cents"
    if let Some((dollars_part, rest)) = input.split_once(" united states dollars and ") {
        if rest.ends_with(" cents") || rest.ends_with(" cent") {
            let cents_words = rest.trim_end_matches(" cents").trim_end_matches(" cent");
            let dollars = words_to_number(dollars_part)? as i64;
            let cents = words_to_number(cents_words)? as i64;
            return Some(format!("${}.{:02}", dollars, cents));
        }
    }

    // Pattern: "X dollar and Y cents" (singular)
    if let Some((dollars_part, rest)) = input.split_once(" dollar and ") {
        if rest.ends_with(" cents") || rest.ends_with(" cent") {
            let cents_words = rest.trim_end_matches(" cents").trim_end_matches(" cent");
            let dollars = words_to_number(dollars_part)? as i64;
            let cents = words_to_number(cents_words)? as i64;
            return Some(format!("${}.{:02}", dollars, cents));
        }
    }

    // Pattern: "X dollars and Y cents"
    if let Some((dollars_part, rest)) = input.split_once(" dollars and ") {
        if rest.ends_with(" cents") || rest.ends_with(" cent") {
            let cents_words = rest.trim_end_matches(" cents").trim_end_matches(" cent");
            let dollars = words_to_number(dollars_part)? as i64;
            let cents = words_to_number(cents_words)? as i64;
            return Some(format!("${}.{:02}", dollars, cents));
        }
    }

    // Pattern: "X dollars Y cents" (without "and")
    if let Some((dollars_part, rest)) = input.split_once(" dollars ") {
        if rest.ends_with(" cents") {
            let cents_words = rest.trim_end_matches(" cents");
            let dollars = words_to_number(dollars_part)? as i64;
            let cents = words_to_number(cents_words)? as i64;
            return Some(format!("${}.{:02}", dollars, cents));
        }
        // Pattern: "X dollars Y" (implied cents, e.g., "seventy five dollars sixty three")
        if let Some(cents) = words_to_number(rest) {
            let cents = cents as i64;
            if cents > 0 && cents < 100 {
                let dollars = words_to_number(dollars_part)? as i64;
                return Some(format!("${}.{:02}", dollars, cents));
            }
        }
    }

    None
}

/// Parse "X dollars" pattern
fn parse_dollars(input: &str) -> Option<String> {
    // "one dollar" (singular)
    if input == "one dollar" {
        return Some("$1".to_string());
    }

    // "X dollar" (singular with number, e.g., "twenty dollar")
    if input.ends_with(" dollar") {
        let num_part = input.trim_end_matches(" dollar");
        let num = parse_money_number(num_part)?;
        return Some(format!("${}", num));
    }

    // "X dollars"
    if input.ends_with(" dollars") {
        let num_part = input.trim_end_matches(" dollars");
        let num = parse_money_number(num_part)?;
        return Some(format!("${}", num));
    }

    None
}

/// Parse money number, handling shorthand like "one fifty five" = 155
fn parse_money_number(input: &str) -> Option<i64> {
    let words: Vec<&str> = input.split_whitespace().collect();

    // Try shorthand patterns first
    if words.len() >= 2 {
        // Check for "X hundred" pattern at the end (ninety nine hundred = 9900)
        if *words.last()? == "hundred" {
            let prefix = words[..words.len() - 1].join(" ");
            if let Some(num) = words_to_number(&prefix) {
                return Some((num as i64) * 100);
            }
        }

        // Check for "X YY" shorthand (one fifty five = 155)
        // Only applies when first word is a single digit (1-9)
        let first_word = words[0];
        let is_single_digit = matches!(
            first_word,
            "one" | "two" | "three" | "four" | "five" | "six" | "seven" | "eight" | "nine"
        );

        if is_single_digit {
            if let Some(first) = words_to_number(first_word) {
                let first = first as i64;
                let rest = words[1..].join(" ");
                // Rest must be a two-digit number (10-99)
                if let Some(tens_ones) = words_to_number(&rest) {
                    let tens_ones = tens_ones as i64;
                    if tens_ones >= 10 && tens_ones <= 99 {
                        return Some(first * 100 + tens_ones);
                    }
                }
            }
        }
    }

    // Fall back to standard cardinal parsing
    words_to_number(input).map(|n| n as i64)
}

/// Parse "X cents" pattern
fn parse_cents(input: &str) -> Option<String> {
    if input == "one cent" {
        return Some("$0.01".to_string());
    }

    if input.ends_with(" cents") {
        let num_part = input.trim_end_matches(" cents");
        let cents = words_to_number(num_part)? as i64;
        return Some(format!("$0.{:02}", cents));
    }

    None
}

/// Parse large currency amounts (billions, millions)
fn parse_large_currency(original: &str, input_lower: &str) -> Option<String> {
    // "X billion dollars" → "$X billion"
    for scale in &["trillion", "billion", "million"] {
        let pattern = format!(" {} dollars", scale);
        if input_lower.ends_with(&pattern) {
            let num_part = &input_lower[..input_lower.len() - pattern.len()];

            // Extract original scale word to preserve casing
            // "dollars" is 7 chars, scale is scale.len() chars, space is 1 char
            let scale_start = original.len() - 7 - 1 - scale.len();
            let scale_end = original.len() - 7 - 1;
            let orig_scale = &original[scale_start..scale_end];

            // Handle decimal like "two point five billion dollars"
            if num_part.contains(" point ") {
                let result = parse_decimal_scale(num_part, orig_scale)?;
                return Some(result);
            }
            let num = words_to_number(num_part)? as i64;
            return Some(format!("${} {}", num, orig_scale));
        }
    }

    None
}

/// Parse decimal scale numbers like "two point five"
fn parse_decimal_scale(input: &str, scale: &str) -> Option<String> {
    let parts: Vec<&str> = input.split(" point ").collect();
    if parts.len() != 2 {
        return None;
    }

    let integer = words_to_number(parts[0])? as i64;
    let decimal = parse_decimal_digits(parts[1])?;

    Some(format!("${}.{} {}", integer, decimal, scale))
}

/// Parse decimal digits ("five" → "5", "five o" → "50")
fn parse_decimal_digits(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();

    for word in words {
        let digit = match word {
            "zero" | "o" | "oh" => '0',
            "one" => '1',
            "two" => '2',
            "three" => '3',
            "four" => '4',
            "five" => '5',
            "six" => '6',
            "seven" => '7',
            "eight" => '8',
            "nine" => '9',
            _ => return None,
        };
        result.push(digit);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dollars() {
        assert_eq!(parse("one dollar"), Some("$1".to_string()));
        assert_eq!(parse("five dollars"), Some("$5".to_string()));
        assert_eq!(parse("twenty dollars"), Some("$20".to_string()));
        assert_eq!(parse("one hundred dollars"), Some("$100".to_string()));
        assert_eq!(
            parse("fifteen thousand dollars"),
            Some("$15000".to_string())
        );
    }

    #[test]
    fn test_dollars_and_cents() {
        assert_eq!(
            parse("one dollar and fifty cents"),
            Some("$1.50".to_string())
        );
        assert_eq!(
            parse("five dollars and twenty five cents"),
            Some("$5.25".to_string())
        );
        assert_eq!(
            parse("eleven dollars and fifty one cents"),
            Some("$11.51".to_string())
        );
    }

    #[test]
    fn test_dollars_implied_cents() {
        assert_eq!(
            parse("seventy five dollars sixty three"),
            Some("$75.63".to_string())
        );
        assert_eq!(
            parse("twenty nine dollars fifty"),
            Some("$29.50".to_string())
        );
    }

    #[test]
    fn test_cents() {
        assert_eq!(parse("one cent"), Some("$0.01".to_string()));
        assert_eq!(parse("fifty cents"), Some("$0.50".to_string()));
        assert_eq!(parse("ninety nine cents"), Some("$0.99".to_string()));
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(
            parse("fifty million dollars"),
            Some("$50 million".to_string())
        );
        assert_eq!(
            parse("fifty billion dollars"),
            Some("$50 billion".to_string())
        );
        assert_eq!(
            parse("two point five billion dollars"),
            Some("$2.5 billion".to_string())
        );
    }

    #[test]
    fn test_not_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("five"), None);
    }
}
