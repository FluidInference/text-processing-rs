//! Measure TN tagger for French.
//!
//! Converts written measurements to spoken French:
//! - "200 km/h" → "deux cents kilometres par heure"
//! - "1 kg" → "un kilogramme"
//! - "72°C" → "soixante-douze degres celsius"

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
        m.insert("mm", UnitInfo { singular: "millimetre", plural: "millimetres" });
        m.insert("cm", UnitInfo { singular: "centimetre", plural: "centimetres" });
        m.insert("m", UnitInfo { singular: "metre", plural: "metres" });
        m.insert("km", UnitInfo { singular: "kilometre", plural: "kilometres" });
        m.insert("in", UnitInfo { singular: "pouce", plural: "pouces" });
        m.insert("ft", UnitInfo { singular: "pied", plural: "pieds" });
        m.insert("mi", UnitInfo { singular: "mile", plural: "miles" });

        // Weight
        m.insert("mg", UnitInfo { singular: "milligramme", plural: "milligrammes" });
        m.insert("g", UnitInfo { singular: "gramme", plural: "grammes" });
        m.insert("kg", UnitInfo { singular: "kilogramme", plural: "kilogrammes" });
        m.insert("lb", UnitInfo { singular: "livre", plural: "livres" });
        m.insert("oz", UnitInfo { singular: "once", plural: "onces" });
        m.insert("t", UnitInfo { singular: "tonne", plural: "tonnes" });

        // Volume
        m.insert("ml", UnitInfo { singular: "millilitre", plural: "millilitres" });
        m.insert("l", UnitInfo { singular: "litre", plural: "litres" });
        m.insert("L", UnitInfo { singular: "litre", plural: "litres" });

        // Speed
        m.insert("km/h", UnitInfo { singular: "kilometre par heure", plural: "kilometres par heure" });
        m.insert("mph", UnitInfo { singular: "mile par heure", plural: "miles par heure" });
        m.insert("m/s", UnitInfo { singular: "metre par seconde", plural: "metres par seconde" });

        // Time
        m.insert("s", UnitInfo { singular: "seconde", plural: "secondes" });
        m.insert("sec", UnitInfo { singular: "seconde", plural: "secondes" });
        m.insert("min", UnitInfo { singular: "minute", plural: "minutes" });
        m.insert("h", UnitInfo { singular: "heure", plural: "heures" });
        m.insert("hr", UnitInfo { singular: "heure", plural: "heures" });

        // Temperature
        m.insert("°C", UnitInfo { singular: "degre celsius", plural: "degres celsius" });
        m.insert("°F", UnitInfo { singular: "degre fahrenheit", plural: "degres fahrenheit" });

        // Data
        m.insert("KB", UnitInfo { singular: "kilooctet", plural: "kilooctets" });
        m.insert("MB", UnitInfo { singular: "megaoctet", plural: "megaoctets" });
        m.insert("GB", UnitInfo { singular: "gigaoctet", plural: "gigaoctets" });
        m.insert("TB", UnitInfo { singular: "teraoctet", plural: "teraoctets" });

        // Percentage
        m.insert("%", UnitInfo { singular: "pour cent", plural: "pour cent" });

        // Frequency
        m.insert("Hz", UnitInfo { singular: "hertz", plural: "hertz" });
        m.insert("kHz", UnitInfo { singular: "kilohertz", plural: "kilohertz" });
        m.insert("MHz", UnitInfo { singular: "megahertz", plural: "megahertz" });
        m.insert("GHz", UnitInfo { singular: "gigahertz", plural: "gigahertz" });

        m
    };
}

/// Parse a written measurement to spoken French.
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
                    format!("moins {} virgule {}", int_words, frac_words)
                } else {
                    format!("{} virgule {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_word));
            }
            continue;
        }

        let Ok(n) = clean.parse::<i64>() else {
            continue;
        };
        let num_words = if is_negative {
            format!("moins {}", number_to_words(n))
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
            Some("deux cents kilometres par heure".to_string())
        );
        assert_eq!(parse("1 kg"), Some("un kilogramme".to_string()));
        assert_eq!(parse("2 kg"), Some("deux kilogrammes".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(
            parse("72°C"),
            Some("soixante-douze degres celsius".to_string())
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("cinquante pour cent".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-66 kg"),
            Some("moins soixante-six kilogrammes".to_string())
        );
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("cinq cents megaoctets".to_string()));
        assert_eq!(parse("1 GB"), Some("un gigaoctet".to_string()));
    }

    #[test]
    fn test_decimal_with_empty_integer() {
        assert_eq!(
            parse(".5 kg"),
            Some("zero virgule cinq kilogrammes".to_string())
        );
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
