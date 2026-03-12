//! Date tagger.
//!
//! Converts spoken date expressions to written form:
//! - "july twenty fifth two thousand twelve" → "july 25 2012"
//! - "nineteen eighties" → "1980s"
//! - "the twenty fifth of july" → "25 july"
//! - "january first" → "january 1"

use super::cardinal::words_to_number;
use super::ordinal;

/// Month names for matching
const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// Parse spoken date expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let original = input.trim();
    let input_lower = original.to_lowercase();

    // Try quarter pattern first (most specific)
    if let Some(result) = parse_quarter(&input_lower) {
        return Some(result);
    }

    // Try BC/AD years
    if let Some(result) = parse_bc_year(&input_lower) {
        return Some(result);
    }

    // Try decades (nineteen eighties → 1980s)
    if let Some(result) = parse_decade(&input_lower) {
        return Some(result);
    }

    // Try "the Xth of month [year]" pattern
    if let Some(result) = parse_day_of_month(original, &input_lower) {
        return Some(result);
    }

    // Try month + year first (july 2012, july two thousand twelve)
    // This must come before month_day_year to avoid "two" being parsed as day 2
    if let Some(result) = parse_month_year(original, &input_lower) {
        return Some(result);
    }

    // Try month + day + year patterns (july twenty fifth twenty twelve)
    if let Some(result) = parse_month_day_year(original, &input_lower) {
        return Some(result);
    }

    // Try standalone year patterns
    if let Some(result) = parse_year(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse quarter expressions like "second quarter of twenty twenty two" → "Q2 2022"
fn parse_quarter(input: &str) -> Option<String> {
    let quarters = [
        ("first quarter of ", "Q1"),
        ("second quarter of ", "Q2"),
        ("third quarter of ", "Q3"),
        ("fourth quarter of ", "Q4"),
    ];

    for (pattern, q) in &quarters {
        if input.starts_with(pattern) {
            let year_part = input.strip_prefix(pattern)?;
            let year = parse_year_number(year_part)?;
            return Some(format!("{} {}", q, year));
        }
    }

    None
}

/// Parse BC years like "seven fifty b c" → "750BC"
fn parse_bc_year(input: &str) -> Option<String> {
    let suffixes = [" b c", " bc", " a d", " ad"];
    for suffix in &suffixes {
        if input.ends_with(suffix) {
            let num_part = input.strip_suffix(suffix)?;
            // Try year-style parsing first (seven fifty → 750)
            // This handles patterns like "seven fifty" as 7*100+50
            let year =
                parse_old_year(num_part).or_else(|| words_to_number(num_part).map(|n| n as i64))?;
            let era = suffix.replace(" ", "").to_uppercase();
            return Some(format!("{}{}", year, era));
        }
    }
    None
}

/// Parse old-style year like "seven fifty" → 750, "twelve thirty four" → 1234
fn parse_old_year(input: &str) -> Option<i64> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }

    // First word is century (ones or tens digit)
    let century = words_to_number(words[0])? as i64;
    if century < 1 || century > 99 {
        return None;
    }

    // Remaining words are the two-digit year
    let year_part = words[1..].join(" ");
    let year_digits = words_to_number(&year_part)? as i64;
    if year_digits < 0 || year_digits > 99 {
        return None;
    }

    Some(century * 100 + year_digits)
}

/// Parse decades like "nineteen eighties" → "1980s"
fn parse_decade(input: &str) -> Option<String> {
    let decades = [
        ("twenties", 20),
        ("thirties", 30),
        ("forties", 40),
        ("fifties", 50),
        ("sixties", 60),
        ("seventies", 70),
        ("eighties", 80),
        ("nineties", 90),
    ];

    for (suffix, decade_val) in &decades {
        if input.ends_with(suffix) {
            let prefix = input.strip_suffix(suffix)?.trim();
            if prefix.is_empty() {
                // Just "eighties" without century
                return Some(format!("{}s", decade_val));
            }
            // Parse century prefix like "nineteen"
            let century = parse_century_prefix(prefix)?;
            return Some(format!("{}{}s", century, decade_val));
        }
    }

    None
}

/// Parse century prefix (e.g., "nineteen" → 19)
fn parse_century_prefix(input: &str) -> Option<i64> {
    match input {
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "twenty one" => Some(21),
        _ => None,
    }
}

/// Parse "the Xth of month [year]" pattern
fn parse_day_of_month(original: &str, input: &str) -> Option<String> {
    if !input.starts_with("the ") {
        return None;
    }

    let rest = input.strip_prefix("the ")?;

    // Find " of "
    let parts: Vec<&str> = rest.splitn(2, " of ").collect();
    if parts.len() != 2 {
        return None;
    }

    let day_part = parts[0];
    let month_year_part = parts[1];

    // Parse day as ordinal
    let day = ordinal::parse(day_part)?;
    // Remove suffix to get number
    let day_num: String = day.chars().filter(|c| c.is_ascii_digit()).collect();

    // Parse month and optional year
    let words: Vec<&str> = month_year_part.split_whitespace().collect();
    let orig_words: Vec<&str> = original.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let _month = find_month(words[0])?;
    // Find the original month casing
    let orig_month = find_original_month(orig_words.iter().copied(), words[0]);

    if words.len() == 1 {
        // Just month
        return Some(format!("{} {}", day_num, orig_month));
    }

    // Month + year
    let year_words = words[1..].join(" ");
    let year = parse_year_number(&year_words)?;
    Some(format!("{} {} {}", day_num, orig_month, year))
}

/// Parse month + day + year patterns
fn parse_month_day_year(original: &str, input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let orig_words: Vec<&str> = original.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    // First word must be a month
    let _month = find_month(words[0])?;
    let orig_month = orig_words.first().copied().unwrap_or(words[0]);

    if words.len() < 2 {
        return None;
    }

    // Try to find where day ends and year begins
    // Day can be ordinal (twenty fifth) or cardinal (thirty)
    for split_point in 2..=words.len().min(4) {
        let day_words = words[1..split_point].join(" ");

        // Try ordinal first
        if let Some(day_str) = ordinal::parse(&day_words) {
            let day_num: String = day_str.chars().filter(|c| c.is_ascii_digit()).collect();

            if split_point == words.len() {
                // No year
                return Some(format!("{} {}", orig_month, day_num));
            }

            // Try to parse year from remaining words
            let year_words = words[split_point..].join(" ");
            if let Some(year) = parse_year_number(&year_words) {
                return Some(format!("{} {} {}", orig_month, day_num, year));
            }
        }
    }

    // Try cardinal day (june thirty)
    if words.len() >= 2 {
        if let Some(day) = words_to_number(words[1]).map(|n| n as i64) {
            if day >= 1 && day <= 31 {
                if words.len() == 2 {
                    return Some(format!("{} {}", orig_month, day));
                }

                // Try to parse year
                let year_words = words[2..].join(" ");
                if let Some(year) = parse_year_number(&year_words) {
                    return Some(format!("{} {} {}", orig_month, day, year));
                }
            }
        }
    }

    None
}

/// Parse month + year (july 2012)
fn parse_month_year(original: &str, input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let orig_words: Vec<&str> = original.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }

    let _month = find_month(words[0])?;
    let orig_month = orig_words.first().copied().unwrap_or(words[0]);
    let year_words = words[1..].join(" ");
    let year = parse_year_number(&year_words)?;

    Some(format!("{} {}", orig_month, year))
}

/// Parse standalone year patterns
/// Only matches specific year patterns, not general numbers
fn parse_year(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();

    // "two thousand and X" or "one thousand X" patterns are always years
    if input.starts_with("two thousand") || input.starts_with("one thousand") {
        return parse_year_number(input).map(|y| y.to_string());
    }

    // "nineteen X" or "twenty X" pattern - must have exactly 2 words
    // This prevents "twenty one" from being matched as 2001 (should be cardinal 21)
    if words.len() == 2 {
        let century_prefix = words[0];
        let year_suffix = words[1];

        // For "twenty X", only allow teens (ten-nineteen) as suffix
        // This allows "twenty twelve" → 2012 but not "twenty one" → 2001
        if century_prefix == "twenty" {
            let is_teens = matches!(
                year_suffix,
                "ten"
                    | "eleven"
                    | "twelve"
                    | "thirteen"
                    | "fourteen"
                    | "fifteen"
                    | "sixteen"
                    | "seventeen"
                    | "eighteen"
                    | "nineteen"
            );
            if is_teens {
                return parse_year_number(input).map(|y| y.to_string());
            }
        }

        // For other century prefixes (eleven-nineteen), allow teens and tens
        let is_year_suffix = matches!(
            year_suffix,
            "ten"
                | "eleven"
                | "twelve"
                | "thirteen"
                | "fourteen"
                | "fifteen"
                | "sixteen"
                | "seventeen"
                | "eighteen"
                | "nineteen"
                | "twenty"
                | "thirty"
                | "forty"
                | "fifty"
                | "sixty"
                | "seventy"
                | "eighty"
                | "ninety"
        );

        if is_year_suffix
            && matches!(
                century_prefix,
                "eleven"
                    | "twelve"
                    | "thirteen"
                    | "fourteen"
                    | "fifteen"
                    | "sixteen"
                    | "seventeen"
                    | "eighteen"
                    | "nineteen"
            )
        {
            return parse_year_number(input).map(|y| y.to_string());
        }
    }

    // "nineteen seventy six" style - 3+ words starting with century prefix
    if words.len() >= 3 {
        if matches!(
            words[0],
            "eleven"
                | "twelve"
                | "thirteen"
                | "fourteen"
                | "fifteen"
                | "sixteen"
                | "seventeen"
                | "eighteen"
                | "nineteen"
                | "twenty"
        ) {
            return parse_year_number(input).map(|y| y.to_string());
        }
    }

    None
}

/// Parse year number from spoken form
fn parse_year_number(input: &str) -> Option<i64> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    // Handle "two thousand and X" or "two thousand X"
    if input.starts_with("two thousand") {
        let rest = input
            .strip_prefix("two thousand")?
            .trim()
            .trim_start_matches("and ")
            .trim();

        if rest.is_empty() {
            return Some(2000);
        }

        let year_part = words_to_number(rest)? as i64;
        return Some(2000 + year_part);
    }

    // Handle "one thousand X" (like "one thousand eight" → 1008)
    if input.starts_with("one thousand") {
        let rest = input.strip_prefix("one thousand")?.trim();
        if rest.is_empty() {
            return Some(1000);
        }

        let year_part = words_to_number(rest)? as i64;
        return Some(1000 + year_part);
    }

    // Handle "nineteen X" or "twenty X" century patterns
    // "nineteen seventy six" → 1976
    // "twenty twelve" → 2012
    if words.len() >= 2 {
        let century = match words[0] {
            "nineteen" => Some(19),
            "twenty" => Some(20),
            "eighteen" => Some(18),
            "seventeen" => Some(17),
            "sixteen" => Some(16),
            "fifteen" => Some(15),
            "fourteen" => Some(14),
            "thirteen" => Some(13),
            "twelve" => Some(12),
            "eleven" => Some(11),
            _ => None,
        };

        if let Some(c) = century {
            let year_part = words[1..].join(" ");

            // Handle "oh X" pattern (nineteen oh five → 1905)
            if year_part.starts_with("oh ") || year_part.starts_with("o ") {
                let digit_part = year_part
                    .strip_prefix("oh ")
                    .or_else(|| year_part.strip_prefix("o "))?;
                let digit = words_to_number(digit_part)? as i64;
                return Some(c * 100 + digit);
            }

            // Parse the two-digit year part
            if let Some(yy) = words_to_number(&year_part).map(|n| n as i64) {
                if yy >= 0 && yy <= 99 {
                    return Some(c * 100 + yy);
                }
            }
        }
    }

    // Try parsing as a plain number (for years like 1665)
    // Only if it looks like a year (3-4 digits)
    if let Some(num) = words_to_number(input).map(|n| n as i64) {
        if num >= 100 && num <= 9999 {
            return Some(num);
        }
    }

    None
}

/// Find month name from input
fn find_month(word: &str) -> Option<&'static str> {
    for month in &MONTHS {
        if word == *month {
            return Some(month);
        }
    }
    None
}

/// Find the original casing of a month from the original words
fn find_original_month<'a, I>(orig_words: I, lower_month: &str) -> String
where
    I: Iterator<Item = &'a str>,
{
    for word in orig_words {
        if word.to_lowercase() == lower_month {
            return word.to_string();
        }
    }
    lower_month.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decades() {
        assert_eq!(parse("nineteen eighties"), Some("1980s".to_string()));
        assert_eq!(parse("nineteen nineties"), Some("1990s".to_string()));
    }

    #[test]
    fn test_years() {
        assert_eq!(parse("two thousand and twenty"), Some("2020".to_string()));
        assert_eq!(parse("nineteen ninety four"), Some("1994".to_string()));
        assert_eq!(parse("twenty twelve"), Some("2012".to_string()));
    }

    #[test]
    fn test_month_day() {
        assert_eq!(parse("january first"), Some("january 1".to_string()));
        assert_eq!(parse("june thirty"), Some("june 30".to_string()));
    }

    #[test]
    fn test_month_day_year() {
        assert_eq!(
            parse("july twenty fifth two thousand twelve"),
            Some("july 25 2012".to_string())
        );
    }

    #[test]
    fn test_day_of_month() {
        assert_eq!(
            parse("the fifteenth of january"),
            Some("15 january".to_string())
        );
    }

    #[test]
    fn test_quarter() {
        assert_eq!(
            parse("second quarter of twenty twenty two"),
            Some("Q2 2022".to_string())
        );
    }

    #[test]
    fn test_bc() {
        assert_eq!(parse("seven fifty b c"), Some("750BC".to_string()));
    }
}
