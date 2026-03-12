//! Measure TN tagger for Japanese (romaji output).
//!
//! Converts written measurements to spoken Japanese in romaji:
//! - "200 km/h" → "ni hyaku kiromeetoru mai ji"
//! - "1 kg" → "ichi kiroguramu"
//! - "25°C" → "ni juu go do"
//! - "50%" → "go juu paasento"

use super::number_to_words;

use lazy_static::lazy_static;
use std::collections::HashMap;

struct UnitInfo {
    /// Japanese unit name in romaji (no singular/plural distinction in Japanese)
    name: &'static str,
}

lazy_static! {
    static ref UNITS: HashMap<&'static str, UnitInfo> = {
        let mut m = HashMap::new();

        // Length
        m.insert("mm", UnitInfo { name: "mirimeetoru" });
        m.insert("cm", UnitInfo { name: "senchimeetoru" });
        m.insert("m", UnitInfo { name: "meetoru" });
        m.insert("km", UnitInfo { name: "kiromeetoru" });
        m.insert("in", UnitInfo { name: "inchi" });
        m.insert("ft", UnitInfo { name: "fiito" });
        m.insert("mi", UnitInfo { name: "mairu" });

        // Weight
        m.insert("mg", UnitInfo { name: "miriguramu" });
        m.insert("g", UnitInfo { name: "guramu" });
        m.insert("kg", UnitInfo { name: "kiroguramu" });
        m.insert("lb", UnitInfo { name: "pondo" });
        m.insert("oz", UnitInfo { name: "onsu" });
        m.insert("t", UnitInfo { name: "ton" });

        // Volume
        m.insert("ml", UnitInfo { name: "miririttoru" });
        m.insert("l", UnitInfo { name: "rittoru" });
        m.insert("L", UnitInfo { name: "rittoru" });

        // Speed
        m.insert("km/h", UnitInfo { name: "kiromeetoru mai ji" });
        m.insert("mph", UnitInfo { name: "mairu mai ji" });
        m.insert("m/s", UnitInfo { name: "meetoru mai byou" });

        // Time
        m.insert("s", UnitInfo { name: "byou" });
        m.insert("sec", UnitInfo { name: "byou" });
        m.insert("min", UnitInfo { name: "fun" });
        m.insert("h", UnitInfo { name: "jikan" });
        m.insert("hr", UnitInfo { name: "jikan" });

        // Temperature
        m.insert("\u{00B0}C", UnitInfo { name: "do" });
        m.insert("\u{00B0}F", UnitInfo { name: "do" });

        // Data
        m.insert("KB", UnitInfo { name: "kirobaito" });
        m.insert("MB", UnitInfo { name: "megabaito" });
        m.insert("GB", UnitInfo { name: "gigabaito" });
        m.insert("TB", UnitInfo { name: "terabaito" });

        // Percentage
        m.insert("%", UnitInfo { name: "paasento" });

        // Frequency
        m.insert("Hz", UnitInfo { name: "herutsu" });
        m.insert("kHz", UnitInfo { name: "kiroherutsu" });
        m.insert("MHz", UnitInfo { name: "megaherutsu" });
        m.insert("GHz", UnitInfo { name: "gigaherutsu" });

        m
    };
}

/// Parse a written measurement to spoken Japanese in romaji.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

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

    // Sort by unit length descending to prefer longer matches (km/h over h)
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
        let decimal_sep = if clean.contains(',') { ',' } else { '.' };
        if clean.contains(decimal_sep) {
            let parts: Vec<&str> = clean.splitn(2, decimal_sep).collect();
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
                    format!("mainasu {} ten {}", int_words, frac_words)
                } else {
                    format!("{} ten {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_info.name));
            }
            continue;
        }

        let Ok(n) = clean.parse::<i64>() else {
            continue;
        };
        let num_words = if is_negative {
            format!("mainasu {}", number_to_words(n))
        } else {
            number_to_words(n)
        };

        return Some(format!("{} {}", num_words, unit_info.name));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            parse("200 km/h"),
            Some("ni hyaku kiromeetoru mai ji".to_string())
        );
        assert_eq!(parse("1 kg"), Some("ichi kiroguramu".to_string()));
        assert_eq!(parse("5 km"), Some("go kiromeetoru".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(parse("25\u{00B0}C"), Some("ni juu go do".to_string()));
        assert_eq!(parse("0\u{00B0}C"), Some("zero do".to_string()));
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("go juu paasento".to_string()));
        assert_eq!(parse("100%"), Some("hyaku paasento".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-5\u{00B0}C"), Some("mainasu go do".to_string()));
        assert_eq!(
            parse("-66 kg"),
            Some("mainasu roku juu roku kiroguramu".to_string())
        );
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("go hyaku megabaito".to_string()));
        assert_eq!(parse("1 GB"), Some("ichi gigabaito".to_string()));
    }

    #[test]
    fn test_decimal_with_empty_integer() {
        assert_eq!(parse(".5 kg"), Some("zero ten go kiroguramu".to_string()));
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
