//! Date tagger for French.
//!
//! Converts spoken French date expressions to written form:
//! - "cinq janvier deux mille vingt-cinq" → "5 janvier 2025"
//! - "premier janvier" → "1er janvier"
//! - "quatorze juillet" → "14 juillet"

use super::cardinal::words_to_number;

/// French month names
const MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// Parse spoken French date expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Try day + month + year pattern
    if let Some(result) = parse_day_month_year(&input_lower) {
        return Some(result);
    }

    // Try day + month pattern (no year)
    if let Some(result) = parse_day_month(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse "X month year" pattern
fn parse_day_month_year(input: &str) -> Option<String> {
    // Find month in the input
    for month in &MONTHS {
        if let Some(month_pos) = input.find(month) {
            let day_part = &input[..month_pos].trim();
            let after_month = &input[month_pos + month.len()..].trim();

            // Parse day
            let day_str = if day_part == &"premier" || day_part == &"première" {
                "1er".to_string()
            } else if let Some(day_num) = words_to_number(day_part) {
                (day_num as i64).to_string()
            } else {
                return None;
            };

            // Parse year if present
            if !after_month.is_empty() {
                let year = words_to_number(after_month)? as i64;
                return Some(format!("{} {} {}", day_str, month, year));
            } else {
                return Some(format!("{} {}", day_str, month));
            }
        }
    }

    None
}

/// Parse "X month" pattern (no year)
fn parse_day_month(input: &str) -> Option<String> {
    // Find month in the input
    for month in &MONTHS {
        if input.contains(month) {
            let parts: Vec<&str> = input.split(month).collect();
            if parts.len() == 2 && parts[1].trim().is_empty() {
                let day_part = parts[0].trim();

                // Parse day
                let day_str = if day_part == "premier" || day_part == "première" {
                    "1er".to_string()
                } else if let Some(day_num) = words_to_number(day_part) {
                    (day_num as i64).to_string()
                } else {
                    return None;
                };

                return Some(format!("{} {}", day_str, month));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_month_year() {
        assert_eq!(
            parse("cinq janvier deux mille vingt-cinq"),
            Some("5 janvier 2025".to_string())
        );
        assert_eq!(
            parse("quatorze juillet deux mille"),
            Some("14 juillet 2000".to_string())
        );
    }

    #[test]
    fn test_day_month() {
        assert_eq!(parse("quatorze juillet"), Some("14 juillet".to_string()));
        assert_eq!(
            parse("vingt-cinq décembre"),
            Some("25 décembre".to_string())
        );
    }

    #[test]
    fn test_premier() {
        assert_eq!(parse("premier janvier"), Some("1er janvier".to_string()));
        assert_eq!(
            parse("premier mai deux mille vingt"),
            Some("1er mai 2020".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("vingt"), None);
    }
}
