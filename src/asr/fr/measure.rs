//! Measure tagger for French.
//!
//! Converts spoken French measurements to written form:
//! - "deux cents mètres" → "200 m"
//! - "dix-huit virgule cinq kilomètres" → "18,5 km"
//! - "cent kilomètres par heure" → "100 km/h"

use super::cardinal::words_to_number;
use super::decimal;

/// Parse spoken French measurement expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    // Try compound units first (most specific)
    if let Some(result) = parse_compound_unit(input_trimmed) {
        return Some(result);
    }

    // Try simple unit
    if let Some(result) = parse_simple_unit(input_trimmed) {
        return Some(result);
    }

    None
}

/// Parse compound units like "kilomètres par heure" → "km/h"
fn parse_compound_unit(input: &str) -> Option<String> {
    // "X kilomètres par heure" → "X km/h"
    if input.ends_with(" kilomètres par heure") || input.ends_with(" kilomètre par heure") {
        let num_part = input
            .strip_suffix(" kilomètres par heure")
            .or_else(|| input.strip_suffix(" kilomètre par heure"))?;
        let num_value = parse_number_value(num_part.trim())?;
        return Some(format!("{} km/h", num_value));
    }

    // "X mètres par seconde" → "X m/s"
    if input.ends_with(" mètres par seconde") || input.ends_with(" mètre par seconde") {
        let num_part = input
            .strip_suffix(" mètres par seconde")
            .or_else(|| input.strip_suffix(" mètre par seconde"))?;
        let num_value = parse_number_value(num_part.trim())?;
        return Some(format!("{} m/s", num_value));
    }

    None
}

/// Parse simple measurement: number + unit
fn parse_simple_unit(input: &str) -> Option<String> {
    let (value, unit) = parse_number_and_unit(input)?;
    Some(format!("{} {}", value, unit))
}

/// Parse number and unit from input
fn parse_number_and_unit(input: &str) -> Option<(String, String)> {
    // Handle negative
    let (is_negative, rest) = if input.starts_with("moins ") {
        (true, input.strip_prefix("moins ")?)
    } else {
        (false, input)
    };

    // Try to find unit at the end
    let (num_part, unit_symbol) = extract_unit(rest)?;

    // Parse the number part
    let num_value = parse_number_value(num_part.trim())?;

    let sign = if is_negative { "-" } else { "" };
    Some((format!("{}{}", sign, num_value), unit_symbol))
}

/// Extract unit from end of string
fn extract_unit(input: &str) -> Option<(&str, String)> {
    // Try each unit pattern
    for (spoken, symbol) in get_unit_mappings() {
        if input.ends_with(spoken) {
            let num_part = input.strip_suffix(spoken)?.trim();
            return Some((num_part, symbol.to_string()));
        }
    }

    None
}

/// Parse number value (handles both cardinal and decimal)
fn parse_number_value(input: &str) -> Option<String> {
    // Try decimal first (has "virgule")
    if input.contains(" virgule ") {
        return decimal::parse(input);
    }

    // Cardinal number
    let num = words_to_number(input)?;
    Some((num as i64).to_string())
}

/// Get French unit mappings (spoken -> symbol)
fn get_unit_mappings() -> Vec<(&'static str, &'static str)> {
    vec![
        // Distance/Length (plural and singular)
        (" kilomètres", "km"),
        (" kilomètre", "km"),
        (" mètres", "m"),
        (" mètre", "m"),
        (" centimètres", "cm"),
        (" centimètre", "cm"),
        (" millimètres", "mm"),
        (" millimètre", "mm"),
        // Mass/Weight
        (" kilogrammes", "kg"),
        (" kilogramme", "kg"),
        (" grammes", "g"),
        (" gramme", "g"),
        (" tonnes", "t"),
        (" tonne", "t"),
        // Volume
        (" litres", "l"),
        (" litre", "l"),
        (" millilitres", "ml"),
        (" millilitre", "ml"),
        // Time
        (" heures", "h"),
        (" heure", "h"),
        (" minutes", "min"),
        (" minute", "min"),
        (" secondes", "s"),
        (" seconde", "s"),
        // Temperature
        (" degrés celsius", "°C"),
        (" degré celsius", "°C"),
        (" degrés", "°"),
        (" degré", "°"),
        // Data
        (" gigaoctets", "Go"),
        (" gigaoctet", "Go"),
        (" mégaoctets", "Mo"),
        (" mégaoctet", "Mo"),
        (" kilooctets", "Ko"),
        (" kilooctet", "Ko"),
        // Power
        (" kilowatts", "kW"),
        (" kilowatt", "kW"),
        (" watts", "W"),
        (" watt", "W"),
        // Percentage
        (" pourcent", "%"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        assert_eq!(parse("cent mètres"), Some("100 m".to_string()));
        assert_eq!(parse("cinq kilomètres"), Some("5 km".to_string()));
    }

    #[test]
    fn test_speed() {
        assert_eq!(
            parse("cent kilomètres par heure"),
            Some("100 km/h".to_string())
        );
    }

    #[test]
    fn test_weight() {
        assert_eq!(parse("deux kilogrammes"), Some("2 kg".to_string()));
        assert_eq!(parse("cinquante grammes"), Some("50 g".to_string()));
    }

    #[test]
    fn test_temperature() {
        assert_eq!(parse("vingt degrés celsius"), Some("20 °C".to_string()));
    }

    #[test]
    fn test_decimal_measure() {
        assert_eq!(
            parse("dix-huit virgule cinq kilomètres"),
            Some("18,5 km".to_string())
        );
    }
}
