//! Time TN tagger for Japanese (romaji output).
//!
//! Converts written time expressions to spoken Japanese in romaji:
//! - "14:30" → "juu yo ji san juppun"
//! - "9:00" → "ku ji"
//! - "7:15" → "shichi ji juu go fun"

use super::number_to_words;

/// Special hour readings for Japanese.
/// 4=yo ji, 7=shichi ji, 9=ku ji; compound hours preserve these:
/// 14=juu yo ji, 17=juu shichi ji, 19=juu ku ji.
fn hour_to_romaji(hour: u32) -> String {
    match hour {
        0 => "zero ji".to_string(),
        4 => "yo ji".to_string(),
        7 => "shichi ji".to_string(),
        9 => "ku ji".to_string(),
        14 => "juu yo ji".to_string(),
        17 => "juu shichi ji".to_string(),
        19 => "juu ku ji".to_string(),
        _ => format!("{} ji", number_to_words(hour as i64)),
    }
}

/// Special minute readings with sound changes (rendaku).
/// 1=ippun, 3=sanpun, 4=yonpun, 6=roppun, 8=happun, 10=juppun
/// Others: number + fun
fn minute_to_romaji(minute: u32) -> String {
    match minute {
        0 => String::new(),
        1 => "ippun".to_string(),
        2 => "ni fun".to_string(),
        3 => "sanpun".to_string(),
        4 => "yonpun".to_string(),
        5 => "go fun".to_string(),
        6 => "roppun".to_string(),
        7 => "nana fun".to_string(),
        8 => "happun".to_string(),
        9 => "kyuu fun".to_string(),
        10 => "juppun".to_string(),
        _ => {
            // For compound minutes, apply rules to the ones digit
            let tens = minute / 10;
            let ones = minute % 10;

            if ones == 0 {
                // Exact tens: 20, 30, 40, 50
                let tens_word = number_to_words(tens as i64);
                match tens {
                    2 => "ni juppun".to_string(),
                    3 => "san juppun".to_string(),
                    4 => "yon juppun".to_string(),
                    5 => "go juppun".to_string(),
                    _ => format!("{} juppun", tens_word),
                }
            } else {
                // Compound: tens + ones minute reading
                let tens_part = if tens > 1 {
                    format!("{} juu", number_to_words(tens as i64))
                } else if tens == 1 {
                    "juu".to_string()
                } else {
                    String::new()
                };

                let ones_part = match ones {
                    1 => "ippun".to_string(),
                    2 => "ni fun".to_string(),
                    3 => "sanpun".to_string(),
                    4 => "yonpun".to_string(),
                    5 => "go fun".to_string(),
                    6 => "roppun".to_string(),
                    7 => "nana fun".to_string(),
                    8 => "happun".to_string(),
                    9 => "kyuu fun".to_string(),
                    _ => unreachable!(),
                };

                if tens_part.is_empty() {
                    ones_part
                } else {
                    format!("{} {}", tens_part, ones_part)
                }
            }
        }
    }
}

/// Parse a written time expression to spoken Japanese in romaji.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try "14:30" format
    if let Some(result) = parse_colon_format(trimmed) {
        return Some(result);
    }

    // Try "14時30分" format
    if let Some(result) = parse_japanese_format(trimmed) {
        return Some(result);
    }

    None
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

fn parse_japanese_format(input: &str) -> Option<String> {
    // Pattern: H時M分
    let ji_pos = input.find('\u{6642}')?; // 時
    let hour_str = &input[..ji_pos];
    if !hour_str.chars().all(|c| c.is_ascii_digit()) || hour_str.is_empty() {
        return None;
    }
    let hour: u32 = hour_str.parse().ok()?;
    if hour > 23 {
        return None;
    }

    let after_ji = &input[ji_pos + '\u{6642}'.len_utf8()..];

    let minute: u32 = if after_ji.is_empty() {
        0
    } else {
        let min_str = if let Some(fun_pos) = after_ji.find('\u{5206}') {
            // 分
            &after_ji[..fun_pos]
        } else {
            after_ji.trim()
        };
        if min_str.is_empty() {
            0
        } else if !min_str.chars().all(|c| c.is_ascii_digit()) {
            return None;
        } else {
            let m: u32 = min_str.parse().ok()?;
            if m > 59 {
                return None;
            }
            m
        }
    };

    Some(format_time(hour, minute))
}

fn format_time(hour: u32, minute: u32) -> String {
    let hour_words = hour_to_romaji(hour);

    if minute == 0 {
        hour_words
    } else {
        let min_words = minute_to_romaji(minute);
        format!("{} {}", hour_words, min_words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colon_format() {
        assert_eq!(
            parse("14:30"),
            Some("juu yo ji san juppun".to_string())
        );
        assert_eq!(parse("9:00"), Some("ku ji".to_string()));
        assert_eq!(parse("7:15"), Some("shichi ji juu go fun".to_string()));
        assert_eq!(parse("4:00"), Some("yo ji".to_string()));
    }

    #[test]
    fn test_minute_sound_changes() {
        assert_eq!(parse("3:01"), Some("san ji ippun".to_string()));
        assert_eq!(parse("3:03"), Some("san ji sanpun".to_string()));
        assert_eq!(parse("3:06"), Some("san ji roppun".to_string()));
        assert_eq!(parse("3:08"), Some("san ji happun".to_string()));
        assert_eq!(parse("3:10"), Some("san ji juppun".to_string()));
    }

    #[test]
    fn test_japanese_format() {
        assert_eq!(
            parse("14\u{6642}30\u{5206}"),
            Some("juu yo ji san juppun".to_string())
        );
        assert_eq!(parse("9\u{6642}"), Some("ku ji".to_string()));
    }

    #[test]
    fn test_compound_minutes() {
        assert_eq!(
            parse("3:21"),
            Some("san ji ni juu ippun".to_string())
        );
        assert_eq!(
            parse("3:45"),
            Some("san ji yon juu go fun".to_string())
        );
    }

    #[test]
    fn test_24h() {
        assert_eq!(parse("14:00"), Some("juu yo ji".to_string()));
        assert_eq!(
            parse("23:59"),
            Some("ni juu san ji go juu kyuu fun".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
        assert_eq!(parse(""), None);
    }
}
