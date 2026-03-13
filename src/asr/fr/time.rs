//! Time tagger for French.
//!
//! Converts spoken French time expressions to written form:
//! - "quatorze heures trente" → "14 h 30"
//! - "midi" → "12 h"
//! - "minuit" → "0 h"
//! - "huit heures du soir" → "20 h"
//! - "midi moins le quart" → "11 h 45"

use super::cardinal::words_to_number;

/// Parse spoken French time expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Try "moins" patterns first (subtractive)
    if let Some(result) = parse_moins_pattern(&input_lower) {
        return Some(result);
    }

    // Special base times
    if input_lower.starts_with("midi") {
        return parse_midi_pattern(&input_lower);
    }
    if input_lower.starts_with("minuit") {
        return parse_minuit_pattern(&input_lower);
    }

    // Standard pattern: "X heures Y" with modifiers
    if let Some(result) = parse_heures_pattern(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse "midi" patterns
fn parse_midi_pattern(input: &str) -> Option<String> {
    if input == "midi" {
        return Some("12 h".to_string());
    }
    // "midi moins le quart" → 11:45
    if input == "midi moins le quart" {
        return Some("11 h 45".to_string());
    }
    // "midi moins X" → 12 - X
    if let Some(rest) = input.strip_prefix("midi moins ") {
        let subtract = words_to_number(rest)? as i64;
        let minutes = 60 - subtract;
        return Some(format!("11 h {:02}", minutes));
    }
    None
}

/// Parse "minuit" patterns
fn parse_minuit_pattern(input: &str) -> Option<String> {
    if input == "minuit" {
        return Some("0 h".to_string());
    }
    // "minuit X" → 0:X
    if let Some(rest) = input.strip_prefix("minuit ") {
        let minutes = words_to_number(rest)? as i64;
        if minutes > 59 {
            return None;
        }
        return Some(format!("0 h {:02}", minutes));
    }
    None
}

/// Parse "X heures moins Y" patterns
fn parse_moins_pattern(input: &str) -> Option<String> {
    // "X heures moins le quart" → X-1:45
    if let Some(hour_part) = input.strip_suffix(" heures moins le quart") {
        let hour = words_to_number(hour_part)? as i64;
        let actual_hour = if hour > 1 { hour - 1 } else { 23 };
        return Some(format!("{} h 45", actual_hour));
    }

    // "X heures moins Y"
    if let Some((before, after)) = input.split_once(" heures moins ") {
        let hour = words_to_number(before)? as i64;
        let subtract = words_to_number(after)? as i64;
        let actual_hour = if hour > 1 { hour - 1 } else { 23 };
        let minutes = 60 - subtract;
        return Some(format!("{} h {:02}", actual_hour, minutes));
    }

    None
}

/// Parse "X heures Y" pattern
fn parse_heures_pattern(input: &str) -> Option<String> {
    // Remove time-of-day modifiers
    let cleaned = input
        .replace(" du matin", "")
        .replace(" du soir", "")
        .replace(" de l'après-midi", "");

    let add_12 = input.contains(" du soir") || input.contains(" de l'après-midi");

    // Pattern: "X heures et demie" → X:30
    if let Some(hour_part) = cleaned.strip_suffix(" heures et demie") {
        let mut hour = words_to_number(hour_part)? as i64;
        if add_12 && hour < 12 {
            hour += 12;
        }
        if hour > 23 {
            return None;
        }
        return Some(format!("{} h 30", hour));
    }

    // Pattern: "X heures et trois quarts" → X:45
    if let Some(hour_part) = cleaned.strip_suffix(" heures et trois quarts") {
        let mut hour = words_to_number(hour_part)? as i64;
        if add_12 && hour < 12 {
            hour += 12;
        }
        if hour > 23 {
            return None;
        }
        return Some(format!("{} h 45", hour));
    }

    // Pattern: "X heures Y"
    if let Some((hour_part, minute_part)) = cleaned.split_once(" heures ") {
        let mut hour = words_to_number(hour_part)? as i64;
        if add_12 && hour < 12 {
            hour += 12;
        }
        if hour > 23 {
            return None;
        }

        let minute = words_to_number(minute_part)? as i64;
        if minute > 59 {
            return None;
        }

        return Some(format!("{} h {:02}", hour, minute));
    }

    // Pattern: just "X heures" (no minutes)
    if let Some(hour_part) = cleaned.strip_suffix(" heures") {
        let mut hour = words_to_number(hour_part)? as i64;
        if add_12 && hour < 12 {
            hour += 12;
        }
        if hour > 23 {
            return None;
        }
        return Some(format!("{} h", hour));
    }

    // Singular: "une heure"
    if cleaned.ends_with(" heure") {
        let hour_part = cleaned.strip_suffix(" heure")?;
        let mut hour = if hour_part == "une" {
            1
        } else {
            words_to_number(hour_part)? as i64
        };
        if add_12 && hour < 12 {
            hour += 12;
        }
        if hour > 23 {
            return None;
        }
        return Some(format!("{} h", hour));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_times() {
        assert_eq!(parse("midi"), Some("12 h".to_string()));
        assert_eq!(parse("minuit"), Some("0 h".to_string()));
    }

    #[test]
    fn test_heures_pattern() {
        assert_eq!(parse("quatorze heures trente"), Some("14 h 30".to_string()));
        assert_eq!(parse("quinze heures"), Some("15 h".to_string()));
        assert_eq!(parse("neuf heures dix"), Some("9 h 10".to_string()));
    }

    #[test]
    fn test_time_of_day() {
        assert_eq!(parse("huit heures du matin"), Some("8 h".to_string()));
        assert_eq!(parse("huit heures du soir"), Some("20 h".to_string()));
    }

    #[test]
    fn test_special_minutes() {
        assert_eq!(parse("onze heures et demie"), Some("11 h 30".to_string()));
        assert_eq!(parse("midi moins le quart"), Some("11 h 45".to_string()));
    }

    #[test]
    fn test_singular() {
        assert_eq!(parse("une heure"), Some("1 h".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("vingt-cinq heures"), None); // > 23
        assert_eq!(parse("hello"), None);
    }
}
