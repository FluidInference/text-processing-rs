//! Date TN tagger for Mandarin Chinese.
//!
//! Converts written date expressions to spoken Mandarin pinyin:
//! - "2025年1月5日" -> "er ling er wu nian yi yue wu ri"
//! - "2025-01-05" -> "er ling er wu nian yi yue wu ri"
//! - "January 5, 2025" -> "er ling er wu nian yi yue wu ri"
//!
//! Year: each digit spelled out + "nian"
//! Month: cardinal number + "yue"
//! Day: cardinal number + "ri" (or "hao")

use super::{number_to_words, spell_digits};

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

/// Parse a written date to spoken Mandarin pinyin.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "yi jiu ba ling nian dai" (一九八零年代)
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try Chinese format: 2025年1月5日
    if let Some(result) = parse_chinese_date(trimmed) {
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

/// Parse decade: "1980s" → "yi jiu ba ling nian dai" (一九八零年代)
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

    // Chinese: spell each digit + "nian dai"
    let year_words = spell_digits(s);
    Some(format!("{} nian dai", year_words))
}

fn parse_chinese_date(input: &str) -> Option<String> {
    // Look for 年 (nian), 月 (yue), 日 (ri) markers
    let nian_char = '\u{5E74}'; // 年
    let yue_char = '\u{6708}'; // 月
    let ri_char = '\u{65E5}'; // 日
    let hao_char = '\u{53F7}'; // 号

    let has_nian = input.contains(nian_char);
    let has_yue = input.contains(yue_char);

    if !has_nian && !has_yue {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    let mut remaining = input;

    // Extract year (before 年)
    if has_nian {
        let nian_pos = remaining.find(nian_char)?;
        let year_str = &remaining[..nian_pos];
        if !year_str.is_empty() && year_str.chars().all(|c| c.is_ascii_digit()) {
            let year_words = spell_digits(year_str);
            parts.push(format!("{} nian", year_words));
        }
        remaining = &remaining[nian_pos + nian_char.len_utf8()..];
    }

    // Extract month (before 月)
    if has_yue {
        let yue_pos = remaining.find(yue_char)?;
        let month_str = &remaining[..yue_pos].trim();
        if !month_str.is_empty() && month_str.chars().all(|c| c.is_ascii_digit()) {
            let month: u32 = month_str.parse().ok()?;
            if month == 0 || month > 12 {
                return None;
            }
            parts.push(format!("{} yue", number_to_words(month as i64)));
        }
        remaining = &remaining[yue_pos + yue_char.len_utf8()..];
    }

    // Extract day (before 日 or 号)
    let day_end = remaining.find(ri_char).or_else(|| remaining.find(hao_char));
    if let Some(pos) = day_end {
        let day_str = &remaining[..pos].trim();
        if !day_str.is_empty() && day_str.chars().all(|c| c.is_ascii_digit()) {
            let day: u32 = day_str.parse().ok()?;
            if day == 0 || day > 31 {
                return None;
            }
            parts.push(format!("{} ri", number_to_words(day as i64)));
        }
    } else {
        // No 日/号 marker, check if there are trailing digits for the day
        let day_str = remaining.trim();
        if !day_str.is_empty() && day_str.chars().all(|c| c.is_ascii_digit()) {
            let day: u32 = day_str.parse().ok()?;
            if day > 0 && day <= 31 {
                parts.push(format!("{} ri", number_to_words(day as i64)));
            }
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(" "))
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
        let tokens: Vec<&str> = rest.splitn(2, ' ').collect();
        if tokens.len() == 2
            && tokens[0]
                .trim_end_matches("st")
                .trim_end_matches("nd")
                .trim_end_matches("rd")
                .trim_end_matches("th")
                .chars()
                .all(|c| c.is_ascii_digit())
        {
            let year_clean =
                tokens[1].trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
            if year_clean.chars().all(|c| c.is_ascii_digit()) && year_clean.len() == 4 {
                (tokens[0], Some(year_clean))
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

    let mut parts: Vec<String> = Vec::new();

    if let Some(year_str) = year_part {
        let year_str = year_str
            .trim()
            .trim_end_matches(|c: char| c == '.' || c == ',' || c == '!' || c == '?');
        if !year_str.is_empty() && year_str.chars().all(|c| c.is_ascii_digit()) {
            parts.push(format!("{} nian", spell_digits(year_str)));
        }
    }

    parts.push(format!("{} yue", number_to_words(month_num as i64)));
    parts.push(format!("{} ri", number_to_words(day as i64)));

    Some(parts.join(" "))
}

/// Parse numeric date YYYY-MM-DD or YYYY/MM/DD.
fn parse_numeric_date(input: &str) -> Option<String> {
    let sep = if input.contains('/') {
        '/'
    } else if input.contains('-') && input.chars().filter(|c| *c == '-').count() == 2 {
        '-'
    } else {
        return None;
    };

    let tokens: Vec<&str> = input.splitn(3, sep).collect();
    if tokens.len() != 3 {
        return None;
    }

    if !tokens
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }

    // Determine if YYYY-MM-DD or DD-MM-YYYY by checking first token length
    let (year, month_num, day) = if tokens[0].len() == 4 {
        // YYYY-MM-DD
        let y: u32 = tokens[0].parse().ok()?;
        let m: u32 = tokens[1].parse().ok()?;
        let d: u32 = tokens[2].parse().ok()?;
        (y, m, d)
    } else {
        // Assume DD-MM-YYYY (less common for Chinese context, but support it)
        let d: u32 = tokens[0].parse().ok()?;
        let m: u32 = tokens[1].parse().ok()?;
        let y: u32 = tokens[2].parse().ok()?;
        (y, m, d)
    };

    if month_num == 0 || month_num > 12 || day == 0 || day > 31 {
        return None;
    }

    let year_words = spell_digits(&year.to_string());
    let month_words = number_to_words(month_num as i64);
    let day_words = number_to_words(day as i64);

    Some(format!(
        "{} nian {} yue {} ri",
        year_words, month_words, day_words
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_date() {
        assert_eq!(
            parse("2025\u{5E74}1\u{6708}5\u{65E5}"),
            Some("er ling er wu nian yi yue wu ri".to_string())
        );
        assert_eq!(
            parse("2025\u{5E74}12\u{6708}31\u{65E5}"),
            Some("er ling er wu nian shi er yue san shi yi ri".to_string())
        );
    }

    #[test]
    fn test_english_month() {
        assert_eq!(
            parse("January 5, 2025"),
            Some("er ling er wu nian yi yue wu ri".to_string())
        );
        assert_eq!(
            parse("December 25, 2000"),
            Some("er ling ling ling nian shi er yue er shi wu ri".to_string())
        );
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("2025-01-05"),
            Some("er ling er wu nian yi yue wu ri".to_string())
        );
        assert_eq!(
            parse("2025/03/15"),
            Some("er ling er wu nian san yue shi wu ri".to_string())
        );
    }

    #[test]
    fn test_decade() {
        assert_eq!(parse("1980s"), Some("yi jiu ba ling nian dai".to_string()));
        assert_eq!(
            parse("2000s"),
            Some("er ling ling ling nian dai".to_string())
        );
        assert_eq!(parse("1990s"), Some("yi jiu jiu ling nian dai".to_string()));
    }

    #[test]
    fn test_year_verbalization() {
        // In Chinese, year digits are spelled individually
        assert_eq!(spell_digits("2025"), "er ling er wu".to_string());
        assert_eq!(spell_digits("2000"), "er ling ling ling".to_string());
        assert_eq!(spell_digits("1990"), "yi jiu jiu ling".to_string());
        assert_eq!(spell_digits("1900"), "yi jiu ling ling".to_string());
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
