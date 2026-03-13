//! Measure TN tagger for Spanish.
//!
//! Converts written measurements to spoken Spanish:
//! - "200 km/h" → "doscientos kilometros por hora"
//! - "1 kg" → "un kilogramo"
//! - "72°C" → "setenta y dos grados celsius"

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
        m.insert("mm", UnitInfo { singular: "milimetro", plural: "milimetros" });
        m.insert("cm", UnitInfo { singular: "centimetro", plural: "centimetros" });
        m.insert("m", UnitInfo { singular: "metro", plural: "metros" });
        m.insert("km", UnitInfo { singular: "kilometro", plural: "kilometros" });
        m.insert("in", UnitInfo { singular: "pulgada", plural: "pulgadas" });
        m.insert("ft", UnitInfo { singular: "pie", plural: "pies" });
        m.insert("mi", UnitInfo { singular: "milla", plural: "millas" });

        // Weight
        m.insert("mg", UnitInfo { singular: "miligramo", plural: "miligramos" });
        m.insert("g", UnitInfo { singular: "gramo", plural: "gramos" });
        m.insert("kg", UnitInfo { singular: "kilogramo", plural: "kilogramos" });
        m.insert("lb", UnitInfo { singular: "libra", plural: "libras" });
        m.insert("oz", UnitInfo { singular: "onza", plural: "onzas" });
        m.insert("t", UnitInfo { singular: "tonelada", plural: "toneladas" });

        // Volume
        m.insert("ml", UnitInfo { singular: "mililitro", plural: "mililitros" });
        m.insert("l", UnitInfo { singular: "litro", plural: "litros" });
        m.insert("L", UnitInfo { singular: "litro", plural: "litros" });

        // Speed
        m.insert("km/h", UnitInfo { singular: "kilometro por hora", plural: "kilometros por hora" });
        m.insert("mph", UnitInfo { singular: "milla por hora", plural: "millas por hora" });
        m.insert("m/s", UnitInfo { singular: "metro por segundo", plural: "metros por segundo" });

        // Time
        m.insert("s", UnitInfo { singular: "segundo", plural: "segundos" });
        m.insert("sec", UnitInfo { singular: "segundo", plural: "segundos" });
        m.insert("min", UnitInfo { singular: "minuto", plural: "minutos" });
        m.insert("h", UnitInfo { singular: "hora", plural: "horas" });
        m.insert("hr", UnitInfo { singular: "hora", plural: "horas" });

        // Temperature
        m.insert("°C", UnitInfo { singular: "grado celsius", plural: "grados celsius" });
        m.insert("°F", UnitInfo { singular: "grado fahrenheit", plural: "grados fahrenheit" });

        // Data
        m.insert("KB", UnitInfo { singular: "kilobyte", plural: "kilobytes" });
        m.insert("MB", UnitInfo { singular: "megabyte", plural: "megabytes" });
        m.insert("GB", UnitInfo { singular: "gigabyte", plural: "gigabytes" });
        m.insert("TB", UnitInfo { singular: "terabyte", plural: "terabytes" });

        // Percentage
        m.insert("%", UnitInfo { singular: "por ciento", plural: "por ciento" });

        // Frequency
        m.insert("Hz", UnitInfo { singular: "hercio", plural: "hercios" });
        m.insert("kHz", UnitInfo { singular: "kilohercio", plural: "kilohercios" });
        m.insert("MHz", UnitInfo { singular: "megahercio", plural: "megahercios" });
        m.insert("GHz", UnitInfo { singular: "gigahercio", plural: "gigahercios" });

        m
    };
}

/// Convert trailing "uno" to "un" for use before masculine nouns.
/// "uno" → "un", "veintiuno" → "veintiun", "treinta y uno" → "treinta y un"
fn apocope_uno(s: &str) -> String {
    if s == "uno" {
        return "un".to_string();
    }
    if let Some(prefix) = s.strip_suffix(" uno") {
        return format!("{} un", prefix);
    }
    if s.ends_with("iuno") {
        // "veintiuno" → "veintiun"
        return format!("{}un", &s[..s.len() - 3]);
    }
    s.to_string()
}

/// Parse a written measurement to spoken Spanish.
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
                    format!("menos {} coma {}", int_words, frac_words)
                } else {
                    format!("{} coma {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_word));
            }
            continue;
        }

        let Ok(n) = clean.parse::<i64>() else {
            continue;
        };
        let raw_words = if is_negative {
            format!("menos {}", number_to_words(n))
        } else {
            number_to_words(n)
        };
        // In Spanish, "uno" becomes "un" before a masculine noun (unit).
        // Also "veintiuno" → "veintiun", etc.
        let num_words = apocope_uno(&raw_words);

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
            Some("doscientos kilometros por hora".to_string())
        );
        assert_eq!(parse("1 kg"), Some("un kilogramo".to_string()));
        assert_eq!(parse("2 kg"), Some("dos kilogramos".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(
            parse("72°C"),
            Some("setenta y dos grados celsius".to_string())
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("cincuenta por ciento".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-66 kg"),
            Some("menos sesenta y seis kilogramos".to_string())
        );
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("quinientos megabytes".to_string()));
        assert_eq!(parse("1 GB"), Some("un gigabyte".to_string()));
    }

    #[test]
    fn test_decimal_with_empty_integer() {
        assert_eq!(
            parse(".5 kg"),
            Some("cero coma cinco kilogramos".to_string())
        );
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
