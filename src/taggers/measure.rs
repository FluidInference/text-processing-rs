//! Measure tagger.
//!
//! Converts spoken measurements to written form:
//! - "two hundred meters" → "200 m"
//! - "eighteen point five kilometers" → "18.5 km"
//! - "two hundred kilometers per hour" → "200 km/h"
//! - "thirty one thousand square feet" → "31000 sq ft"

use super::cardinal::words_to_number;
use super::decimal;

/// Parse spoken measurement expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input = input.to_lowercase();
    let input = input.trim();

    // Try compound units first (most specific)
    if let Some(result) = parse_compound_unit(input) {
        return Some(result);
    }

    // Try simple unit
    if let Some(result) = parse_simple_unit(input) {
        return Some(result);
    }

    None
}

/// Parse compound units like "kilometers per hour" → "km/h"
fn parse_compound_unit(input: &str) -> Option<String> {
    // Special case: "X miles per hour" → "X mph"
    if input.ends_with(" miles per hour") {
        let num_part = input.strip_suffix(" miles per hour")?;
        let num_value = parse_number_value(num_part.trim())?;
        return Some(format!("{} mph", num_value));
    }

    // Special case: "X kilograms force per square centimeter" → "X kgf/cm²"
    if input.ends_with(" kilograms force per square centimeter") {
        let num_part = input.strip_suffix(" kilograms force per square centimeter")?;
        let num_value = parse_number_value(num_part.trim())?;
        return Some(format!("{} kgf/cm²", num_value));
    }

    // Special case: "X per square Y" without unit (e.g., "fifty six per square kilometer")
    if let Some(idx) = input.find(" per square ") {
        let num_part = &input[..idx];
        let denom_part = &input[idx + 12..]; // " per square " is 12 chars

        // Parse numerator (just number, no unit)
        let num_value = parse_number_value(num_part.trim())?;
        let denom_unit = get_unit_symbol(denom_part)?;

        return Some(format!("{} /{}²", num_value, denom_unit));
    }

    // "X per cubic Y" pattern
    if let Some(idx) = input.find(" per cubic ") {
        let num_part = &input[..idx];
        let denom_part = &input[idx + 11..];

        let num_value = parse_number_value(num_part.trim())?;
        let denom_unit = get_unit_symbol(denom_part)?;

        return Some(format!("{} /{}³", num_value, denom_unit));
    }

    // "X unit per Y" pattern (e.g., "kilometers per hour")
    if let Some(idx) = input.find(" per ") {
        let num_unit_part = &input[..idx];
        let denom_part = &input[idx + 5..];

        // Try to parse as number + unit
        if let Some((num_value, num_unit)) = parse_number_and_unit(num_unit_part) {
            let denom_unit = get_unit_symbol(denom_part)?;
            return Some(format!("{} {}/{}", num_value, num_unit, denom_unit));
        }
    }

    None
}

/// Parse simple measurement: number + unit
fn parse_simple_unit(input: &str) -> Option<String> {
    let (value, unit) = parse_number_and_unit(input)?;
    Some(format!("{} {}", value, unit))
}

/// Parse number and unit from input, returning (formatted_number, unit_symbol)
fn parse_number_and_unit(input: &str) -> Option<(String, String)> {
    // Handle negative
    let (is_negative, rest) = if input.starts_with("minus ") {
        (true, input.strip_prefix("minus ")?)
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

/// Extract unit from end of string, return (number_part, unit_symbol)
fn extract_unit(input: &str) -> Option<(&str, String)> {
    // Check for "miles per hour" first - special case for mph
    if input.ends_with(" miles per hour") {
        let num_part = input.strip_suffix(" miles per hour")?;
        return Some((num_part, "mph".to_string()));
    }

    // Check for square/cubic prefixes
    let (prefix, rest, modifier) = if input.contains(" square ") {
        let idx = input.rfind(" square ")?;
        let after_square = &input[idx + 8..];
        (&input[..idx], after_square, "sq")
    } else if input.contains(" cubic ") {
        let idx = input.rfind(" cubic ")?;
        let after_cubic = &input[idx + 7..];
        (&input[..idx], after_cubic, "³")
    } else {
        (input, "", "")
    };

    // If we have a modifier (square/cubic), parse the unit from rest
    if !modifier.is_empty() {
        let unit = get_unit_symbol(rest)?;
        // Use "sq ft", "sq mi" format for imperial, "m²", "km²" for metric
        let formatted = if modifier == "sq" {
            match unit {
                "ft" => "sq ft".to_string(),
                "mi" => "sq mi".to_string(),
                _ => format!("{}²", unit),
            }
        } else {
            format!("{}{}", unit, modifier)
        };
        return Some((prefix, formatted));
    }

    // Try each unit pattern from longest to shortest
    for (spoken, symbol) in get_unit_mappings() {
        if input.ends_with(spoken) {
            let num_part = input.strip_suffix(spoken)?.trim();
            return Some((num_part, symbol.to_string()));
        }
    }

    None
}

/// Get unit symbol from spoken unit name
fn get_unit_symbol(unit_name: &str) -> Option<&'static str> {
    let unit_name = unit_name.trim();

    for (spoken, symbol) in get_unit_mappings() {
        // Remove leading space from spoken pattern for matching
        let spoken_trimmed = spoken.trim();
        if unit_name == spoken_trimmed || unit_name == spoken_trimmed.trim_end_matches('s') {
            return Some(symbol);
        }
    }

    // Handle singular/plural variations
    match unit_name {
        "meter" | "meters" => Some("m"),
        "kilometer" | "kilometers" => Some("km"),
        "centimeter" | "centimeters" => Some("cm"),
        "decimeter" | "decimeters" | "deci meter" | "deci meters" => Some("dm"),
        "millimeter" | "millimeters" => Some("mm"),
        "micrometer" | "micrometers" => Some("μm"),
        "nanometer" | "nanometers" => Some("nm"),
        "foot" | "feet" => Some("ft"),
        "mile" | "miles" => Some("mi"),
        "hour" | "hours" => Some("h"),
        "second" | "seconds" => Some("s"),
        "minute" | "minutes" => Some("min"),
        "gram" | "grams" => Some("g"),
        "kilogram" | "kilograms" => Some("kg"),
        "hectare" | "hectares" => Some("ha"),
        "liter" | "liters" | "litre" | "litres" => Some("l"),
        "milliliter" | "milliliters" => Some("ml"),
        _ => None,
    }
}

/// Get all unit mappings (spoken -> symbol)
/// Ordered from longest to shortest to match most specific first
fn get_unit_mappings() -> Vec<(&'static str, &'static str)> {
    vec![
        // Compound/special units (longest first)
        (" kilo watt hours", "kWh"),
        (" giga watt hours", "gWh"),
        (" mega watt hours", "MWh"),
        (" watt hours", "Wh"),
        (" kilograms force", "kgf"),
        (" astronomical units", "au"),
        (" miles per hour", "mph"),
        (" kilometers per hour", "km/h"),

        // Square/cubic variations
        (" square kilometers", "km²"),
        (" square kilometer", "km²"),
        (" square meters", "m²"),
        (" square meter", "m²"),
        (" square feet", "sq ft"),
        (" square foot", "sq ft"),
        (" square miles", "sq mi"),
        (" square mile", "sq mi"),
        (" cubic meters", "m³"),
        (" cubic meter", "m³"),
        (" cubic deci meters", "dm³"),
        (" cubic decimeters", "dm³"),

        // Data units
        (" peta bytes", "pb"),
        (" petabytes", "pb"),
        (" giga bytes", "gb"),
        (" gigabytes", "gb"),
        (" mega bytes", "mb"),
        (" megabytes", "mb"),
        (" kilo bytes", "kb"),
        (" kilobytes", "kb"),
        (" kilobits", "kb"),
        (" bytes", "b"),

        // Power/Energy
        (" megawatts", "mW"),
        (" megawatt", "mW"),
        (" kilowatts", "kW"),
        (" kilowatt", "kW"),
        (" gigawatts", "gW"),
        (" watts", "W"),
        (" watt", "W"),
        (" horsepower", "hp"),

        // Data rates
        (" gigabits per second", "gbps"),
        (" gigabit per second", "gbps"),
        (" megabits per second", "mbps"),
        (" megabit per second", "mbps"),

        // Temperature
        (" degrees celsius", "°C"),
        (" degree celsius", "°C"),
        (" degrees fahrenheit", "°F"),
        (" degree fahrenheit", "°F"),
        (" kelvin", "K"),

        // Frequency
        (" megahertz", "mhz"),
        (" kilohertz", "khz"),
        (" hertz", "hz"),

        // Electrical
        (" milli volt", "mv"),
        (" millivolts", "mv"),
        (" volts", "v"),
        (" volt", "v"),
        (" mega siemens", "ms"),

        // Length
        (" micrometers", "μm"),
        (" micrometer", "μm"),
        (" nanometers", "nm"),
        (" nanometer", "nm"),
        (" millimeters", "mm"),
        (" millimeter", "mm"),
        (" centimeters", "cm"),
        (" centimeter", "cm"),
        (" kilometers", "km"),
        (" kilometer", "km"),
        (" meters", "m"),
        (" meter", "m"),
        (" feet", "ft"),
        (" foot", "ft"),
        (" miles", "mi"),
        (" mile", "mi"),
        (" ounces", "oz"),
        (" ounce", "oz"),

        // Mass
        (" kilograms", "kg"),
        (" kilogram", "kg"),
        (" grams", "g"),
        (" gram", "g"),

        // Volume
        (" kilo liters", "kl"),
        (" milliliters", "ml"),
        (" milliliter", "ml"),
        (" liters", "l"),
        (" liter", "l"),
        (" c c", "cc"),

        // Area
        (" hectares", "ha"),
        (" hectare", "ha"),

        // Time
        (" hours", "h"),
        (" hour", "h"),

        // Light
        (" lumens", "lm"),
        (" lumen", "lm"),

        // Percent
        (" percent", "%"),
    ]
}

/// Parse number value (cardinal, decimal, or with "point")
fn parse_number_value(input: &str) -> Option<String> {
    // Try decimal first (handles "point" patterns)
    if input.contains(" point ") || input.starts_with("point ") {
        return decimal::parse(input);
    }

    // Try cardinal
    if let Some(num) = words_to_number(input) {
        return Some((num as i64).to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_units() {
        assert_eq!(parse("two hundred meters"), Some("200 m".to_string()));
        assert_eq!(parse("ninety grams"), Some("90 g".to_string()));
        assert_eq!(parse("three hours"), Some("3 h".to_string()));
    }

    #[test]
    fn test_decimal_units() {
        assert_eq!(
            parse("eighteen point five kilometers"),
            Some("18.5 km".to_string())
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("minus sixty six kilograms"), Some("-66 kg".to_string()));
    }

    #[test]
    fn test_square_units() {
        assert_eq!(parse("two square meters"), Some("2 m²".to_string()));
        assert_eq!(
            parse("sixty five thousand square kilometers"),
            Some("65000 km²".to_string())
        );
    }

    #[test]
    fn test_compound_units() {
        assert_eq!(
            parse("two hundred kilometers per hour"),
            Some("200 km/h".to_string())
        );
    }

    #[test]
    fn test_special_units() {
        assert_eq!(parse("two kilo watt hours"), Some("2 kWh".to_string()));
        assert_eq!(parse("one hundred fifty c c"), Some("150 cc".to_string()));
    }

    #[test]
    fn test_percent() {
        assert_eq!(
            parse("eighteen point one four percent"),
            Some("18.14 %".to_string())
        );
    }
}
