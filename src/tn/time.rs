//! Time TN tagger.
//!
//! Converts written time expressions to spoken form:
//! - "2:30" → "two thirty"
//! - "2:05" → "two oh five"
//! - "2:00 PM" → "two p m"
//! - "14:00" → "two p m"
//! - "2:30 AM" → "two thirty a m"

use super::number_to_words;

/// Parse a written time expression to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Split off AM/PM suffix if present
    let (time_part, period) = extract_period(trimmed);

    // Must contain a colon
    if !time_part.contains(':') {
        return None;
    }

    let parts: Vec<&str> = time_part.splitn(2, ':').collect();
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

    // Convert 24h to 12h if no period specified and hour > 12
    let (spoken_hour, inferred_period) = if period.is_none() && hour > 12 {
        (hour - 12, Some("p m"))
    } else if period.is_none() && hour == 0 {
        (12, Some("a m"))
    } else if period.is_none() && hour == 12 {
        (12, None)
    } else {
        (hour, None)
    };

    let hour_words = number_to_words(spoken_hour as i64);

    let result = if minute == 0 {
        // "2:00" → "two" or "two p m"
        hour_words.clone()
    } else if minute < 10 {
        // "2:05" → "two oh five"
        format!("{} oh {}", hour_words, number_to_words(minute as i64))
    } else {
        // "2:30" → "two thirty"
        format!("{} {}", hour_words, number_to_words(minute as i64))
    };

    // Append period
    let final_period = period.or(inferred_period);
    if let Some(p) = final_period {
        Some(format!("{} {}", result, p))
    } else {
        Some(result)
    }
}

/// Extract AM/PM period from end of string.
fn extract_period(input: &str) -> (&str, Option<&str>) {
    let upper = input.to_uppercase();
    for (suffix, spoken) in &[
        (" PM", "p m"),
        (" AM", "a m"),
        (" P.M.", "p m"),
        (" A.M.", "a m"),
        (" P.M", "p m"),
        (" A.M", "a m"),
        (" pm", "p m"),
        (" am", "a m"),
    ] {
        if upper.ends_with(&suffix.to_uppercase()) {
            let time_part = &input[..input.len() - suffix.len()];
            return (time_part.trim(), Some(spoken));
        }
    }
    (input, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_time() {
        assert_eq!(parse("2:30"), Some("two thirty".to_string()));
        assert_eq!(parse("2:05"), Some("two oh five".to_string()));
        assert_eq!(parse("12:00"), Some("twelve".to_string()));
    }

    #[test]
    fn test_with_period() {
        assert_eq!(parse("2:30 PM"), Some("two thirty p m".to_string()));
        assert_eq!(parse("2:00 PM"), Some("two p m".to_string()));
        assert_eq!(parse("8:15 AM"), Some("eight fifteen a m".to_string()));
    }

    #[test]
    fn test_24h() {
        assert_eq!(parse("14:00"), Some("two p m".to_string()));
        assert_eq!(parse("0:00"), Some("twelve a m".to_string()));
        assert_eq!(parse("23:59"), Some("eleven fifty nine p m".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
    }
}
