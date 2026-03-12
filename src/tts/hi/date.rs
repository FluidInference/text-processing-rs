//! Date TN tagger for Hindi (romanized).
//!
//! Converts written date expressions to spoken romanized Hindi:
//! - "5 January 2025" → "paanch janvari do hazaar pachchees"
//! - "5/1/2025" → "paanch janvari do hazaar pachchees" (DD/MM/YYYY)
//! - "15 march 2000" → "pandrah march do hazaar"

use super::number_to_words;

/// English month names mapped to their Hindi romanized equivalents and month number.
const MONTHS_EN: &[(&str, &str, u32)] = &[
    ("january", "janvari", 1),
    ("february", "farvari", 2),
    ("march", "march", 3),
    ("april", "aprail", 4),
    ("may", "mai", 5),
    ("june", "june", 6),
    ("july", "julai", 7),
    ("august", "agast", 8),
    ("september", "sitambar", 9),
    ("october", "aktubar", 10),
    ("november", "navambar", 11),
    ("december", "disambar", 12),
];

/// Month names by index (1-based) in romanized Hindi.
const MONTH_NAMES: &[&str] = &[
    "",          // 0 placeholder
    "janvari",   // 1
    "farvari",   // 2
    "march",     // 3
    "aprail",    // 4
    "mai",       // 5
    "june",      // 6
    "julai",     // 7
    "agast",     // 8
    "sitambar",  // 9
    "aktubar",   // 10
    "navambar",  // 11
    "disambar",  // 12
];

/// Parse a written date to spoken romanized Hindi.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "unnis sau assee ka dashak"
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try "5 January 2025" or "5 january" format
    if let Some(result) = parse_day_month_year(trimmed) {
        return Some(result);
    }

    // Try "January 5, 2025" format (English style)
    if let Some(result) = parse_month_day_year(trimmed) {
        return Some(result);
    }

    // Try numeric DD/MM/YYYY
    if let Some(result) = parse_numeric_date(trimmed) {
        return Some(result);
    }

    None
}

/// Parse decade: "1980s" → "unnis sau assee ka dashak"
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

    // Hindi: year number + "ka dashak" (का दशक)
    let year_words = number_to_words(year as i64);
    Some(format!("{} ka dashak", year_words))
}

/// Parse "5 January 2025" or "5 january" format.
fn parse_day_month_year(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    // Strip ordinal suffixes from day
    let day_str = tokens[0]
        .trim_end_matches("st")
        .trim_end_matches("nd")
        .trim_end_matches("rd")
        .trim_end_matches("th");
    if !day_str.chars().all(|c| c.is_ascii_digit()) || day_str.is_empty() {
        return None;
    }

    let day: u32 = day_str.parse().ok()?;
    if day == 0 || day > 31 {
        return None;
    }

    // Find month name
    let month_token = tokens[1].trim_end_matches(',');
    let month_hindi = MONTHS_EN
        .iter()
        .find(|(en, _, _)| *en == month_token)
        .map(|(_, hi, _)| *hi)?;

    let day_word = number_to_words(day as i64);

    if tokens.len() >= 3 {
        let year_str = tokens[2]
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if year_str.chars().all(|c| c.is_ascii_digit()) && year_str.len() == 4 {
            let year: u32 = year_str.parse().ok()?;
            let year_words = number_to_words(year as i64);
            return Some(format!("{} {} {}", day_word, month_hindi, year_words));
        }
    }

    Some(format!("{} {}", day_word, month_hindi))
}

/// Parse "January 5, 2025" format.
fn parse_month_day_year(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    let mut month_hindi = None;
    let mut rest = "";
    for &(en_name, hi_name, _) in MONTHS_EN {
        if let Some(r) = lower.strip_prefix(en_name) {
            if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
                month_hindi = Some(hi_name);
                rest = r.trim_start_matches(|c: char| c == ' ' || c == ',');
                break;
            }
        }
    }

    let month_hindi = month_hindi?;
    if rest.is_empty() {
        return None;
    }

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

    let day_word = number_to_words(day as i64);

    if let Some(year_str) = year_part {
        let year_str = year_str
            .trim()
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if !year_str.is_empty() && year_str.chars().all(|c| c.is_ascii_digit()) {
            let year: u32 = year_str.parse().ok()?;
            let year_words = number_to_words(year as i64);
            return Some(format!("{} {} {}", day_word, month_hindi, year_words));
        }
    }

    Some(format!("{} {}", day_word, month_hindi))
}

/// Parse numeric date DD/MM/YYYY (Indian convention, same as French: day first).
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
    let day_word = number_to_words(day as i64);
    let year_words = number_to_words(year as i64);

    Some(format!("{} {} {}", day_word, month_name, year_words))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_month_year() {
        assert_eq!(
            parse("5 January 2025"),
            Some("paanch janvari do hazaar pachchees".to_string())
        );
        assert_eq!(
            parse("15 march 2000"),
            Some("pandrah march do hazaar".to_string())
        );
        assert_eq!(parse("1 january"), Some("ek janvari".to_string()));
    }

    #[test]
    fn test_month_day_year() {
        assert_eq!(
            parse("January 5, 2025"),
            Some("paanch janvari do hazaar pachchees".to_string())
        );
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("05/01/2025"),
            Some("paanch janvari do hazaar pachchees".to_string())
        );
        assert_eq!(
            parse("26/01/1950"),
            Some("chhabees janvari ek hazaar nau sau pachaas".to_string())
        );
    }

    #[test]
    fn test_decade() {
        assert_eq!(
            parse("1980s"),
            Some("ek hazaar nau sau assi ka dashak".to_string())
        );
        assert_eq!(
            parse("2000s"),
            Some("do hazaar ka dashak".to_string())
        );
        assert_eq!(
            parse("1990s"),
            Some("ek hazaar nau sau nabbe ka dashak".to_string())
        );
    }

    #[test]
    fn test_year_verbalization() {
        assert_eq!(
            number_to_words(2025),
            "do hazaar pachchees".to_string()
        );
        assert_eq!(number_to_words(2000), "do hazaar".to_string());
        assert_eq!(
            number_to_words(1990),
            "ek hazaar nau sau nabbe".to_string()
        );
        assert_eq!(
            number_to_words(1900),
            "ek hazaar nau sau".to_string()
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
