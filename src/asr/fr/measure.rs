//! Measure tagger for French.
//!
//! Converts spoken French measurements to written form:
//! - "deux cents mètres" → "200 m"
//! - "dix-huit virgule cinq kilomètres" → "18,5 km"
//! - "cent kilomètres par heure" → "100 km/h"
//! - "soixante-cinq kilomètres carrés" → "65 km²"
//! - "deux mètres cubes" → "2 m³"

use super::cardinal::words_to_number;
use super::decimal;

/// Parse spoken French measurement expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    // Try rate units first (X par Y): "par kilomètre carré", "par mètre carré"
    if let Some(result) = parse_rate_unit(input_trimmed) {
        return Some(result);
    }

    // Try compound units: "kilomètres par heure", "mètres par seconde", "kilomètres heure"
    if let Some(result) = parse_compound_unit(input_trimmed) {
        return Some(result);
    }

    // Try simple unit with modifiers (carrés, cubes)
    if let Some(result) = parse_simple_unit(input_trimmed) {
        return Some(result);
    }

    None
}

/// Parse rate expressions: "X par kilomètre carré" → "X /km²"
fn parse_rate_unit(input: &str) -> Option<String> {
    let rate_units = [
        (" par kilomètre carré", "/km²"),
        (" par mètre carré", "/m²"),
        (" par mètre cube", "/m³"),
        (" par kilomètre", "/km"),
        (" par mètre", "/m"),
        (" par seconde", "/s"),
        (" par heure", "/h"),
        (" par minute", "/min"),
        (" par litre", "/l"),
    ];

    for (spoken, symbol) in &rate_units {
        if input.ends_with(spoken) {
            let num_part = input.strip_suffix(spoken)?.trim();
            let num_value = parse_number_value(num_part)?;
            return Some(format!("{} {}", num_value, symbol));
        }
    }

    None
}

/// Parse compound units like "kilomètres par heure" → "km/h"
fn parse_compound_unit(input: &str) -> Option<String> {
    let compound_units = [
        (" kilomètres par heure", "km/h"),
        (" kilomètre par heure", "km/h"),
        (" kilomètres heure", "km/h"),
        (" kilomètre heure", "km/h"),
        (" mètres par seconde", "m/s"),
        (" mètre par seconde", "m/s"),
    ];

    for (spoken, symbol) in &compound_units {
        if input.ends_with(spoken) {
            let num_part = input.strip_suffix(spoken)?.trim();
            let num_value = parse_number_value(num_part)?;
            return Some(format!("{} {}", num_value, symbol));
        }
    }

    None
}

/// Parse simple measurement: number + unit (with optional modifier carré/cube)
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

/// Extract unit from end of string (includes modifier handling)
fn extract_unit(input: &str) -> Option<(&str, String)> {
    // Try units with modifiers first (most specific)
    for (spoken, symbol) in get_modifier_unit_mappings() {
        if input.ends_with(spoken) {
            let num_part = input.strip_suffix(spoken)?.trim();
            return Some((num_part, symbol.to_string()));
        }
    }

    // Then simple units
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
    if input.is_empty() {
        return None;
    }

    // Handle "zéro"/"zero"
    if input == "zéro" || input == "zero" {
        return Some("0".to_string());
    }

    // Try decimal first (has "virgule")
    if input.contains("virgule") {
        return decimal::parse(input);
    }

    // Cardinal number
    let num = words_to_number(input)?;
    let n = num as i64;

    // Format large numbers with spaces
    Some(format_with_spaces(n))
}

/// Format number with French space separators for thousands
fn format_with_spaces(n: i64) -> String {
    let abs_n = n.unsigned_abs();
    let s = abs_n.to_string();

    if s.len() <= 3 {
        return if n < 0 { format!("-{}", s) } else { s };
    }

    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }

    if n < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// Unit mappings with modifiers (squared, cubed)
fn get_modifier_unit_mappings() -> Vec<(&'static str, &'static str)> {
    vec![
        // Squared/Cubed variants (must be before simple)
        (" kilomètres carrés", "km²"),
        (" kilomètre carré", "km²"),
        (" mètres carrés", "m²"),
        (" mètre carré", "m²"),
        (" centimètres carrés", "cm²"),
        (" centimètre carré", "cm²"),
        (" mètres cubes", "m³"),
        (" mètre cube", "m³"),
        (" centimètres cubes", "cm³"),
        (" centimètre cube", "cm³"),
    ]
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
        (" micromètres", "µm"),
        (" micromètre", "µm"),
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
        assert_eq!(parse("trois cents micromètres"), Some("300 µm".to_string()));
    }

    #[test]
    fn test_speed() {
        assert_eq!(
            parse("cent kilomètres par heure"),
            Some("100 km/h".to_string())
        );
        assert_eq!(
            parse("deux-cents kilomètres heure"),
            Some("200 km/h".to_string())
        );
    }

    #[test]
    fn test_squared_cubed() {
        assert_eq!(
            parse("soixante-cinq kilomètres carrés"),
            Some("65 km²".to_string())
        );
        assert_eq!(parse("deux mètres cubes"), Some("2 m³".to_string()));
    }

    #[test]
    fn test_rate() {
        assert_eq!(
            parse("cinquante-six virgule trois par kilomètre carré"),
            Some("56,3 /km²".to_string())
        );
    }

    #[test]
    fn test_weight() {
        assert_eq!(parse("deux kilogrammes"), Some("2 kg".to_string()));
        assert_eq!(parse("cinquante grammes"), Some("50 g".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("moins soixante-six kilogrammes"),
            Some("-66 kg".to_string())
        );
    }

    #[test]
    fn test_decimal_measure() {
        assert_eq!(
            parse("dix-huit virgule cinq kilomètres"),
            Some("18,5 km".to_string())
        );
    }
}
