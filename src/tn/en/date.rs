//! Date TN tagger.
//!
//! Converts written date expressions to spoken form:
//! - "January 5, 2025" → "january fifth twenty twenty five"
//! - "January 5" → "january fifth"
//! - "1/5/2025" → "january fifth twenty twenty five"
//! - "1980s" → "nineteen eighties"

use super::number_to_words;

const MONTHS: &[(&str, &str)] = &[
    ("january", "january"),
    ("february", "february"),
    ("march", "march"),
    ("april", "april"),
    ("may", "may"),
    ("june", "june"),
    ("july", "july"),
    ("august", "august"),
    ("september", "september"),
    ("october", "october"),
    ("november", "november"),
    ("december", "december"),
];

const MONTH_NUMBERS: &[(&str, u32)] = &[
    ("january", 1),
    ("february", 2),
    ("march", 3),
    ("april", 4),
    ("may", 5),
    ("june", 6),
    ("july", 7),
    ("august", 8),
    ("september", 9),
    ("october", 10),
    ("november", 11),
    ("december", 12),
];

/// Parse a written date to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "nineteen eighties"
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try "Month Day, Year" or "Month Day"
    if let Some(result) = parse_month_day_year(trimmed) {
        return Some(result);
    }

    // Try numeric: "1/5/2025" or "01/05/2025"
    if let Some(result) = parse_numeric_date(trimmed) {
        return Some(result);
    }

    None
}

/// Pluralize the final word of a spelled decade into its `-s`/`-ies` form,
/// e.g. `"eighty"` → `"eighties"`, `"hundred"` → `"hundreds"`. Returns `None`
/// for any word that is not a round decade term (so `"zero"` from `"00s"` is
/// rejected rather than becoming `"zeros"`).
fn pluralize_decade_word(word: &str) -> Option<&'static str> {
    let plural = match word {
        "ten" => "tens",
        "twenty" => "twenties",
        "thirty" => "thirties",
        "forty" => "forties",
        "fifty" => "fifties",
        "sixty" => "sixties",
        "seventy" => "seventies",
        "eighty" => "eighties",
        "ninety" => "nineties",
        "hundred" => "hundreds",
        "thousand" => "thousands",
        _ => return None,
    };
    Some(plural)
}

/// Parse decade: "1980s" → "nineteen eighties", "2000s" → "two thousands",
/// "90s"/"'90s" → "nineties". A leading apostrophe (elided century) is
/// accepted; two-digit forms read as the century-less tens word, which the
/// spoken form drops anyway ("'90s" and "1990s" are both "nineties").
fn parse_decade(input: &str) -> Option<String> {
    // Strip the trailing plural `s`, then an optional leading apostrophe.
    // `is_split_punct` keeps `'` attached to the token, so we handle it here.
    let s = input.strip_suffix('s')?;
    let s = s.strip_prefix('\'').unwrap_or(s);

    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Two-digit decade ("90s", "'90s") → tens word pluralized. "00s" maps to
    // "zero", which `pluralize_decade_word` rejects, so it is left untouched.
    if s.len() == 2 {
        let tens: u32 = s.parse().ok()?;
        if tens % 10 != 0 {
            return None; // "95s" is not a valid decade
        }
        let word = number_to_words(tens as i64);
        return pluralize_decade_word(&word).map(str::to_string);
    }

    // Four-digit decade ("1980s") → year-style reading, last word pluralized.
    if s.len() != 4 {
        return None;
    }

    let year: u32 = s.parse().ok()?;
    if year < 1000 {
        return None;
    }

    if year % 10 != 0 {
        return None; // "1985s" is not a valid decade
    }

    let year_words = verbalize_year(year)?;
    let words: Vec<&str> = year_words.split_whitespace().collect();
    let last = *words.last()?;
    let plural = pluralize_decade_word(last)?;

    if words.len() == 1 {
        Some(plural.to_string())
    } else {
        let prefix = words[..words.len() - 1].join(" ");
        Some(format!("{} {}", prefix, plural))
    }
}

/// Parse "Month Day, Year" or "Month Day" formats.
fn parse_month_day_year(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // Find the month
    let mut month_name = None;
    let mut rest = "";
    for &(name, spoken) in MONTHS {
        if let Some(r) = lower.strip_prefix(name) {
            if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
                month_name = Some(spoken);
                rest = r.trim_start_matches(|c: char| c == ' ' || c == ',');
                break;
            }
        }
    }

    let month = month_name?;

    if rest.is_empty() {
        return None; // Just a month name alone
    }

    // Parse day, possibly followed by comma and year
    let (day_str, year_part) = if let Some(comma_pos) = rest.find(',') {
        (&rest[..comma_pos], Some(rest[comma_pos + 1..].trim()))
    } else {
        // Could be "January 5 2025" or "January 5 2025." (space-separated, optional trailing punct)
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        if parts.len() == 2 && parts[0].chars().all(|c| c.is_ascii_digit()) {
            let year_clean =
                parts[1].trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
            if year_clean.chars().all(|c| c.is_ascii_digit()) && year_clean.len() == 4 {
                (parts[0], Some(year_clean))
            } else {
                (rest, None)
            }
        } else {
            (rest, None)
        }
    };

    let day_str = day_str.trim();
    // Strip ordinal suffix from day if present (e.g., "5th")
    let day_digits = day_str
        .trim_end_matches("st")
        .trim_end_matches("nd")
        .trim_end_matches("rd")
        .trim_end_matches("th");

    if !day_digits.chars().all(|c| c.is_ascii_digit()) || day_digits.is_empty() {
        return None;
    }

    let day: u32 = day_digits.parse().ok()?;
    if day == 0 || day > 31 {
        return None;
    }

    let day_ordinal = ordinal_word(day);

    if let Some(year_str) = year_part {
        // Strip trailing sentence punctuation (e.g. "2026." → "2026") so that
        // "March 8, 2026." correctly parses the year instead of dropping it.
        let year_str = year_str
            .trim()
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if !year_str.is_empty() && year_str.chars().all(|c| c.is_ascii_digit()) {
            let year: u32 = year_str.parse().ok()?;
            let year_words = verbalize_year(year)?;
            return Some(format!("{} {} {}", month, day_ordinal, year_words));
        }
    }

    Some(format!("{} {}", month, day_ordinal))
}

/// Parse numeric date: "1/5/2025" or "01/05/2025" (M/D/Y).
fn parse_numeric_date(input: &str) -> Option<String> {
    // Support both / and - separators
    let sep = if input.contains('/') {
        '/'
    } else if input.contains('-') && input.chars().filter(|c| *c == '-').count() == 2 {
        '-'
    } else {
        return None;
    };

    let parts: Vec<&str> = input.splitn(3, sep).collect();
    if parts.len() != 3 {
        return None;
    }

    // All parts must be digits
    if !parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }

    let month_num: u32 = parts[0].parse().ok()?;
    let day: u32 = parts[1].parse().ok()?;
    let year: u32 = parts[2].parse().ok()?;

    if month_num == 0 || month_num > 12 || day == 0 || day > 31 {
        return None;
    }

    let month_name = MONTH_NUMBERS.iter().find(|(_, n)| *n == month_num)?.0;

    let day_ordinal = ordinal_word(day);
    let year_words = verbalize_year(year)?;

    Some(format!("{} {} {}", month_name, day_ordinal, year_words))
}

/// Verbalize a year.
///
/// - 2025 → "twenty twenty five"
/// - 2000 → "two thousand"
/// - 2001 → "two thousand one"
/// - 1990 → "nineteen ninety"
/// - 1900 → "nineteen hundred"
/// - 800 → "eight hundred"
pub fn verbalize_year(year: u32) -> Option<String> {
    if year == 0 {
        return Some("zero".to_string());
    }

    if year < 100 {
        return Some(number_to_words(year as i64));
    }

    // Years 100-999
    if year < 1000 {
        return Some(number_to_words(year as i64));
    }

    let century = year / 100;
    let remainder = year % 100;

    if remainder == 0 {
        // 2000 → "two thousand", 1900 → "nineteen hundred"
        if year % 1000 == 0 {
            let thousands = year / 1000;
            return Some(format!("{} thousand", number_to_words(thousands as i64)));
        }
        return Some(format!("{} hundred", number_to_words(century as i64)));
    }

    // 2001-2009: "two thousand one" through "two thousand nine"
    if century == 20 && remainder < 10 {
        return Some(format!(
            "two thousand {}",
            number_to_words(remainder as i64)
        ));
    }

    // Standard: split into two halves
    // 2025 → "twenty" + "twenty five"
    // 1990 → "nineteen" + "ninety"
    // 1901 → "nineteen" + "oh one"
    let first_half = number_to_words(century as i64);
    let second_half = if remainder < 10 {
        format!("oh {}", number_to_words(remainder as i64))
    } else {
        number_to_words(remainder as i64)
    };

    Some(format!("{} {}", first_half, second_half))
}

/// Convert a day number to its ordinal word form.
fn ordinal_word(n: u32) -> String {
    match n {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        4 => "fourth".to_string(),
        5 => "fifth".to_string(),
        6 => "sixth".to_string(),
        7 => "seventh".to_string(),
        8 => "eighth".to_string(),
        9 => "ninth".to_string(),
        10 => "tenth".to_string(),
        11 => "eleventh".to_string(),
        12 => "twelfth".to_string(),
        13 => "thirteenth".to_string(),
        14 => "fourteenth".to_string(),
        15 => "fifteenth".to_string(),
        16 => "sixteenth".to_string(),
        17 => "seventeenth".to_string(),
        18 => "eighteenth".to_string(),
        19 => "nineteenth".to_string(),
        20 => "twentieth".to_string(),
        21 => "twenty first".to_string(),
        22 => "twenty second".to_string(),
        23 => "twenty third".to_string(),
        24 => "twenty fourth".to_string(),
        25 => "twenty fifth".to_string(),
        26 => "twenty sixth".to_string(),
        27 => "twenty seventh".to_string(),
        28 => "twenty eighth".to_string(),
        29 => "twenty ninth".to_string(),
        30 => "thirtieth".to_string(),
        31 => "thirty first".to_string(),
        _ => {
            // Fallback for larger numbers (shouldn't happen for days)
            let cardinal = number_to_words(n as i64);
            format!("{}th", cardinal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_day() {
        assert_eq!(parse("January 5"), Some("january fifth".to_string()));
        assert_eq!(
            parse("December 25"),
            Some("december twenty fifth".to_string())
        );
    }

    #[test]
    fn test_month_day_year() {
        assert_eq!(
            parse("January 5, 2025"),
            Some("january fifth twenty twenty five".to_string())
        );
        assert_eq!(
            parse("July 4, 1776"),
            Some("july fourth seventeen seventy six".to_string())
        );
    }

    #[test]
    fn test_decade() {
        assert_eq!(parse("1980s"), Some("nineteen eighties".to_string()));
        assert_eq!(parse("2000s"), Some("two thousands".to_string()));
        assert_eq!(parse("1990s"), Some("nineteen nineties".to_string()));
        assert_eq!(parse("1900s"), Some("nineteen hundreds".to_string()));
        assert_eq!(parse("2010s"), Some("twenty tens".to_string()));
    }

    #[test]
    fn test_two_digit_and_apostrophe_decade() {
        // Two-digit and apostrophe-elided decades read as the tens word.
        assert_eq!(parse("90s"), Some("nineties".to_string()));
        assert_eq!(parse("'90s"), Some("nineties".to_string()));
        assert_eq!(parse("80s"), Some("eighties".to_string()));
        assert_eq!(parse("'20s"), Some("twenties".to_string()));
        assert_eq!(parse("10s"), Some("tens".to_string()));
        assert_eq!(parse("'1980s"), Some("nineteen eighties".to_string()));
    }

    #[test]
    fn test_invalid_decade() {
        assert_eq!(parse("95s"), None); // not a round decade
        assert_eq!(parse("00s"), None); // "zero" is not a decade word
        assert_eq!(parse("'00s"), None);
        assert_eq!(parse("5s"), None); // single digit
        assert_eq!(parse("'s"), None); // no digits
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("1/5/2025"),
            Some("january fifth twenty twenty five".to_string())
        );
        assert_eq!(
            parse("12/25/2000"),
            Some("december twenty fifth two thousand".to_string())
        );
    }

    #[test]
    fn test_year_verbalization() {
        assert_eq!(verbalize_year(2025), Some("twenty twenty five".to_string()));
        assert_eq!(verbalize_year(2000), Some("two thousand".to_string()));
        assert_eq!(verbalize_year(2001), Some("two thousand one".to_string()));
        assert_eq!(verbalize_year(1990), Some("nineteen ninety".to_string()));
        assert_eq!(verbalize_year(1900), Some("nineteen hundred".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
