//! Measure tagger for German.
//!
//! Converts spoken German measurements to written form:
//! - "zwei hundert kilometer pro stunde" → "200 km/h"
//! - "minus sechs und sechzig kilogramm" → "-66 kg"
//! - "eins komma eins zentimeter" → "1,1 cm"
//! - "ein halb fuß" → "1/2 ft"

use super::cardinal;
use super::decimal;
use super::fraction;

/// Unit definition
struct Unit {
    spoken: &'static [&'static str],
    symbol: &'static str,
}

const COMPOUND_UNITS: &[(&str, &str)] = &[
    ("kilometer pro stunde", "km/h"),
    ("meter pro sekunde", "m/s"),
];

const MODIFIER_UNITS: &[(&str, &str, &str)] = &[
    // (modifier, base_spoken, symbol_suffix)
    ("quadrat kilometer", "km²", ""),
    ("quadrat meter", "m²", ""),
    ("kubik zentimeter", "cm³", ""),
    ("kubik meter", "m³", ""),
];

const SIMPLE_UNITS: &[(&str, &str)] = &[
    // Longest first to avoid partial matches
    ("kilowattstunden", "kwh"),
    ("kilowattstunde", "kwh"),
    ("mikrometer", "μm"),
    ("millimeter", "mm"),
    ("zentimeter", "cm"),
    ("kilometer", "km"),
    ("millivolt", "mv"),
    ("milliliter", "ml"),
    ("kilogramm", "kg"),
    ("milligramm", "mg"),
    ("meter", "m"),
    ("gramm", "g"),
    ("tonnen", "t"),
    ("tonne", "t"),
    ("liter", "l"),
    ("stunden", "h"),
    ("stunde", "h"),
    ("minuten", "min"),
    ("minute", "min"),
    ("sekunden", "s"),
    ("sekunde", "s"),
    ("hertz", "hz"),
    ("volt", "v"),
    ("watt", "w"),
    ("fuß", "ft"),
    ("fuss", "ft"),
];

/// Parse spoken German measurement expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Try "pro MODIFIER UNIT" pattern: "X pro quadrat kilometer"
    if let Some(result) = parse_per_unit(input_trim) {
        return Some(result);
    }

    // Try compound units: "X kilometer pro stunde"
    if let Some(result) = parse_compound_unit(input_trim) {
        return Some(result);
    }

    // Try modifier units: "X quadrat kilometer", "X kubik meter"
    if let Some(result) = parse_modifier_unit(input_trim) {
        return Some(result);
    }

    // Try simple unit
    if let Some(result) = parse_simple_unit(input_trim) {
        return Some(result);
    }

    None
}

/// Parse "X pro MODIFIER UNIT" → "X /UNIT²"
fn parse_per_unit(input: &str) -> Option<String> {
    if !input.contains(" pro ") {
        return None;
    }

    let parts: Vec<&str> = input.splitn(2, " pro ").collect();
    if parts.len() != 2 {
        return None;
    }

    let num_part = parts[0].trim();
    let unit_part = parts[1].trim();

    // Parse unit with modifier
    let unit_symbol = parse_unit_symbol(unit_part)?;
    let num_value = parse_number_value(num_part)?;

    Some(format!("{} /{}", num_value, unit_symbol))
}

/// Parse compound unit: "X kilometer pro stunde"
fn parse_compound_unit(input: &str) -> Option<String> {
    for &(spoken, symbol) in COMPOUND_UNITS {
        if input.ends_with(spoken) {
            let num_part = input[..input.len() - spoken.len()].trim();
            let num_value = parse_number_value(num_part)?;
            return Some(format!("{} {}", num_value, symbol));
        }
    }
    None
}

/// Parse modifier unit: "X quadrat kilometer", "X kubik meter"
fn parse_modifier_unit(input: &str) -> Option<String> {
    for &(spoken, symbol, _) in MODIFIER_UNITS {
        if input.ends_with(spoken) {
            let num_part = input[..input.len() - spoken.len()].trim();
            let num_value = parse_number_value(num_part)?;
            return Some(format!("{} {}", num_value, symbol));
        }
    }
    None
}

/// Parse simple unit: "X kilogramm"
fn parse_simple_unit(input: &str) -> Option<String> {
    // Handle negative
    let (is_negative, rest) = if input.starts_with("minus ") {
        (true, input.strip_prefix("minus ")?)
    } else {
        (false, input)
    };

    for &(spoken, symbol) in SIMPLE_UNITS {
        if rest.ends_with(spoken) {
            let num_part = rest[..rest.len() - spoken.len()].trim();
            let sign = if is_negative { "-" } else { "" };
            let num_value = parse_number_value(num_part)?;
            return Some(format!("{}{} {}", sign, num_value, symbol));
        }
    }

    None
}

/// Parse unit symbol from spoken form
fn parse_unit_symbol(input: &str) -> Option<String> {
    // Check modifier units
    for &(spoken, symbol, _) in MODIFIER_UNITS {
        if input == spoken {
            return Some(symbol.to_string());
        }
    }

    // Check simple units
    for &(spoken, symbol) in SIMPLE_UNITS {
        if input == spoken {
            return Some(symbol.to_string());
        }
    }

    None
}

/// Parse number value - handles cardinal, decimal, fraction, and scale
fn parse_number_value(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    // Check for fraction: "ein halb", "ein ein halb"
    if let Some(frac_result) = fraction::parse(input) {
        return Some(frac_result);
    }

    // Check for decimal: contains "komma"
    if input.contains("komma") {
        return decimal::parse(input);
    }

    // Check for scale: "eine million", "neunzig millionen"
    let scale_words = ["millionen", "million", "milliarden", "milliarde"];
    for &sw in &scale_words {
        if input.ends_with(sw) {
            let num_part = input[..input.len() - sw.len()].trim();
            if let Some(num) = cardinal::words_to_number(num_part) {
                return Some(format!("{} {}", num, sw));
            }
        }
    }

    // Cardinal number
    if let Some(num) = cardinal::words_to_number(input) {
        return Some(num.to_string());
    }

    // Try as "null" → 0
    if input == "null" {
        return Some("0".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(parse("zwei hundert meter"), Some("200 m".to_string()));
        assert_eq!(parse("neunzig gramm"), Some("90 g".to_string()));
    }

    #[test]
    fn test_compound() {
        assert_eq!(
            parse("zwei hundert kilometer pro stunde"),
            Some("200 km/h".to_string())
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("minus sechs und sechzig kilogramm"),
            Some("-66 kg".to_string())
        );
    }

    #[test]
    fn test_per_unit() {
        assert_eq!(
            parse("sechs und fünfzig komma drei pro quadrat kilometer"),
            Some("56,3 /km²".to_string())
        );
    }

    #[test]
    fn test_fraction_measure() {
        assert_eq!(parse("ein halb fuß"), Some("1/2 ft".to_string()));
    }
}
