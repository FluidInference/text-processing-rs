//! Time TN tagger for French.
//!
//! Converts written time expressions to spoken French:
//! - "14:30" → "quatorze heures trente"
//! - "14h30" → "quatorze heures trente"
//! - "2:00" → "deux heures"
//! - "0:00" → "minuit"
//! - "12:00" → "midi"

use super::number_to_words;

/// Parse a written time expression to spoken French.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try "14h30" or "14h" format (French convention)
    if let Some(result) = parse_h_format(trimmed) {
        return Some(result);
    }

    // Try "14:30" format
    if let Some(result) = parse_colon_format(trimmed) {
        return Some(result);
    }

    None
}

fn parse_h_format(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // Find 'h' separator
    let h_pos = lower.find('h')?;
    let hour_str = &lower[..h_pos];
    let min_str = lower[h_pos + 1..].trim();

    if !hour_str.chars().all(|c| c.is_ascii_digit()) || hour_str.is_empty() {
        return None;
    }

    let hour: u32 = hour_str.parse().ok()?;
    if hour > 23 {
        return None;
    }

    let minute: u32 = if min_str.is_empty() {
        0
    } else {
        if !min_str.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let m: u32 = min_str.parse().ok()?;
        if m > 59 {
            return None;
        }
        m
    };

    Some(format_time(hour, minute))
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
        return "minuit".to_string();
    }
    if hour == 12 && minute == 0 {
        return "midi".to_string();
    }
    if hour == 0 {
        return format!("minuit {}", number_to_words(minute as i64));
    }
    if hour == 12 {
        return format!("midi {}", number_to_words(minute as i64));
    }

    let hour_words = number_to_words(hour as i64);

    if minute == 0 {
        format!("{} heures", hour_words)
    } else {
        format!(
            "{} heures {}",
            hour_words,
            number_to_words(minute as i64)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h_format() {
        assert_eq!(
            parse("14h30"),
            Some("quatorze heures trente".to_string())
        );
        assert_eq!(parse("14h"), Some("quatorze heures".to_string()));
        assert_eq!(parse("8h15"), Some("huit heures quinze".to_string()));
    }

    #[test]
    fn test_colon_format() {
        assert_eq!(
            parse("14:30"),
            Some("quatorze heures trente".to_string())
        );
        assert_eq!(parse("2:00"), Some("deux heures".to_string()));
    }

    #[test]
    fn test_special_hours() {
        assert_eq!(parse("0:00"), Some("minuit".to_string()));
        assert_eq!(parse("12:00"), Some("midi".to_string()));
        assert_eq!(parse("0:30"), Some("minuit trente".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
    }
}
