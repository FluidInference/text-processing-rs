//! Time tagger for French.
//!
//! Converts spoken French time expressions to written form:
//! - "quatorze heures trente" → "14:30"
//! - "midi" → "12:00"
//! - "minuit" → "00:00"
//! - "quinze heures" → "15:00"

use super::cardinal::words_to_number;

/// Parse spoken French time expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Special cases
    if input_lower == "midi" {
        return Some("12:00".to_string());
    }
    if input_lower == "minuit" {
        return Some("00:00".to_string());
    }

    // Standard pattern: "X heures Y" or just "X heures"
    if let Some(result) = parse_heures_pattern(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse "X heures Y" pattern
fn parse_heures_pattern(input: &str) -> Option<String> {
    // Pattern: "X heures Y" or "X heure Y" (singular)
    if let Some((hour_part, minute_part)) = input.split_once(" heures ") {
        let hour = words_to_number(hour_part)? as i64;
        if hour > 23 {
            return None;
        }

        let minute = if minute_part.is_empty() {
            0
        } else {
            words_to_number(minute_part)? as i64
        };
        if minute > 59 {
            return None;
        }

        return Some(format!("{:02}:{:02}", hour, minute));
    }

    // Pattern: just "X heures" (no minutes)
    if input.ends_with(" heures") {
        let hour_part = input.strip_suffix(" heures")?;
        let hour = words_to_number(hour_part)? as i64;
        if hour > 23 {
            return None;
        }
        return Some(format!("{:02}:00", hour));
    }

    // Singular: "une heure"
    if input.ends_with(" heure") {
        let hour_part = input.strip_suffix(" heure")?;
        let hour = if hour_part == "une" {
            1
        } else {
            words_to_number(hour_part)? as i64
        };
        if hour > 23 {
            return None;
        }
        return Some(format!("{:02}:00", hour));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_times() {
        assert_eq!(parse("midi"), Some("12:00".to_string()));
        assert_eq!(parse("minuit"), Some("00:00".to_string()));
    }

    #[test]
    fn test_heures_pattern() {
        assert_eq!(parse("quatorze heures trente"), Some("14:30".to_string()));
        assert_eq!(parse("quinze heures"), Some("15:00".to_string()));
        assert_eq!(parse("neuf heures dix"), Some("09:10".to_string()));
    }

    #[test]
    fn test_singular() {
        assert_eq!(parse("une heure"), Some("01:00".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("vingt-cinq heures"), None); // > 23
        assert_eq!(parse("hello"), None);
    }
}
