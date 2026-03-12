//! Date TN tagger for Japanese (romaji output).
//!
//! Converts written date expressions to spoken Japanese in romaji:
//! - "2025年1月5日" → "ni sen ni juu go nen ichi gatsu itsuka"
//! - "2025-01-05" → "ni sen ni juu go nen ichi gatsu itsuka"
//! - "January 5, 2025" → "ni sen ni juu go nen ichi gatsu itsuka"

use super::number_to_words;

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

/// Special day readings for Japanese dates.
/// Days 1-10 and some others have special kun'yomi readings.
fn day_to_romaji(day: u32) -> String {
    match day {
        1 => "tsuitachi".to_string(),
        2 => "futsuka".to_string(),
        3 => "mikka".to_string(),
        4 => "yokka".to_string(),
        5 => "itsuka".to_string(),
        6 => "muika".to_string(),
        7 => "nanoka".to_string(),
        8 => "youka".to_string(),
        9 => "kokonoka".to_string(),
        10 => "tooka".to_string(),
        14 => "juu yokka".to_string(),
        20 => "hatsuka".to_string(),
        24 => "ni juu yokka".to_string(),
        _ => format!("{} nichi", number_to_words(day as i64)),
    }
}

/// Month reading: number + "gatsu"
fn month_to_romaji(month: u32) -> String {
    format!("{} gatsu", number_to_words(month as i64))
}

/// Parse a written date to spoken Japanese in romaji.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "sen kyuu hyaku hachi juu nen dai"
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try Japanese format: "2025年1月5日"
    if let Some(result) = parse_japanese_date(trimmed) {
        return Some(result);
    }

    // Try English month format: "January 5, 2025"
    if let Some(result) = parse_english_month_date(trimmed) {
        return Some(result);
    }

    // Try numeric YYYY-MM-DD or YYYY/MM/DD
    if let Some(result) = parse_numeric_date(trimmed) {
        return Some(result);
    }

    None
}

/// Parse decade: "1980s" → "sen kyuu hyaku hachi juu nen dai" (1980年代)
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

    // Japanese: year + "nen dai" (年代)
    let year_words = number_to_words(year as i64);
    Some(format!("{} nen dai", year_words))
}

fn parse_japanese_date(input: &str) -> Option<String> {
    // Pattern: YYYY年M月D日
    let nen_pos = input.find('\u{5E74}')?; // 年
    let year_str = &input[..nen_pos];
    if !year_str.chars().all(|c| c.is_ascii_digit()) || year_str.is_empty() {
        return None;
    }
    let year: u32 = year_str.parse().ok()?;

    let after_nen = &input[nen_pos + '\u{5E74}'.len_utf8()..];

    let gatsu_pos = after_nen.find('\u{6708}')?; // 月
    let month_str = &after_nen[..gatsu_pos];
    if !month_str.chars().all(|c| c.is_ascii_digit()) || month_str.is_empty() {
        return None;
    }
    let month: u32 = month_str.parse().ok()?;
    if month == 0 || month > 12 {
        return None;
    }

    let after_gatsu = &after_nen[gatsu_pos + '\u{6708}'.len_utf8()..];

    // Day part: may or may not end with 日
    let day_str = if let Some(nichi_pos) = after_gatsu.find('\u{65E5}') {
        // 日
        &after_gatsu[..nichi_pos]
    } else {
        after_gatsu.trim()
    };

    if !day_str.chars().all(|c| c.is_ascii_digit()) || day_str.is_empty() {
        return None;
    }
    let day: u32 = day_str.parse().ok()?;
    if day == 0 || day > 31 {
        return None;
    }

    let year_words = number_to_words(year as i64);
    let month_words = month_to_romaji(month);
    let day_words = day_to_romaji(day);

    Some(format!("{} nen {} {}", year_words, month_words, day_words))
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

    let month_words = month_to_romaji(month_num);
    let day_words = day_to_romaji(day);

    if let Some(year_str) = year_part {
        let year_str = year_str
            .trim()
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if !year_str.is_empty() && year_str.chars().all(|c| c.is_ascii_digit()) {
            let year: u32 = year_str.parse().ok()?;
            let year_words = number_to_words(year as i64);
            return Some(format!("{} nen {} {}", year_words, month_words, day_words));
        }
    }

    Some(format!("{} {}", month_words, day_words))
}

/// Parse numeric date in YYYY-MM-DD or YYYY/MM/DD format.
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

    // Assume YYYY-MM-DD (ISO format, common in Japan)
    let year: u32 = parts[0].parse().ok()?;
    let month_num: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    if month_num == 0 || month_num > 12 || day == 0 || day > 31 {
        return None;
    }

    // Reject if first part looks like a day (1-31) rather than a year
    if year <= 31 {
        return None;
    }

    let year_words = number_to_words(year as i64);
    let month_words = month_to_romaji(month_num);
    let day_words = day_to_romaji(day);

    Some(format!("{} nen {} {}", year_words, month_words, day_words))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_date() {
        assert_eq!(
            parse("2025\u{5E74}1\u{6708}5\u{65E5}"),
            Some("ni sen ni juu go nen ichi gatsu itsuka".to_string())
        );
        assert_eq!(
            parse("2025\u{5E74}3\u{6708}14\u{65E5}"),
            Some("ni sen ni juu go nen san gatsu juu yokka".to_string())
        );
        assert_eq!(
            parse("2025\u{5E74}12\u{6708}20\u{65E5}"),
            Some("ni sen ni juu go nen juu ni gatsu hatsuka".to_string())
        );
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("2025-01-05"),
            Some("ni sen ni juu go nen ichi gatsu itsuka".to_string())
        );
        assert_eq!(
            parse("2025/03/01"),
            Some("ni sen ni juu go nen san gatsu tsuitachi".to_string())
        );
    }

    #[test]
    fn test_english_month() {
        assert_eq!(
            parse("January 5, 2025"),
            Some("ni sen ni juu go nen ichi gatsu itsuka".to_string())
        );
        assert_eq!(
            parse("March 14, 2025"),
            Some("ni sen ni juu go nen san gatsu juu yokka".to_string())
        );
    }

    #[test]
    fn test_special_days() {
        assert_eq!(
            parse("2025\u{5E74}1\u{6708}1\u{65E5}"),
            Some("ni sen ni juu go nen ichi gatsu tsuitachi".to_string())
        );
        assert_eq!(
            parse("2025\u{5E74}1\u{6708}10\u{65E5}"),
            Some("ni sen ni juu go nen ichi gatsu tooka".to_string())
        );
        assert_eq!(
            parse("2025\u{5E74}1\u{6708}24\u{65E5}"),
            Some("ni sen ni juu go nen ichi gatsu ni juu yokka".to_string())
        );
    }

    #[test]
    fn test_decade() {
        assert_eq!(
            parse("1980s"),
            Some("sen kyuu hyaku hachi juu nen dai".to_string())
        );
        assert_eq!(parse("2000s"), Some("ni sen nen dai".to_string()));
        assert_eq!(
            parse("1990s"),
            Some("sen kyuu hyaku kyuu juu nen dai".to_string())
        );
    }

    #[test]
    fn test_year_verbalization() {
        assert_eq!(number_to_words(2025), "ni sen ni juu go".to_string());
        assert_eq!(number_to_words(2000), "ni sen".to_string());
        assert_eq!(number_to_words(1990), "sen kyuu hyaku kyuu juu".to_string());
        assert_eq!(number_to_words(1900), "sen kyuu hyaku".to_string());
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
