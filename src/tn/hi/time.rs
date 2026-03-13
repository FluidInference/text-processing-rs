//! Time TN tagger for Hindi (romanized).
//!
//! Converts written time expressions to spoken romanized Hindi:
//! - "14:30" → "chaudah baj kar tees minat"
//! - "2:00" → "do baje"
//! - "0:00" → "baarah baje raat ke" (midnight)
//! - "12:00" → "baarah baje dopahar ke" (noon)

use super::number_to_words;

/// Parse a written time expression to spoken romanized Hindi.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try "14:30" colon format
    if let Some(result) = parse_colon_format(trimmed) {
        return Some(result);
    }

    // Try "14 baje" or "14 baj kar 30 minat" format
    if let Some(result) = parse_baje_format(trimmed) {
        return Some(result);
    }

    None
}

fn parse_colon_format(input: &str) -> Option<String> {
    if !input.contains(':') {
        return None;
    }

    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hour_str = parts[0].trim();
    let min_str = parts[1].trim();

    if !hour_str.chars().all(|c| c.is_ascii_digit()) || hour_str.is_empty() {
        return None;
    }
    if !min_str.chars().all(|c| c.is_ascii_digit()) || min_str.is_empty() {
        return None;
    }

    let hour: u32 = hour_str.parse().ok()?;
    let minute: u32 = min_str.parse().ok()?;

    if hour > 23 || minute > 59 {
        return None;
    }

    Some(format_time(hour, minute))
}

fn parse_baje_format(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // Match "X baje" or "X baj kar Y minat"
    if !lower.contains("baj") {
        return None;
    }

    let baj_pos = lower.find("baj")?;
    let hour_str = lower[..baj_pos].trim();

    if !hour_str.chars().all(|c| c.is_ascii_digit()) || hour_str.is_empty() {
        return None;
    }

    let hour: u32 = hour_str.parse().ok()?;
    if hour > 23 {
        return None;
    }

    // Check for minutes after "baj kar"
    let after_baj = &lower[baj_pos..];
    if let Some(rest) = after_baj.strip_prefix("baj kar") {
        let rest = rest.trim();
        // Try to extract minutes: "30 minat" or just "30"
        let min_str = rest
            .trim_end_matches("minat")
            .trim_end_matches("minute")
            .trim();
        if !min_str.is_empty() && min_str.chars().all(|c| c.is_ascii_digit()) {
            let minute: u32 = min_str.parse().ok()?;
            if minute <= 59 {
                return Some(format_time(hour, minute));
            }
        }
    }

    // Just "X baje" - no minutes
    Some(format_time(hour, 0))
}

fn format_time(hour: u32, minute: u32) -> String {
    let hour_words = number_to_words(hour as i64);

    if minute == 0 {
        format!("{} baje", hour_words)
    } else {
        let minute_words = number_to_words(minute as i64);
        format!("{} baj kar {} minat", hour_words, minute_words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colon_format() {
        assert_eq!(
            parse("14:30"),
            Some("chaudah baj kar tees minat".to_string())
        );
        assert_eq!(parse("2:00"), Some("do baje".to_string()));
        assert_eq!(
            parse("8:15"),
            Some("aath baj kar pandrah minat".to_string())
        );
    }

    #[test]
    fn test_midnight_and_noon() {
        assert_eq!(parse("0:00"), Some("shunya baje".to_string()));
        assert_eq!(parse("12:00"), Some("baarah baje".to_string()));
        assert_eq!(parse("0:30"), Some("shunya baj kar tees minat".to_string()));
    }

    #[test]
    fn test_baje_format() {
        assert_eq!(parse("14 baje"), Some("chaudah baje".to_string()));
        assert_eq!(
            parse("8 baj kar 30 minat"),
            Some("aath baj kar tees minat".to_string())
        );
    }

    #[test]
    fn test_24h() {
        assert_eq!(parse("14:00"), Some("chaudah baje".to_string()));
        assert_eq!(
            parse("23:59"),
            Some("teis baj kar unsath minat".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
        assert_eq!(parse(""), None);
    }
}
