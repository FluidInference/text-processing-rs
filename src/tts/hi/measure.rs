//! Measure TN tagger for Hindi (romanized).
//!
//! Converts written measurements to spoken romanized Hindi:
//! - "200 km/h" → "do sau kilometre prati ghanta"
//! - "1 kg" → "ek kilogram"
//! - "50%" → "pachaas pratishat"
//! - "72°C" → "bahattar digri selsiyas"

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
        m.insert("mm", UnitInfo { singular: "millimetre", plural: "millimetre" });
        m.insert("cm", UnitInfo { singular: "centimetre", plural: "centimetre" });
        m.insert("m", UnitInfo { singular: "metre", plural: "metre" });
        m.insert("km", UnitInfo { singular: "kilometre", plural: "kilometre" });
        m.insert("in", UnitInfo { singular: "inch", plural: "inch" });
        m.insert("ft", UnitInfo { singular: "feet", plural: "feet" });
        m.insert("mi", UnitInfo { singular: "mile", plural: "mile" });

        // Weight
        m.insert("mg", UnitInfo { singular: "milligram", plural: "milligram" });
        m.insert("g", UnitInfo { singular: "gram", plural: "gram" });
        m.insert("kg", UnitInfo { singular: "kilogram", plural: "kilogram" });
        m.insert("lb", UnitInfo { singular: "pound", plural: "pound" });
        m.insert("oz", UnitInfo { singular: "aunce", plural: "aunce" });
        m.insert("t", UnitInfo { singular: "tan", plural: "tan" });

        // Volume
        m.insert("ml", UnitInfo { singular: "millilitre", plural: "millilitre" });
        m.insert("l", UnitInfo { singular: "litre", plural: "litre" });
        m.insert("L", UnitInfo { singular: "litre", plural: "litre" });

        // Speed
        m.insert("km/h", UnitInfo { singular: "kilometre prati ghanta", plural: "kilometre prati ghanta" });
        m.insert("mph", UnitInfo { singular: "mile prati ghanta", plural: "mile prati ghanta" });
        m.insert("m/s", UnitInfo { singular: "metre prati second", plural: "metre prati second" });

        // Time
        m.insert("s", UnitInfo { singular: "second", plural: "second" });
        m.insert("sec", UnitInfo { singular: "second", plural: "second" });
        m.insert("min", UnitInfo { singular: "minat", plural: "minat" });
        m.insert("h", UnitInfo { singular: "ghanta", plural: "ghante" });
        m.insert("hr", UnitInfo { singular: "ghanta", plural: "ghante" });

        // Temperature
        m.insert("\u{00B0}C", UnitInfo { singular: "digri selsiyas", plural: "digri selsiyas" });
        m.insert("\u{00B0}F", UnitInfo { singular: "digri farenheit", plural: "digri farenheit" });

        // Data
        m.insert("KB", UnitInfo { singular: "kilobyte", plural: "kilobyte" });
        m.insert("MB", UnitInfo { singular: "megabyte", plural: "megabyte" });
        m.insert("GB", UnitInfo { singular: "gigabyte", plural: "gigabyte" });
        m.insert("TB", UnitInfo { singular: "terabyte", plural: "terabyte" });

        // Percentage
        m.insert("%", UnitInfo { singular: "pratishat", plural: "pratishat" });

        // Frequency
        m.insert("Hz", UnitInfo { singular: "hertz", plural: "hertz" });
        m.insert("kHz", UnitInfo { singular: "kilohertz", plural: "kilohertz" });
        m.insert("MHz", UnitInfo { singular: "megahertz", plural: "megahertz" });
        m.insert("GHz", UnitInfo { singular: "gigahertz", plural: "gigahertz" });

        m
    };
}

/// Parse a written measurement to spoken romanized Hindi.
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

    // Sort by length descending so longer unit matches take priority (e.g. "km/h" over "h")
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
                let unit_word = unit_info.plural;
                let num_words = if is_negative {
                    format!("rhin {} dashmlav {}", int_words, frac_words)
                } else {
                    format!("{} dashmlav {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_word));
            }
            continue;
        }

        let Ok(n) = clean.parse::<i64>() else {
            continue;
        };
        let num_words = if is_negative {
            format!("rhin {}", number_to_words(n))
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
    fn test_speed() {
        assert_eq!(
            parse("200 km/h"),
            Some("do sau kilometre prati ghanta".to_string())
        );
    }

    #[test]
    fn test_weight() {
        assert_eq!(parse("1 kg"), Some("ek kilogram".to_string()));
        assert_eq!(parse("2 kg"), Some("do kilogram".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(
            parse("72\u{00B0}C"),
            Some("bahattar digri selsiyas".to_string())
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("pachaas pratishat".to_string()));
        assert_eq!(parse("100%"), Some("ek sau pratishat".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-66 kg"),
            Some("rhin chhiyaasath kilogram".to_string())
        );
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("paanch sau megabyte".to_string()));
        assert_eq!(parse("1 GB"), Some("ek gigabyte".to_string()));
    }

    #[test]
    fn test_decimal_with_empty_integer() {
        assert_eq!(
            parse(".5 kg"),
            Some("shunya dashmlav paanch kilogram".to_string())
        );
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
