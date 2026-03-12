//! Measure TN tagger.
//!
//! Converts written measurements to spoken form:
//! - "200 km/h" → "two hundred kilometers per hour"
//! - "1 kg" → "one kilogram"
//! - "2 kg" → "two kilograms"
//! - "72°F" → "seventy two degrees fahrenheit"

use super::number_to_words;

use lazy_static::lazy_static;
use std::collections::HashMap;

/// Unit info: (singular, plural)
struct UnitInfo {
    singular: &'static str,
    plural: &'static str,
}

lazy_static! {
    static ref UNITS: HashMap<&'static str, UnitInfo> = {
        let mut m = HashMap::new();

        // Length
        m.insert("mm", UnitInfo { singular: "millimeter", plural: "millimeters" });
        m.insert("cm", UnitInfo { singular: "centimeter", plural: "centimeters" });
        m.insert("m", UnitInfo { singular: "meter", plural: "meters" });
        m.insert("km", UnitInfo { singular: "kilometer", plural: "kilometers" });
        m.insert("in", UnitInfo { singular: "inch", plural: "inches" });
        m.insert("ft", UnitInfo { singular: "foot", plural: "feet" });
        m.insert("yd", UnitInfo { singular: "yard", plural: "yards" });
        m.insert("mi", UnitInfo { singular: "mile", plural: "miles" });

        // Weight/Mass
        m.insert("mg", UnitInfo { singular: "milligram", plural: "milligrams" });
        m.insert("g", UnitInfo { singular: "gram", plural: "grams" });
        m.insert("kg", UnitInfo { singular: "kilogram", plural: "kilograms" });
        m.insert("lb", UnitInfo { singular: "pound", plural: "pounds" });
        m.insert("lbs", UnitInfo { singular: "pound", plural: "pounds" });
        m.insert("oz", UnitInfo { singular: "ounce", plural: "ounces" });
        m.insert("t", UnitInfo { singular: "ton", plural: "tons" });

        // Volume
        m.insert("ml", UnitInfo { singular: "milliliter", plural: "milliliters" });
        m.insert("l", UnitInfo { singular: "liter", plural: "liters" });
        m.insert("L", UnitInfo { singular: "liter", plural: "liters" });
        m.insert("gal", UnitInfo { singular: "gallon", plural: "gallons" });

        // Speed
        m.insert("km/h", UnitInfo { singular: "kilometer per hour", plural: "kilometers per hour" });
        m.insert("kmh", UnitInfo { singular: "kilometer per hour", plural: "kilometers per hour" });
        m.insert("mph", UnitInfo { singular: "mile per hour", plural: "miles per hour" });
        m.insert("m/s", UnitInfo { singular: "meter per second", plural: "meters per second" });
        m.insert("kph", UnitInfo { singular: "kilometer per hour", plural: "kilometers per hour" });

        // Time
        m.insert("s", UnitInfo { singular: "second", plural: "seconds" });
        m.insert("sec", UnitInfo { singular: "second", plural: "seconds" });
        m.insert("min", UnitInfo { singular: "minute", plural: "minutes" });
        m.insert("h", UnitInfo { singular: "hour", plural: "hours" });
        m.insert("hr", UnitInfo { singular: "hour", plural: "hours" });
        m.insert("hrs", UnitInfo { singular: "hour", plural: "hours" });

        // Temperature
        m.insert("°C", UnitInfo { singular: "degree celsius", plural: "degrees celsius" });
        m.insert("°F", UnitInfo { singular: "degree fahrenheit", plural: "degrees fahrenheit" });
        m.insert("°K", UnitInfo { singular: "kelvin", plural: "kelvin" });
        m.insert("C", UnitInfo { singular: "degree celsius", plural: "degrees celsius" });
        m.insert("F", UnitInfo { singular: "degree fahrenheit", plural: "degrees fahrenheit" });

        // Data
        m.insert("B", UnitInfo { singular: "byte", plural: "bytes" });
        m.insert("KB", UnitInfo { singular: "kilobyte", plural: "kilobytes" });
        m.insert("MB", UnitInfo { singular: "megabyte", plural: "megabytes" });
        m.insert("GB", UnitInfo { singular: "gigabyte", plural: "gigabytes" });
        m.insert("TB", UnitInfo { singular: "terabyte", plural: "terabytes" });

        // Area
        m.insert("sq ft", UnitInfo { singular: "square foot", plural: "square feet" });
        m.insert("sq m", UnitInfo { singular: "square meter", plural: "square meters" });
        m.insert("sq km", UnitInfo { singular: "square kilometer", plural: "square kilometers" });

        // Frequency
        m.insert("Hz", UnitInfo { singular: "hertz", plural: "hertz" });
        m.insert("kHz", UnitInfo { singular: "kilohertz", plural: "kilohertz" });
        m.insert("MHz", UnitInfo { singular: "megahertz", plural: "megahertz" });
        m.insert("GHz", UnitInfo { singular: "gigahertz", plural: "gigahertz" });

        // Pressure
        m.insert("psi", UnitInfo { singular: "p s i", plural: "p s i" });
        m.insert("atm", UnitInfo { singular: "atmosphere", plural: "atmospheres" });

        // Percentage
        m.insert("%", UnitInfo { singular: "percent", plural: "percent" });

        m
    };
}

/// Parse a written measurement to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try to find a unit suffix (longest match first)
    // Sort by length descending to match "km/h" before "h"
    let mut unit_matches: Vec<(&str, &UnitInfo)> = UNITS
        .iter()
        .filter(|(unit, _)| {
            trimmed.ends_with(*unit)
                && (trimmed.len() == unit.len()
                    || {
                        let before = &trimmed[..trimmed.len() - unit.len()];
                        // Require a space between number and unit for single-letter units
                        // to avoid false matches like "1980s" → "1980 seconds"
                        if unit.len() == 1 && unit.chars().all(|c| c.is_ascii_alphabetic()) {
                            before.ends_with(' ')
                        } else {
                            before.ends_with(' ') || before.ends_with(|c: char| c.is_ascii_digit())
                        }
                    })
        })
        .map(|(k, v)| (*k, v))
        .collect();

    // Sort by unit length descending (prefer longer matches)
    unit_matches.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (unit_str, unit_info) in unit_matches {
        let num_part = trimmed[..trimmed.len() - unit_str.len()].trim();
        if num_part.is_empty() {
            continue;
        }

        // Handle negative
        let (is_negative, digits) = if let Some(rest) = num_part.strip_prefix('-') {
            (true, rest.trim())
        } else {
            (false, num_part)
        };

        // Try to parse as number (with optional commas and decimals)
        let clean: String = digits.chars().filter(|c| *c != ',').collect();

        // Check if it's a valid number
        if clean.is_empty() || !clean.chars().all(|c| c.is_ascii_digit() || c == '.') {
            continue;
        }

        // For decimals, use decimal tagger logic
        if clean.contains('.') {
            let parts: Vec<&str> = clean.splitn(2, '.').collect();
            if parts.len() == 2 {
                let int_val: i64 = parts[0].parse().ok()?;
                let int_words = number_to_words(int_val);
                let frac_words = super::spell_digits(parts[1]);
                let unit_word = unit_info.plural; // decimals are usually plural
                let num_words = if is_negative {
                    format!("minus {} point {}", int_words, frac_words)
                } else {
                    format!("{} point {}", int_words, frac_words)
                };
                return Some(format!("{} {}", num_words, unit_word));
            }
            continue;
        }

        let n: i64 = clean.parse().ok()?;
        let num_words = if is_negative {
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
    fn test_basic_measures() {
        assert_eq!(
            parse("200 km/h"),
            Some("two hundred kilometers per hour".to_string())
        );
        assert_eq!(parse("1 kg"), Some("one kilogram".to_string()));
        assert_eq!(parse("2 kg"), Some("two kilograms".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-66 kg"),
            Some("minus sixty six kilograms".to_string())
        );
    }

    #[test]
    fn test_temperature() {
        assert_eq!(
            parse("72°F"),
            Some("seventy two degrees fahrenheit".to_string())
        );
        assert_eq!(
            parse("100°C"),
            Some("one hundred degrees celsius".to_string())
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(parse("50%"), Some("fifty percent".to_string()));
        assert_eq!(parse("1%"), Some("one percent".to_string()));
    }

    #[test]
    fn test_data() {
        assert_eq!(parse("500 MB"), Some("five hundred megabytes".to_string()));
        assert_eq!(parse("1 GB"), Some("one gigabyte".to_string()));
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
