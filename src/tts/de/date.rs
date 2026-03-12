//! Date TN tagger for German.
//!
//! Converts written date expressions to spoken German:
//! - "5. Januar 2025" → "fuenfter januar zweitausendfuenfundzwanzig"
//! - "05.01.2025" → "fuenfter erster zweitausendfuenfundzwanzig"
//! - "January 5, 2025" → "fuenfter januar zweitausendfuenfundzwanzig"

use super::number_to_words;
use super::ordinal::ordinal_word_ter;

const MONTHS_DE: &[(&str, &str)] = &[
    ("januar", "januar"),
    ("februar", "februar"),
    ("maerz", "maerz"),
    ("april", "april"),
    ("mai", "mai"),
    ("juni", "juni"),
    ("juli", "juli"),
    ("august", "august"),
    ("september", "september"),
    ("oktober", "oktober"),
    ("november", "november"),
    ("dezember", "dezember"),
];

const MONTHS_EN: &[(&str, u32)] = &[
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

const MONTH_NAMES: &[&str] = &[
    "",
    "januar",
    "februar",
    "maerz",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "dezember",
];

/// Parse a written date to spoken German.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "die achtziger jahre"
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try German format: "5. Januar 2025"
    if let Some(result) = parse_german_date(trimmed) {
        return Some(result);
    }

    // Try English month format: "January 5, 2025"
    if let Some(result) = parse_english_month_date(trimmed) {
        return Some(result);
    }

    // Try numeric DD.MM.YYYY (German uses dots)
    if let Some(result) = parse_numeric_date(trimmed) {
        return Some(result);
    }

    None
}

/// Parse decade: "1980s" → "die achtziger jahre"
fn parse_decade(input: &str) -> Option<String> {
    let s = input.strip_suffix('s')?;
    if s.len() != 4 || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let year: u32 = s.parse().ok()?;
    if year < 1000 {
        return None;
    }

    // Must be a round decade (ends in 0)
    if year % 10 != 0 {
        return None;
    }

    // German: "die [year] jahre"
    let year_words = number_to_words(year as i64);
    Some(format!("die {}er jahre", year_words))
}

fn parse_german_date(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    // "5. Januar" or "5. Januar 2025"
    // Day must end with "." for German ordinal
    let day_str = tokens[0].strip_suffix('.')?;
    if !day_str.chars().all(|c| c.is_ascii_digit()) || day_str.is_empty() {
        return None;
    }

    let day: u32 = day_str.parse().ok()?;
    if day == 0 || day > 31 {
        return None;
    }

    // Find month
    let month_name = MONTHS_DE.iter().find(|(name, _)| *name == tokens[1]);
    let month_spoken = month_name?.1;

    let day_word = ordinal_word_ter(day);

    if tokens.len() >= 3 {
        let year_str = tokens[2]
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if year_str.chars().all(|c| c.is_ascii_digit()) && year_str.len() == 4 {
            let year: u32 = year_str.parse().ok()?;
            let year_words = verbalize_year(year)?;
            return Some(format!("{} {} {}", day_word, month_spoken, year_words));
        }
    }

    Some(format!("{} {}", day_word, month_spoken))
}

fn parse_english_month_date(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    let mut month_num = None;
    let mut rest = "";
    for &(name, num) in MONTHS_EN {
        if let Some(r) = lower.strip_prefix(name) {
            if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
                month_num = Some(num);
                rest = r.trim_start_matches(|c: char| c == ' ' || c == ',');
                break;
            }
        }
    }

    let month_num = month_num?;
    if rest.is_empty() {
        return None;
    }

    let month_name = MONTH_NAMES[month_num as usize];

    // Parse day
    let (day_str, year_part) = if let Some(comma_pos) = rest.find(',') {
        (&rest[..comma_pos], Some(rest[comma_pos + 1..].trim()))
    } else {
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        if parts.len() == 2
            && parts[0]
                .trim_end_matches("st")
                .trim_end_matches("nd")
                .trim_end_matches("rd")
                .trim_end_matches("th")
                .chars()
                .all(|c| c.is_ascii_digit())
        {
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

    let day_digits = day_str
        .trim()
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

    let day_word = ordinal_word_ter(day);

    if let Some(year_str) = year_part {
        let year_str = year_str
            .trim()
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if !year_str.is_empty() && year_str.chars().all(|c| c.is_ascii_digit()) {
            let year: u32 = year_str.parse().ok()?;
            let year_words = verbalize_year(year)?;
            return Some(format!("{} {} {}", day_word, month_name, year_words));
        }
    }

    Some(format!("{} {}", day_word, month_name))
}

/// Parse numeric date DD.MM.YYYY (German convention uses dots).
fn parse_numeric_date(input: &str) -> Option<String> {
    let sep = if input.contains('.') && input.chars().filter(|c| *c == '.').count() == 2 {
        '.'
    } else if input.contains('/') {
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

    if !parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }

    let day: u32 = parts[0].parse().ok()?;
    let month_num: u32 = parts[1].parse().ok()?;
    let year: u32 = parts[2].parse().ok()?;

    if month_num == 0 || month_num > 12 || day == 0 || day > 31 {
        return None;
    }

    let month_name = MONTH_NAMES[month_num as usize];
    let day_word = ordinal_word_ter(day);
    let year_words = verbalize_year(year)?;

    Some(format!("{} {} {}", day_word, month_name, year_words))
}

/// Verbalize a year in German.
/// - 2025 → "zweitausendfuenfundzwanzig"
/// - 2000 → "zweitausend"
/// - 1990 → "neunzehnhundertneunzig"
fn verbalize_year(year: u32) -> Option<String> {
    if year == 0 {
        return Some("null".to_string());
    }
    // German typically says the full number for years
    Some(number_to_words(year as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_german_date() {
        assert_eq!(
            parse("5. Januar 2025"),
            Some("fuenfter januar zweitausendfuenfundzwanzig".to_string())
        );
        assert_eq!(parse("1. Mai"), Some("erster mai".to_string()));
        assert_eq!(
            parse("3. Dezember 2000"),
            Some("dritter dezember zweitausend".to_string())
        );
    }

    #[test]
    fn test_english_month() {
        assert_eq!(
            parse("January 5, 2025"),
            Some("fuenfter januar zweitausendfuenfundzwanzig".to_string())
        );
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("05.01.2025"),
            Some("fuenfter januar zweitausendfuenfundzwanzig".to_string())
        );
        assert_eq!(
            parse("31.12.1999"),
            Some(
                "einunddreissigster dezember eintausend neunhundertneunundneunzig".to_string()
            )
        );
    }

    #[test]
    fn test_decade() {
        assert_eq!(
            parse("1980s"),
            Some("die eintausend neunhundertachtziger jahre".to_string())
        );
        assert_eq!(
            parse("2000s"),
            Some("die zweitausender jahre".to_string())
        );
        assert_eq!(
            parse("1990s"),
            Some("die eintausend neunhundertneunziger jahre".to_string())
        );
    }

    #[test]
    fn test_year_verbalization() {
        assert_eq!(
            verbalize_year(2025),
            Some("zweitausendfuenfundzwanzig".to_string())
        );
        assert_eq!(verbalize_year(2000), Some("zweitausend".to_string()));
        assert_eq!(
            verbalize_year(1990),
            Some("eintausend neunhundertneunzig".to_string())
        );
        assert_eq!(
            verbalize_year(1900),
            Some("eintausend neunhundert".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
