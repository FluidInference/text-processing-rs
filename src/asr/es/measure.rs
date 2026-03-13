//! Measure tagger for Spanish.
//!
//! Converts spoken Spanish measurements to written form:
//! - "doscientos metros" → "200 m"
//! - "dos metros y medio" → "2 1/2 m"
//! - "dos más dos es igual a cuatro" → "2 + 2 = 4"

use super::cardinal;
use super::decimal;
use super::fraction;

struct UnitMapping {
    spoken: &'static [&'static str],
    written: &'static str,
}

const UNITS: &[UnitMapping] = &[
    UnitMapping { spoken: &["kilómetros por hora", "kilómetro por hora"], written: "kph" },
    UnitMapping { spoken: &["millas por hora", "milla por hora"], written: "mph" },
    UnitMapping { spoken: &["metros por hora", "metro por hora"], written: "m/h" },
    UnitMapping { spoken: &["metros cúbicos", "metro cúbico"], written: "m³" },
    UnitMapping { spoken: &["kilómetros", "kilómetro"], written: "km" },
    UnitMapping { spoken: &["centímetros", "centímetro"], written: "cm" },
    UnitMapping { spoken: &["milímetros", "milímetro"], written: "mm" },
    UnitMapping { spoken: &["metros", "metro"], written: "m" },
    UnitMapping { spoken: &["kilogramos", "kilogramo", "kilos", "kilo"], written: "kg" },
    UnitMapping { spoken: &["gramos", "gramo"], written: "g" },
    UnitMapping { spoken: &["litros", "litro"], written: "l" },
    UnitMapping { spoken: &["mililitros", "mililitro"], written: "ml" },
    UnitMapping { spoken: &["horas", "hora"], written: "h" },
    UnitMapping { spoken: &["segundos", "segundo"], written: "s" },
    UnitMapping { spoken: &["minutos", "minuto"], written: "min" },
    UnitMapping { spoken: &["grados farenheit", "grado farenheit"], written: "° F" },
    UnitMapping { spoken: &["grados celsius", "grado celsius"], written: "° C" },
    UnitMapping { spoken: &["grados", "grado"], written: "°" },
    UnitMapping { spoken: &["por ciento", "porciento"], written: "%" },
    UnitMapping { spoken: &["millas", "milla"], written: "mi" },
];

/// Parse spoken Spanish measurement to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Try math expression: "dos más dos es igual a cuatro"
    if let Some(result) = parse_math(input_trim) {
        return Some(result);
    }

    // Try fraction + unit: "dos metros y medio" → "2 1/2 m"
    if let Some(result) = parse_fraction_measure(input_trim) {
        return Some(result);
    }

    // Try "tres quintos de metro" → "3/5 m"
    if let Some(result) = parse_fraction_de_unit(input_trim) {
        return Some(result);
    }

    // Try decimal + unit: "sesenta coma dos cuatro cero cero kilogramos"
    if let Some(result) = parse_decimal_measure(input_trim) {
        return Some(result);
    }

    // Try simple: "doscientos metros" → "200 m"
    if let Some(result) = parse_simple_measure(input_trim) {
        return Some(result);
    }

    None
}

/// Parse math expression: "dos más dos es igual a cuatro" → "2 + 2 = 4"
fn parse_math(input: &str) -> Option<String> {
    if !input.contains(" es igual a ") {
        return None;
    }
    let parts: Vec<&str> = input.splitn(2, " es igual a ").collect();
    if parts.len() != 2 {
        return None;
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    // Parse right side
    let right_val = cardinal::words_to_number(right)?;

    // Parse left side: "X más Y" or "X menos Y" or "X por Y"
    if let Some(pos) = left.find(" más ") {
        let a = cardinal::words_to_number(&left[..pos])?;
        let b = cardinal::words_to_number(&left[pos + 5..])?;
        return Some(format!("{} + {} = {}", a, b, right_val));
    }
    if let Some(pos) = left.find(" menos ") {
        let a = cardinal::words_to_number(&left[..pos])?;
        let b = cardinal::words_to_number(&left[pos + 7..])?;
        return Some(format!("{} - {} = {}", a, b, right_val));
    }

    None
}

/// Parse fraction + unit: "dos metros y medio" → "2 1/2 m"
/// Also: "menos tres y medio metros por hora" → "-3 1/2 m/h"
fn parse_fraction_measure(input: &str) -> Option<String> {
    // Check for negative
    let (sign, rest) = if input.starts_with("menos ") {
        ("-", &input[6..])
    } else {
        ("", input)
    };

    for unit in UNITS {
        for &spoken in unit.spoken {
            // "X UNIT y medio" → "X 1/2 UNIT"
            let patterns = [
                (format!(" {} y medio", spoken), "1/2"),
                (format!(" {} y media", spoken), "1/2"),
            ];
            for (pattern, frac) in &patterns {
                if rest.ends_with(pattern.as_str()) {
                    let before = rest[..rest.len() - pattern.len()].trim();
                    let num = cardinal::words_to_number(before)?;
                    return Some(format!("{}{} {} {}", sign, num, frac, unit.written));
                }
            }

            // "X y medio UNIT" → "X 1/2 UNIT"
            if rest.ends_with(spoken) {
                let before = rest[..rest.len() - spoken.len()].trim();
                if before.ends_with(" y medio") || before.ends_with(" y media") {
                    let num_part = if before.ends_with(" y medio") {
                        &before[..before.len() - 8]
                    } else {
                        &before[..before.len() - 8]
                    };
                    let num = cardinal::words_to_number(num_part.trim())?;
                    return Some(format!("{}{} 1/2 {}", sign, num, unit.written));
                }
            }
        }
    }
    None
}

/// Parse "tres quintos de metro" → "3/5 m"
fn parse_fraction_de_unit(input: &str) -> Option<String> {
    for unit in UNITS {
        for &spoken in unit.spoken {
            let de_pattern = format!(" de {}", spoken);
            if input.ends_with(&de_pattern) {
                let before = input[..input.len() - de_pattern.len()].trim();
                if let Some(frac) = fraction::parse(before) {
                    return Some(format!("{} {}", frac, unit.written));
                }
            }
        }
    }
    None
}

/// Parse decimal + unit: "sesenta coma dos cuatro cero cero kilogramos" → "60,2400 kg"
fn parse_decimal_measure(input: &str) -> Option<String> {
    if !input.contains(" coma ") {
        return None;
    }

    for unit in UNITS {
        for &spoken in unit.spoken {
            if input.ends_with(spoken) {
                let before = input[..input.len() - spoken.len()].trim();
                if let Some(dec_result) = decimal::parse(before) {
                    return Some(format!("{} {}", dec_result, unit.written));
                }
            }
        }
    }
    None
}

/// Parse simple measure: "doscientos metros" → "200 m"
fn parse_simple_measure(input: &str) -> Option<String> {
    // Check for negative
    let (sign, rest) = if input.starts_with("menos ") {
        ("-", &input[6..])
    } else {
        ("", input)
    };

    for unit in UNITS {
        for &spoken in unit.spoken {
            if rest.ends_with(spoken) {
                let before = rest[..rest.len() - spoken.len()].trim();
                if before.is_empty() {
                    continue;
                }
                // Handle "una hora" → "1 h" (feminine)
                let num = if before == "una" || before == "un" {
                    1
                } else {
                    cardinal::words_to_number(before)? as i64
                };
                return Some(format!("{}{} {}", sign, num, unit.written));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(parse("doscientos metros"), Some("200 m".to_string()));
        assert_eq!(parse("una hora"), Some("1 h".to_string()));
    }

    #[test]
    fn test_fraction() {
        assert_eq!(parse("dos metros y medio"), Some("2 1/2 m".to_string()));
    }

    #[test]
    fn test_math() {
        assert_eq!(parse("dos más dos es igual a cuatro"), Some("2 + 2 = 4".to_string()));
    }
}
