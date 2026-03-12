//! Measure TN tagger for Mandarin Chinese.
//!
//! Converts written measurements to spoken Mandarin pinyin:
//! - "200 km/h" -> "er bai gongli mei xiaoshi"
//! - "1 kg" -> "yi gongjin"
//! - "72°C" -> "qi shi er shengshi du"
//! - "50%" -> "bai fen zhi wu shi"
//!
//! Chinese unit names are given in pinyin. "mei" is used for "per".
//! Percentage uses the Chinese idiom "bai fen zhi N" (百分之N, literally "of 100 parts, N").

use super::number_to_words;

use lazy_static::lazy_static;
use std::collections::HashMap;

struct UnitInfo {
    /// Spoken name in pinyin (Chinese has no singular/plural distinction)
    name: &'static str,
}

lazy_static! {
    static ref UNITS: HashMap<&'static str, UnitInfo> = {
        let mut m = HashMap::new();

        // Length
        m.insert("mm", UnitInfo { name: "haomi" });       // 毫米
        m.insert("cm", UnitInfo { name: "limi" });         // 厘米
        m.insert("m", UnitInfo { name: "mi" });            // 米
        m.insert("km", UnitInfo { name: "gongli" });       // 公里
        m.insert("in", UnitInfo { name: "yingcun" });      // 英寸
        m.insert("ft", UnitInfo { name: "yingchi" });      // 英尺
        m.insert("mi", UnitInfo { name: "yingli" });       // 英里

        // Weight
        m.insert("mg", UnitInfo { name: "haoke" });        // 毫克
        m.insert("g", UnitInfo { name: "ke" });            // 克
        m.insert("kg", UnitInfo { name: "gongjin" });      // 公斤
        m.insert("lb", UnitInfo { name: "bang" });         // 磅
        m.insert("oz", UnitInfo { name: "angsi" });        // 盎司
        m.insert("t", UnitInfo { name: "dun" });           // 吨

        // Volume
        m.insert("ml", UnitInfo { name: "haosheng" });     // 毫升
        m.insert("l", UnitInfo { name: "sheng" });         // 升
        m.insert("L", UnitInfo { name: "sheng" });         // 升

        // Speed
        m.insert("km/h", UnitInfo { name: "gongli mei xiaoshi" }); // 公里每小时
        m.insert("mph", UnitInfo { name: "yingli mei xiaoshi" });  // 英里每小时
        m.insert("m/s", UnitInfo { name: "mi mei miao" });        // 米每秒

        // Time
        m.insert("s", UnitInfo { name: "miao" });          // 秒
        m.insert("sec", UnitInfo { name: "miao" });        // 秒
        m.insert("min", UnitInfo { name: "fenzhong" });    // 分钟
        m.insert("h", UnitInfo { name: "xiaoshi" });       // 小时
        m.insert("hr", UnitInfo { name: "xiaoshi" });      // 小时

        // Temperature
        m.insert("\u{00B0}C", UnitInfo { name: "sheshidu" });      // 摄氏度
        m.insert("\u{00B0}F", UnitInfo { name: "huashidu" });      // 华氏度

        // Data
        m.insert("KB", UnitInfo { name: "qianzi jie" });    // 千字节
        m.insert("MB", UnitInfo { name: "zhaozi jie" });    // 兆字节
        m.insert("GB", UnitInfo { name: "jizi jie" });      // 吉字节
        m.insert("TB", UnitInfo { name: "taizi jie" });     // 太字节

        // Frequency
        m.insert("Hz", UnitInfo { name: "hezi" });          // 赫兹
        m.insert("kHz", UnitInfo { name: "qianhezi" });     // 千赫兹
        m.insert("MHz", UnitInfo { name: "zhaohezi" });     // 兆赫兹
        m.insert("GHz", UnitInfo { name: "jihezi" });       // 吉赫兹

        m
    };
}

/// Parse a written measurement to spoken Mandarin pinyin.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Special handling for percentage: "50%" -> "bai fen zhi wu shi"
    if let Some(result) = parse_percentage(trimmed) {
        return Some(result);
    }

    // Try matching known units (longest match first)
    let mut unit_matches: Vec<(&str, &UnitInfo)> = UNITS
        .iter()
        .filter(|(unit, _)| {
            trimmed.ends_with(*unit)
                && (trimmed.len() == unit.len() || {
                    let before = &trimmed[..trimmed.len() - unit.len()];
                    if unit.len() == 1 && unit.chars().all(|c| c.is_ascii_alphabetic()) {
                        before.ends_with(' ')
                    } else {
                        before.ends_with(' ') || before.ends_with(|c: char| c.is_ascii_digit())
                    }
                })
        })
        .map(|(k, v)| (*k, v))
        .collect();

    unit_matches.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (unit_str, unit_info) in unit_matches {
        let num_part = trimmed[..trimmed.len() - unit_str.len()].trim();
        if num_part.is_empty() {
            continue;
        }

        let (is_negative, digits) = if let Some(rest) = num_part.strip_prefix('-') {
            (true, rest.trim())
        } else {
            (false, num_part)
        };

        let clean: String = digits
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
            .collect();

        if clean.is_empty()
            || !clean
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
        {
            continue;
        }

        // Handle decimals
        if clean.contains('.') {
            let parts: Vec<&str> = clean.splitn(2, '.').collect();
            if parts.len() == 2 {
                let int_val: i64 = if parts[0].is_empty() {
                    0
                } else {
                    let Ok(v) = parts[0].parse::<i64>() else {
                        continue;
                    };
                    v
                };
                let int_words = number_to_words(int_val);
                let frac_words = super::spell_digits(parts[1]);
                let num_words = if is_negative {
                    format!("fu {} dian {}", int_words, frac_words)
                } else {
                    format!("{} dian {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_info.name));
            }
            continue;
        }

        let Ok(n) = clean.parse::<i64>() else {
            continue;
        };
        let num_words = if is_negative {
            format!("fu {}", number_to_words(n))
        } else {
            number_to_words(n)
        };

        return Some(format!("{} {}", num_words, unit_info.name));
    }

    None
}

/// Parse percentage: "50%" -> "bai fen zhi wu shi" (百分之五十)
fn parse_percentage(input: &str) -> Option<String> {
    let num_str = input.strip_suffix('%')?;
    let num_str = num_str.trim();

    if num_str.is_empty() {
        return None;
    }

    let (is_negative, digits) = if let Some(rest) = num_str.strip_prefix('-') {
        (true, rest.trim())
    } else {
        (false, num_str)
    };

    // Handle decimal percentages
    if digits.contains('.') {
        let parts: Vec<&str> = digits.splitn(2, '.').collect();
        if parts.len() == 2
            && !parts[0].is_empty()
            && parts[0].chars().all(|c| c.is_ascii_digit())
            && !parts[1].is_empty()
            && parts[1].chars().all(|c| c.is_ascii_digit())
        {
            let int_val: i64 = parts[0].parse().ok()?;
            let int_words = number_to_words(int_val);
            let frac_words = super::spell_digits(parts[1]);
            let num_words = format!("{} dian {}", int_words, frac_words);
            if is_negative {
                return Some(format!("fu bai fen zhi {}", num_words));
            } else {
                return Some(format!("bai fen zhi {}", num_words));
            }
        }
        return None;
    }

    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: i64 = digits.parse().ok()?;
    let num_words = number_to_words(n);

    if is_negative {
        Some(format!("fu bai fen zhi {}", num_words))
    } else {
        Some(format!("bai fen zhi {}", num_words))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_units() {
        assert_eq!(
            parse("200 km/h"),
            Some("er bai gongli mei xiaoshi".to_string())
        );
        assert_eq!(parse("1 kg"), Some("yi gongjin".to_string()));
        assert_eq!(parse("5 m"), Some("wu mi".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(parse("72\u{00B0}C"), Some("qi shi er sheshidu".to_string()));
        assert_eq!(
            parse("98\u{00B0}F"),
            Some("jiu shi ba huashidu".to_string())
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("bai fen zhi wu shi".to_string()));
        assert_eq!(parse("100%"), Some("bai fen zhi yi bai".to_string()));
        assert_eq!(parse("3.5%"), Some("bai fen zhi san dian wu".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-66 kg"),
            Some("fu liu shi liu gongjin".to_string())
        );
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("wu bai zhaozi jie".to_string()));
        assert_eq!(parse("1 GB"), Some("yi jizi jie".to_string()));
    }

    #[test]
    fn test_decimal_with_empty_integer() {
        assert_eq!(
            parse(".5 kg"),
            Some("ling dian wu gongjin".to_string())
        );
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
