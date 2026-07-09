//! Measure TN tagger.
//!
//! Converts written measurements to spoken form:
//! - "200 km/h" → "two hundred kilometers per hour"
//! - "1 kg" → "one kilogram"
//! - "2 kg" → "two kilograms"
//! - "72°F" → "seventy two degrees Fahrenheit"

use super::number_to_words_and;

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

        // Data rate
        m.insert("mbps", UnitInfo { singular: "megabit per second", plural: "megabits per second" });
        m.insert("gbps", UnitInfo { singular: "gigabit per second", plural: "gigabits per second" });
        m.insert("kbps", UnitInfo { singular: "kilobit per second", plural: "kilobits per second" });

        // Time
        m.insert("s", UnitInfo { singular: "second", plural: "seconds" });
        m.insert("sec", UnitInfo { singular: "second", plural: "seconds" });
        m.insert("min", UnitInfo { singular: "minute", plural: "minutes" });
        m.insert("h", UnitInfo { singular: "hour", plural: "hours" });
        m.insert("hr", UnitInfo { singular: "hour", plural: "hours" });
        m.insert("hrs", UnitInfo { singular: "hour", plural: "hours" });

        // Temperature
        m.insert("°C", UnitInfo { singular: "degree Celsius", plural: "degrees Celsius" });
        m.insert("°F", UnitInfo { singular: "degree Fahrenheit", plural: "degrees Fahrenheit" });
        m.insert("°K", UnitInfo { singular: "kelvin", plural: "kelvin" });
        m.insert("C", UnitInfo { singular: "degree Celsius", plural: "degrees Celsius" });
        m.insert("F", UnitInfo { singular: "degree Fahrenheit", plural: "degrees Fahrenheit" });

        // Data. Bare "B" is intentionally omitted: after a number an uppercase
        // B/K/M/G/T is a magnitude abbreviation (billion, …) kept literal by the
        // decimal tagger (NeMo). KB/MB/GB/TB still spell out as bytes.
        m.insert("KB", UnitInfo { singular: "kilobyte", plural: "kilobytes" });
        m.insert("MB", UnitInfo { singular: "megabyte", plural: "megabytes" });
        m.insert("GB", UnitInfo { singular: "gigabyte", plural: "gigabytes" });
        m.insert("TB", UnitInfo { singular: "terabyte", plural: "terabytes" });

        // Area
        m.insert("sq ft", UnitInfo { singular: "square foot", plural: "square feet" });
        m.insert("sq m", UnitInfo { singular: "square meter", plural: "square meters" });
        m.insert("sq km", UnitInfo { singular: "square kilometer", plural: "square kilometers" });

        // Volume (cubic)
        m.insert("mm³", UnitInfo { singular: "cubic millimeter", plural: "cubic millimeters" });
        m.insert("cm³", UnitInfo { singular: "cubic centimeter", plural: "cubic centimeters" });
        m.insert("m³", UnitInfo { singular: "cubic meter", plural: "cubic meters" });
        m.insert("km³", UnitInfo { singular: "cubic kilometer", plural: "cubic kilometers" });

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

    // Decimal attached to a unit word by a hyphen ("7.2-millimeter") or the "x"
    // multiplier ("4.4x"), per NeMo's decimal_dash_alpha / decimal_times.
    if let Some(result) = parse_decimal_unit(trimmed) {
        return Some(result);
    }

    // Dimensions with an area/volume unit: "2x8 m2" → "two by eight square
    // meters" (the "x" reads "by" only when an area unit follows).
    if let Some(result) = parse_dimension(trimmed) {
        return Some(result);
    }

    // Try to find a unit suffix (longest match first)
    // Sort by length descending to match "km/h" before "h"
    let mut unit_matches: Vec<(&str, &UnitInfo)> = UNITS
        .iter()
        .filter(|(unit, _)| {
            trimmed.ends_with(*unit)
                && (trimmed.len() == unit.len() || {
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

        // A spaced 4-digit decade ("1980 s") is not "1980 seconds" — leave it
        // to the date tagger, which reads it as "nineteen eighties".
        if unit_str == "s" && is_spaced_decade(num_part) {
            continue;
        }

        // Scale word between the number and unit ("100 million kg").
        if let Some(scaled) = parse_scaled(num_part) {
            return Some(format!("{} {}", scaled, unit_info.plural));
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
                let int_val: i64 = if parts[0].is_empty() {
                    0
                } else {
                    let Ok(v) = parts[0].parse::<i64>() else {
                        continue;
                    };
                    v
                };
                let int_words = number_to_words_and(int_val as u128);
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

        let Ok(n) = clean.parse::<u128>() else {
            continue;
        };
        let num_words = if is_negative {
            format!("minus {}", number_to_words_and(n))
        } else {
            number_to_words_and(n)
        };

        let unit_word = if n == 1 {
            unit_info.singular
        } else {
            unit_info.plural
        };

        return Some(format!("{} {}", num_words, unit_word));
    }

    // "value/unit" reads the slash as "per": "12/kg" → "twelve per kilogram",
    // "12kg/kg" → "twelve kilograms per kilogram".
    parse_per(trimmed)
}

/// A 4-digit multiple of ten ("1980") — a decade written before a spaced "s".
fn is_spaced_decade(num: &str) -> bool {
    num.len() == 4
        && num.chars().all(|c| c.is_ascii_digit())
        && num.parse::<u32>().map(|n| n % 10 == 0).unwrap_or(false)
}

/// Read a dimension with an area/volume unit: "2x8 m2" → "two by eight square
/// meters".
fn parse_dimension(input: &str) -> Option<String> {
    let (dims, unit) = input.split_once(' ')?;
    let unit_word = match unit {
        "m2" | "m²" => "square meters",
        "km2" | "km²" => "square kilometers",
        "cm2" | "cm²" => "square centimeters",
        "mm2" | "mm²" => "square millimeters",
        "ft2" => "square feet",
        "m3" | "m³" => "cubic meters",
        "km3" | "km³" => "cubic kilometers",
        _ => return None,
    };
    let (a, b) = dims.split_once(['x', 'X'])?;
    if a.is_empty() || b.is_empty() || !a.chars().chain(b.chars()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!(
        "{} by {} {}",
        number_to_words_and(a.parse().ok()?),
        number_to_words_and(b.parse().ok()?),
        unit_word
    ))
}

/// Read a decimal glued to a unit word by "-" ("7.2-millimeter" → "seven point
/// two millimeter") or by the "x" multiplier ("4.4x" → "four point four x").
/// The trailing word is a spelled-out unit and is kept verbatim.
fn parse_decimal_unit(input: &str) -> Option<String> {
    let (num, unit) = if let Some((n, u)) = input.split_once('-') {
        (n, u.to_string())
    } else if let Some(n) = input.strip_suffix(['x', 'X']) {
        (n, "x".to_string())
    } else {
        return None;
    };
    if unit.is_empty() || !unit.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let (int_str, frac_str) = num.split_once('.')?;
    if frac_str.is_empty() || !frac_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let clean: String = int_str.chars().filter(|c| *c != ',').collect();
    if clean.is_empty() || !clean.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!(
        "{} point {} {}",
        number_to_words_and(clean.parse().ok()?),
        super::spell_digits(frac_str),
        unit
    ))
}

/// Read a "value/unit" form where the right side is a bare unit.
fn parse_per(input: &str) -> Option<String> {
    let (left, right) = input.split_once('/')?;
    let (left, right) = (left.trim(), right.trim());
    let runit = UNITS.get(right)?;
    let left_spoken = parse(left).or_else(|| {
        let clean: String = left.chars().filter(|c| *c != ',').collect();
        if clean.is_empty() || !clean.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        Some(number_to_words_and(clean.parse().ok()?))
    })?;
    Some(format!("{} per {}", left_spoken, runit.singular))
}

/// Read a "<number> <scale>" magnitude ("100 million" → "one hundred million").
fn parse_scaled(s: &str) -> Option<String> {
    let (num, scale) = s.trim().rsplit_once(char::is_whitespace)?;
    let scale = match scale.to_ascii_lowercase().as_str() {
        "thousand" => "thousand",
        "million" => "million",
        "billion" => "billion",
        "trillion" => "trillion",
        _ => return None,
    };
    let clean: String = num.chars().filter(|c| *c != ',').collect();
    if clean.is_empty() || !clean.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!(
        "{} {}",
        number_to_words_and(clean.parse().ok()?),
        scale
    ))
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
            Some("seventy two degrees Fahrenheit".to_string())
        );
        assert_eq!(
            parse("100°C"),
            Some("one hundred degrees Celsius".to_string())
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
    fn test_decimal_with_empty_integer() {
        // ".5 kg" should not cause premature return — continue to next unit
        assert_eq!(
            parse(".5 kg"),
            Some("zero point five kilograms".to_string())
        );
    }

    #[test]
    fn test_non_measure() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
