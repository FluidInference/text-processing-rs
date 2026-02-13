//! Time tagger.
//!
//! Converts spoken time expressions to written form:
//! - "two thirty" → "02:30"
//! - "two thirty pm" → "02:30 p.m."
//! - "quarter past one" → "01:15"
//! - "half past three" → "03:30"

use super::cardinal::words_to_number;

/// Parse spoken time expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let original = input.trim();
    let input_lower = original.to_lowercase();

    // Extract period (am/pm) and timezone if present, preserving original casing
    let (time_part, period, timezone) = extract_period_and_tz(original, &input_lower);

    // Try special patterns first
    if let Some(result) = parse_quarter_half(&time_part, &period, &timezone) {
        return Some(result);
    }

    if let Some(result) = parse_oclock(&time_part, &period, &timezone) {
        return Some(result);
    }

    if let Some(result) = parse_to_pattern(&time_part, &period, &timezone) {
        return Some(result);
    }

    if let Some(result) = parse_standard_time(&time_part, &period, &timezone) {
        return Some(result);
    }

    None
}

/// Extract am/pm period and timezone from input, preserving original casing
fn extract_period_and_tz(original: &str, input_lower: &str) -> (String, String, String) {
    let mut time_part = input_lower.to_string();
    let mut period = String::new();
    let mut timezone = String::new();

    // Check for timezone suffixes (match on lowercase, extract from original)
    let tz_patterns = ["g m t", "gmt", "e s t", "est", "p s t", "pst", "c s t", "cst", "m s t", "mst"];
    for tz in &tz_patterns {
        if time_part.ends_with(tz) {
            // Extract timezone from original to preserve casing
            let tz_start = original.len() - tz.len();
            timezone = original[tz_start..].replace(" ", "");
            time_part = time_part[..time_part.len() - tz.len()].trim().to_string();
            break;
        }
    }

    // Check for period (am/pm) - match on lowercase, preserve original casing
    let period_patterns = [
        (" a m", 4),      // " a m" = 4 chars
        (" am", 3),       // " am" = 3 chars
        (" p m", 4),
        (" pm", 3),
        (" in the morning", 16),
        (" in the afternoon", 18),
        (" in the evening", 15),
    ];

    for (pattern, len) in &period_patterns {
        if time_part.ends_with(pattern) {
            // Get the suffix from original to check casing
            let suffix_start = original.len().saturating_sub(timezone.len() + if timezone.is_empty() { 0 } else {
                // Account for spaces in original timezone
                tz_patterns.iter().find(|p| p.replace(" ", "") == timezone).map(|p| p.len()).unwrap_or(timezone.len())
            });
            let time_original = if timezone.is_empty() { original } else { &original[..suffix_start] }.trim();

            // Check if AM/PM is uppercase in original
            let period_start = time_original.len().saturating_sub(*len);
            let orig_suffix = &time_original[period_start..];

            period = format_period_with_case(orig_suffix, *pattern);
            time_part = time_part[..time_part.len() - len].trim().to_string();
            break;
        }
    }

    (time_part, period, timezone)
}

/// Format period (AM/PM) preserving original casing
fn format_period_with_case(orig_suffix: &str, pattern: &str) -> String {
    let orig_upper = orig_suffix.to_uppercase();

    // Check if original was uppercase
    if pattern.contains("in the") {
        // "in the morning/afternoon/evening" always becomes a.m./p.m.
        if pattern.contains("morning") {
            return "a.m.".to_string();
        } else {
            return "p.m.".to_string();
        }
    }

    // Check if it looks uppercase (A M, AM, P M, PM)
    let is_uppercase = orig_suffix.trim().chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase());

    if is_uppercase {
        if orig_upper.contains('A') {
            "A.M.".to_string()
        } else {
            "P.M.".to_string()
        }
    } else {
        if pattern.contains('a') {
            "a.m.".to_string()
        } else {
            "p.m.".to_string()
        }
    }
}

/// Format time output with period and timezone
fn format_time(hour: i64, minute: i64, period: &str, timezone: &str) -> String {
    let mut result = format!("{:02}:{:02}", hour, minute);

    if !period.is_empty() {
        result.push(' ');
        result.push_str(period);
    }

    if !timezone.is_empty() {
        result.push(' ');
        result.push_str(timezone);
    }

    result
}

/// Parse "quarter past X" and "half past X" patterns
fn parse_quarter_half(input: &str, period: &str, timezone: &str) -> Option<String> {
    if input.starts_with("quarter past ") {
        let hour_part = input.trim_start_matches("quarter past ");
        let hour = words_to_number(hour_part)? as i64;
        return Some(format_time(hour, 15, period, timezone));
    }

    if input.starts_with("half past ") {
        let hour_part = input.trim_start_matches("half past ");
        let hour = words_to_number(hour_part)? as i64;
        return Some(format_time(hour, 30, period, timezone));
    }

    None
}

/// Parse "X o'clock" pattern
fn parse_oclock(input: &str, period: &str, timezone: &str) -> Option<String> {
    if input.ends_with(" o'clock") || input.ends_with(" oclock") {
        let hour_part = input
            .trim_end_matches(" o'clock")
            .trim_end_matches(" oclock");
        let hour = words_to_number(hour_part)? as i64;
        return Some(format_time(hour, 0, period, timezone));
    }

    None
}

/// Parse "X to Y" pattern (e.g., "quarter to one" = 12:45)
fn parse_to_pattern(input: &str, period: &str, timezone: &str) -> Option<String> {
    if input.starts_with("quarter to ") {
        let hour_part = input.trim_start_matches("quarter to ");
        let hour = words_to_number(hour_part)? as i64;
        let prev_hour = if hour == 1 { 12 } else { hour - 1 };
        return Some(format_time(prev_hour, 45, period, timezone));
    }

    // "X min to Y" or "X minutes to Y"
    if input.contains(" to ") {
        let parts: Vec<&str> = input.split(" to ").collect();
        if parts.len() == 2 {
            let min_part = parts[0]
                .trim_end_matches(" min")
                .trim_end_matches(" mins")
                .trim_end_matches(" minute")
                .trim_end_matches(" minutes");
            let minutes_before = words_to_number(min_part)? as i64;
            let hour = words_to_number(parts[1])? as i64;
            let prev_hour = if hour == 1 { 12 } else { hour - 1 };
            let minute = 60 - minutes_before;
            return Some(format_time(prev_hour, minute, period, timezone));
        }
    }

    None
}

/// Parse standard "hour minute" time
fn parse_standard_time(input: &str, period: &str, timezone: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return None;
    }

    // Single word - only treat as time if there's a period (am/pm) or timezone
    // Otherwise "one" would be parsed as "01:00" instead of cardinal "1"
    if words.len() == 1 {
        if period.is_empty() && timezone.is_empty() {
            return None;
        }
        let hour = words_to_number(words[0])? as i64;
        if hour >= 1 && hour <= 24 {
            return Some(format_time(hour, 0, period, timezone));
        }
        return None;
    }

    // For multi-word time without am/pm, require the first word to be a simple hour (1-12)
    // This prevents "twenty one" from being parsed as "20:01"
    // Only single-word hour numbers are valid (e.g., "two", "twelve", not "twenty")
    let hour_word = words[0];
    let hour = parse_simple_hour(hour_word)?;

    // Without am/pm, only allow 1-12 as hours (clock hours)
    if period.is_empty() && timezone.is_empty() && (hour < 1 || hour > 12) {
        return None;
    }

    // Remaining words are the minute
    let minute_words = words[1..].join(" ");
    let minute = parse_minute(&minute_words)?;

    // Without am/pm, avoid matching patterns that look like historical years
    // e.g., "eleven fifty five" should be year 1155, not time 11:55
    // This applies when hour is 10-19 and minute forms a two-digit number
    if period.is_empty() && timezone.is_empty() {
        if hour >= 10 && hour <= 19 && minute >= 10 && minute <= 99 {
            return None;
        }
    }

    if minute >= 0 && minute < 60 {
        Some(format_time(hour, minute, period, timezone))
    } else {
        None
    }
}

/// Parse a simple hour word (one-twelve only)
fn parse_simple_hour(word: &str) -> Option<i64> {
    match word {
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        _ => None,
    }
}

/// Parse minute portion, handling "oh five" = 05, "thirty" = 30
/// Only accepts patterns that look like valid minutes
fn parse_minute(input: &str) -> Option<i64> {
    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return None;
    }

    // Handle "o X" or "oh X" pattern for single digit minutes
    if words.len() == 2 && (words[0] == "o" || words[0] == "oh") {
        let digit_word = words[1];
        let minute = words_to_number(digit_word).map(|n| n as i64)?;
        if minute >= 0 && minute <= 9 {
            return Some(minute);
        }
        return None;
    }

    // Single word: must be a valid minute word (not a sequence of digits)
    if words.len() == 1 {
        let minute = words_to_number(words[0]).map(|n| n as i64)?;
        if minute >= 0 && minute <= 59 {
            return Some(minute);
        }
        return None;
    }

    // Two words: must be tens + units compound (e.g., "forty five")
    // Reject patterns like "nine nine" which are digit sequences
    if words.len() == 2 {
        // First word must be a tens word (twenty, thirty, etc.)
        let is_tens = matches!(
            words[0],
            "twenty" | "thirty" | "forty" | "fifty"
        );
        if !is_tens {
            return None;
        }
        // Second word must be a units word
        let is_units = matches!(
            words[1],
            "one" | "two" | "three" | "four" | "five" | "six" | "seven" | "eight" | "nine"
        );
        if !is_units {
            return None;
        }
        let minute = words_to_number(input).map(|n| n as i64)?;
        if minute >= 0 && minute <= 59 {
            return Some(minute);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_time() {
        assert_eq!(parse("two thirty"), Some("02:30".to_string()));
        assert_eq!(parse("eight fifty one"), Some("08:51".to_string()));
        // Note: "eleven forty five" without am/pm is rejected to avoid
        // conflict with year patterns like "eleven fifty five" → 1155
        assert_eq!(parse("eleven forty five"), None);
        // But with am/pm it works
        assert_eq!(parse("eleven forty five a m"), Some("11:45 a.m.".to_string()));
    }

    #[test]
    fn test_with_period() {
        assert_eq!(parse("two p m"), Some("02:00 p.m.".to_string()));
        assert_eq!(parse("eleven fifty five p m"), Some("11:55 p.m.".to_string()));
        assert_eq!(parse("seven a m"), Some("07:00 a.m.".to_string()));
    }

    #[test]
    fn test_quarter_half() {
        assert_eq!(parse("quarter past one"), Some("01:15".to_string()));
        assert_eq!(parse("half past three"), Some("03:30".to_string()));
        assert_eq!(parse("half past twelve"), Some("12:30".to_string()));
    }

    #[test]
    fn test_quarter_to() {
        assert_eq!(parse("quarter to one"), Some("12:45".to_string()));
        assert_eq!(parse("quarter to twelve"), Some("11:45".to_string()));
    }

    #[test]
    fn test_oclock() {
        assert_eq!(parse("three o'clock"), Some("03:00".to_string()));
    }

    #[test]
    fn test_oh_minutes() {
        assert_eq!(parse("eight o six"), Some("08:06".to_string()));
        assert_eq!(parse("twelve oh five"), Some("12:05".to_string()));
    }

    #[test]
    fn test_with_timezone() {
        assert_eq!(parse("eight oclock g m t"), Some("08:00 gmt".to_string()));
        assert_eq!(parse("seven a m e s t"), Some("07:00 a.m. est".to_string()));
    }

    #[test]
    fn test_rejects_phone_like_input() {
        // These should NOT be parsed as time - they're phone numbers
        assert_eq!(parse("one two three one two three five six seven eight"), None);
        assert_eq!(parse("seven nine nine"), None);
    }
}
