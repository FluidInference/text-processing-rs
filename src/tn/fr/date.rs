//! Date TN tagger for French.
//!
//! Converts written date expressions to spoken French:
//! - "5 janvier 2025" → "cinq janvier deux mille vingt-cinq"
//! - "January 5, 2025" → "cinq janvier deux mille vingt-cinq"
//! - "05/01/2025" → "cinq janvier deux mille vingt-cinq" (DD/MM/YYYY)

use super::number_to_words;

const MONTHS_FR: &[(&str, &str)] = &[
    ("janvier", "janvier"),
    ("fevrier", "fevrier"),
    ("mars", "mars"),
    ("avril", "avril"),
    ("mai", "mai"),
    ("juin", "juin"),
    ("juillet", "juillet"),
    ("aout", "aout"),
    ("septembre", "septembre"),
    ("octobre", "octobre"),
    ("novembre", "novembre"),
    ("decembre", "decembre"),
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
    "janvier",
    "fevrier",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "aout",
    "septembre",
    "octobre",
    "novembre",
    "decembre",
];

/// Parse a written date to spoken French.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "les annees mille neuf cent quatre-vingts"
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try French format: "5 janvier 2025"
    if let Some(result) = parse_french_date(trimmed) {
        return Some(result);
    }

    // Try English month format: "January 5, 2025"
    if let Some(result) = parse_english_month_date(trimmed) {
        return Some(result);
    }

    // Try numeric DD/MM/YYYY
    if let Some(result) = parse_numeric_date(trimmed) {
        return Some(result);
    }

    None
}

/// Parse decade: "1980s" → "les annees mille neuf cent quatre-vingts"
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

    // French: "les annees" + the year number
    let year_words = number_to_words(year as i64);
    Some(format!("les annees {}", year_words))
}

fn parse_french_date(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    // "5 janvier" or "5 janvier 2025" or "1er janvier 2025"
    let day_str = tokens[0]
        .trim_end_matches("er")
        .trim_end_matches("eme")
        .trim_end_matches('e');
    if !day_str.chars().all(|c| c.is_ascii_digit()) || day_str.is_empty() {
        return None;
    }

    let day: u32 = day_str.parse().ok()?;
    if day == 0 || day > 31 {
        return None;
    }

    // Find month
    let month_name = MONTHS_FR.iter().find(|(name, _)| *name == tokens[1]);
    let month_spoken = month_name?.1;

    let day_word = if day == 1 {
        "premier".to_string()
    } else {
        number_to_words(day as i64)
    };

    if tokens.len() >= 3 {
        let year_str =
            tokens[2].trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
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

    let day_word = if day == 1 {
        "premier".to_string()
    } else {
        number_to_words(day as i64)
    };

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

/// Parse numeric date DD/MM/YYYY (French convention).
fn parse_numeric_date(input: &str) -> Option<String> {
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
    let day_word = if day == 1 {
        "premier".to_string()
    } else {
        number_to_words(day as i64)
    };
    let year_words = verbalize_year(year)?;

    Some(format!("{} {} {}", day_word, month_name, year_words))
}

/// Verbalize a year in French.
/// - 2025 → "deux mille vingt-cinq"
/// - 2000 → "deux mille"
/// - 1990 → "mille neuf cent quatre-vingt-dix"
fn verbalize_year(year: u32) -> Option<String> {
    if year == 0 {
        return Some("zero".to_string());
    }
    // French typically says the full number for years
    Some(number_to_words(year as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_french_date() {
        assert_eq!(
            parse("5 janvier 2025"),
            Some("cinq janvier deux mille vingt-cinq".to_string())
        );
        assert_eq!(parse("1er janvier"), Some("premier janvier".to_string()));
    }

    #[test]
    fn test_english_month() {
        assert_eq!(
            parse("January 5, 2025"),
            Some("cinq janvier deux mille vingt-cinq".to_string())
        );
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("05/01/2025"),
            Some("cinq janvier deux mille vingt-cinq".to_string())
        );
    }

    #[test]
    fn test_decade() {
        assert_eq!(
            parse("1980s"),
            Some("les annees mille neuf cent quatre-vingts".to_string())
        );
        assert_eq!(parse("2000s"), Some("les annees deux mille".to_string()));
        assert_eq!(
            parse("1990s"),
            Some("les annees mille neuf cent quatre-vingt-dix".to_string())
        );
    }

    #[test]
    fn test_year_verbalization() {
        assert_eq!(
            verbalize_year(2025),
            Some("deux mille vingt-cinq".to_string())
        );
        assert_eq!(verbalize_year(2000), Some("deux mille".to_string()));
        assert_eq!(
            verbalize_year(1990),
            Some("mille neuf cent quatre-vingt-dix".to_string())
        );
        assert_eq!(verbalize_year(1900), Some("mille neuf cent".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
