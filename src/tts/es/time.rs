//! Time TN tagger for Spanish.
//!
//! Converts written time expressions to spoken Spanish:
//! - "14:30" → "catorce treinta"
//! - "2:00" → "dos en punto"
//! - "0:00" → "medianoche"
//! - "12:00" → "mediodia"

use super::number_to_words;

/// Parse a written time expression to spoken Spanish.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try "14:30" format
    if let Some(result) = parse_colon_format(trimmed) {
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

fn format_time(hour: u32, minute: u32) -> String {
    // Special cases
    if hour == 0 && minute == 0 {
        return "medianoche".to_string();
    }
    if hour == 12 && minute == 0 {
        return "mediodia".to_string();
    }

    let hour_words = number_to_words(hour as i64);

    if minute == 0 {
        format!("{} en punto", hour_words)
    } else {
        format!("{} {}", hour_words, number_to_words(minute as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colon_format() {
        assert_eq!(parse("14:30"), Some("catorce treinta".to_string()));
        assert_eq!(parse("8:15"), Some("ocho quince".to_string()));
        assert_eq!(parse("2:00"), Some("dos en punto".to_string()));
    }

    #[test]
    fn test_special_hours() {
        assert_eq!(parse("0:00"), Some("medianoche".to_string()));
        assert_eq!(parse("12:00"), Some("mediodia".to_string()));
        assert_eq!(parse("0:30"), Some("cero treinta".to_string()));
    }

    #[test]
    fn test_24h() {
        assert_eq!(parse("14:00"), Some("catorce en punto".to_string()));
        assert_eq!(parse("23:59"), Some("veintitres cincuenta y nueve".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
    }
}
