//! Measure TN tagger for German.
//!
//! Converts written measurements to spoken German:
//! - "200 km/h" → "zweihundert kilometer pro stunde"
//! - "1 kg" → "ein kilogramm"
//! - "72°C" → "zweiundsiebzig grad celsius"

use super::number_to_words;

use lazy_static::lazy_static;
use std::collections::HashMap;

struct UnitInfo {
    singular: &'static str,
    plural: &'static str,
}

lazy_static! {
    static ref UNITS: HashMap<&'static str, UnitInfo> = {
        let mut m = HashMap::new();

        // Length
        m.insert("mm", UnitInfo { singular: "millimeter", plural: "millimeter" });
        m.insert("cm", UnitInfo { singular: "zentimeter", plural: "zentimeter" });
        m.insert("m", UnitInfo { singular: "meter", plural: "meter" });
        m.insert("km", UnitInfo { singular: "kilometer", plural: "kilometer" });
        m.insert("in", UnitInfo { singular: "zoll", plural: "zoll" });
        m.insert("ft", UnitInfo { singular: "fuss", plural: "fuss" });
        m.insert("mi", UnitInfo { singular: "meile", plural: "meilen" });

        // Weight
        m.insert("mg", UnitInfo { singular: "milligramm", plural: "milligramm" });
        m.insert("g", UnitInfo { singular: "gramm", plural: "gramm" });
        m.insert("kg", UnitInfo { singular: "kilogramm", plural: "kilogramm" });
        m.insert("lb", UnitInfo { singular: "pfund", plural: "pfund" });
        m.insert("oz", UnitInfo { singular: "unze", plural: "unzen" });
        m.insert("t", UnitInfo { singular: "tonne", plural: "tonnen" });

        // Volume
        m.insert("ml", UnitInfo { singular: "milliliter", plural: "milliliter" });
        m.insert("l", UnitInfo { singular: "liter", plural: "liter" });
        m.insert("L", UnitInfo { singular: "liter", plural: "liter" });

        // Speed — "pro" instead of "per" for rates
        m.insert("km/h", UnitInfo { singular: "kilometer pro stunde", plural: "kilometer pro stunde" });
        m.insert("mph", UnitInfo { singular: "meile pro stunde", plural: "meilen pro stunde" });
        m.insert("m/s", UnitInfo { singular: "meter pro sekunde", plural: "meter pro sekunde" });

        // Time
        m.insert("s", UnitInfo { singular: "sekunde", plural: "sekunden" });
        m.insert("sec", UnitInfo { singular: "sekunde", plural: "sekunden" });
        m.insert("min", UnitInfo { singular: "minute", plural: "minuten" });
        m.insert("h", UnitInfo { singular: "stunde", plural: "stunden" });
        m.insert("hr", UnitInfo { singular: "stunde", plural: "stunden" });

        // Temperature
        m.insert("°C", UnitInfo { singular: "grad celsius", plural: "grad celsius" });
        m.insert("°F", UnitInfo { singular: "grad fahrenheit", plural: "grad fahrenheit" });

        // Data
        m.insert("KB", UnitInfo { singular: "kilobyte", plural: "kilobyte" });
        m.insert("MB", UnitInfo { singular: "megabyte", plural: "megabyte" });
        m.insert("GB", UnitInfo { singular: "gigabyte", plural: "gigabyte" });
        m.insert("TB", UnitInfo { singular: "terabyte", plural: "terabyte" });

        // Percentage — "prozent"
        m.insert("%", UnitInfo { singular: "prozent", plural: "prozent" });

        // Frequency
        m.insert("Hz", UnitInfo { singular: "hertz", plural: "hertz" });
        m.insert("kHz", UnitInfo { singular: "kilohertz", plural: "kilohertz" });
        m.insert("MHz", UnitInfo { singular: "megahertz", plural: "megahertz" });
        m.insert("GHz", UnitInfo { singular: "gigahertz", plural: "gigahertz" });

        m
    };
}

/// Parse a written measurement to spoken German.
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

        if clean.is_empty() || !clean.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
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
                let unit_word = unit_info.plural;
                let num_words = if is_negative {
                    format!("minus {} komma {}", int_words, frac_words)
                } else {
                    format!("{} komma {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_word));
            }
            continue;
        }

        let Ok(n) = clean.parse::<i64>() else {
            continue;
        };

        // Use "ein" instead of "eins" when before a unit
        let num_words = if n == 1 && !is_negative {
            "ein".to_string()
        } else if n == 1 && is_negative {
            "minus ein".to_string()
        } else if is_negative {
            format!("minus {}", number_to_words(n))
        } else {
            number_to_words(n)
        };

        let abs_n = n.unsigned_abs();
        let unit_word = if abs_n == 1 {
            unit_info.singular
        } else {
            unit_info.plural
        };

        return Some(format!("{} {}", num_words, unit_word));
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
            Some("zweihundert kilometer pro stunde".to_string())
        );
        assert_eq!(parse("1 kg"), Some("ein kilogramm".to_string()));
        assert_eq!(parse("2 kg"), Some("zwei kilogramm".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(
            parse("72°C"),
            Some("zweiundsiebzig grad celsius".to_string())
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("fuenfzig prozent".to_string()));
        assert_eq!(parse("100%"), Some("einhundert prozent".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-66 kg"),
            Some("minus sechsundsechzig kilogramm".to_string())
        );
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("fuenfhundert megabyte".to_string()));
        assert_eq!(parse("1 GB"), Some("ein gigabyte".to_string()));
    }

    #[test]
    fn test_decimal_with_empty_integer() {
        assert_eq!(
            parse(".5 kg"),
            Some("null komma fuenf kilogramm".to_string())
        );
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
