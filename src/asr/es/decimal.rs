//! Decimal number tagger for Spanish.
//!
//! Converts spoken Spanish decimal numbers to written form:
//! - "uno coma dos seis" → "1,26"
//! - "tres coma catorce quince noventa y dos sesenta y cinco tres" → "3,141592653"
//! - "uno punto treinta y tres millones" → "1.33 millones"

use super::cardinal;

/// Scale words that should be preserved as suffixes
const SCALE_WORDS: &[&str] = &[
    "millón",
    "millones",
    "millardo",
    "millardos",
    "billón",
    "billones",
    "trillón",
    "trillones",
    "cuatrillón",
    "cuatrillones",
];

/// Parse spoken Spanish decimal number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("menos ") {
        (true, input_trim.strip_prefix("menos ")?)
    } else {
        (false, input_trim)
    };

    let sign = if is_negative { "-" } else { "" };

    // Try "X coma Y [scale]" pattern
    if let Some(result) = parse_coma(rest) {
        return Some(format!("{}{}", sign, result));
    }

    // Try "X punto Y [scale]" pattern
    if let Some(result) = parse_punto(rest) {
        return Some(format!("{}{}", sign, result));
    }

    // Try "punto Y" pattern (no integer part)
    if let Some(result) = parse_punto_only(rest) {
        return Some(format!("{}{}", sign, result));
    }

    // Try scale-only: "un millón" → "1 millón", "dos millones" → "2 millones"
    if let Some(result) = parse_scale_only(rest) {
        return Some(format!("{}{}", sign, result));
    }

    // Try "NUMBER scale" → "N scale" (e.g., "mil ochocientos veinticuatro millones" → "1824 millones")
    if let Some(result) = parse_number_scale(rest) {
        return Some(format!("{}{}", sign, result));
    }

    None
}

/// Parse "X coma Y [scale]"
fn parse_coma(input: &str) -> Option<String> {
    let coma_pos = input.find(" coma ")?;
    let int_part = &input[..coma_pos];
    let after_coma = &input[coma_pos + 6..];

    let int_val = parse_integer_part(int_part)?;

    // Check for scale suffix
    let (dec_str, scale) = extract_scale_suffix(after_coma);

    let dec_digits = parse_decimal_part(dec_str.trim())?;

    let result = if let Some(sw) = scale {
        format!("{},{} {}", int_val, dec_digits, sw)
    } else {
        format!("{},{}", int_val, dec_digits)
    };

    Some(result)
}

/// Parse "X punto Y [scale]"
fn parse_punto(input: &str) -> Option<String> {
    let punto_pos = input.find(" punto ")?;
    let int_part = &input[..punto_pos];
    let after_punto = &input[punto_pos + 7..];

    let int_val = parse_integer_part(int_part)?;

    // Check for scale suffix
    let (dec_str, scale) = extract_scale_suffix(after_punto);

    let dec_digits = parse_decimal_part(dec_str.trim())?;

    let result = if let Some(sw) = scale {
        format!("{}.{} {}", int_val, dec_digits, sw)
    } else {
        format!("{}.{}", int_val, dec_digits)
    };

    Some(result)
}

/// Parse "punto Y" (no integer part)
fn parse_punto_only(input: &str) -> Option<String> {
    if !input.starts_with("punto ") {
        return None;
    }
    let after = &input[6..];
    let dec_digits = parse_decimal_part(after.trim())?;
    Some(format!(".{}", dec_digits))
}

/// Parse scale-only: "un millón" → "1 millón"
fn parse_scale_only(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() != 2 {
        return None;
    }
    let num_word = tokens[0];
    let scale_word = tokens[1];

    if !SCALE_WORDS.contains(&scale_word) {
        return None;
    }

    let num = parse_integer_part(num_word)?;
    Some(format!("{} {}", num, scale_word))
}

/// Parse "NUMBER scale" → "N scale"
fn parse_number_scale(input: &str) -> Option<String> {
    for &sw in SCALE_WORDS {
        if input.ends_with(sw) {
            let before = input[..input.len() - sw.len()].trim();
            if before.is_empty() {
                continue;
            }
            // Must have multiple tokens (not just "un millón" which is handled above)
            if !before.contains(' ') {
                continue;
            }
            let num = cardinal::words_to_number(before)?;
            return Some(format!("{} {}", num, sw));
        }
    }
    None
}

/// Parse the integer part of a decimal
fn parse_integer_part(input: &str) -> Option<i128> {
    let trimmed = input.trim();
    if trimmed == "cero" {
        return Some(0);
    }
    cardinal::words_to_number(trimmed)
}

/// Parse decimal digits from Spanish words.
/// Handles mixed individual digits and compound numbers:
/// "catorce quince noventa y dos sesenta y cinco tres" → "141592653"
///
/// Each group is parsed as the largest compound number possible
/// (hundreds+tens+units, tens+units, teens, or single digits)
/// and its string representation is concatenated.
fn parse_decimal_part(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut result = String::new();
    let mut i = 0;

    while i < tokens.len() {
        let t = tokens[i];

        // Try hundreds: "ciento cuarenta y uno" → 141, "novecientos veintiséis" → 926
        if let Some(hundred_base) = try_parse_hundred(t) {
            let mut val = hundred_base;
            let mut j = i + 1;

            if j < tokens.len() {
                // "ciento cuarenta y uno" — tens word follows
                if let Some(&tv) = lazy_static_tens(tokens[j]) {
                    val += tv;
                    j += 1;
                    // Check for "y UNIT"
                    if j + 1 < tokens.len() && tokens[j] == "y" {
                        if let Some(uv) = try_parse_unit(tokens[j + 1]) {
                            val += uv;
                            j += 2;
                        }
                    }
                }
                // "novecientos veintiséis" — compound teen/veinti- follows
                else if let Some(sv) = try_parse_single(tokens[j]) {
                    if sv >= 1 && sv <= 29 {
                        val += sv;
                        j += 1;
                    }
                }
                // "ciento y uno" — "y" directly follows hundreds
                else if tokens[j] == "y" && j + 1 < tokens.len() {
                    if let Some(uv) = try_parse_unit(tokens[j + 1]) {
                        val += uv;
                        j += 2;
                    }
                }
            }

            result.push_str(&val.to_string());
            i = j;
            continue;
        }

        // Try "TENS y UNIT": "treinta y tres" → 33, "noventa y dos" → 92
        if let Some(&tens_val) = lazy_static_tens(t) {
            if i + 2 < tokens.len() && tokens[i + 1] == "y" {
                if let Some(unit_val) = try_parse_unit(tokens[i + 2]) {
                    let compound = tens_val + unit_val;
                    result.push_str(&compound.to_string());
                    i += 3;
                    continue;
                }
            }
            // Tens alone: "treinta" → 30
            result.push_str(&tens_val.to_string());
            i += 1;
            continue;
        }

        // Single digit, teen, or veinti- compound
        if let Some(val) = try_parse_single(t) {
            result.push_str(&val.to_string());
            i += 1;
            continue;
        }

        return None;
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn lazy_static_tens(word: &str) -> Option<&i64> {
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    lazy_static! {
        static ref TENS_MAP: HashMap<&'static str, i64> = {
            let mut m = HashMap::new();
            m.insert("treinta", 30);
            m.insert("cuarenta", 40);
            m.insert("cincuenta", 50);
            m.insert("sesenta", 60);
            m.insert("setenta", 70);
            m.insert("ochenta", 80);
            m.insert("noventa", 90);
            m
        };
    }
    TENS_MAP.get(word)
}

fn try_parse_unit(word: &str) -> Option<i64> {
    match word {
        "uno" | "un" | "una" => Some(1),
        "dos" => Some(2),
        "tres" => Some(3),
        "cuatro" => Some(4),
        "cinco" => Some(5),
        "seis" => Some(6),
        "siete" => Some(7),
        "ocho" => Some(8),
        "nueve" => Some(9),
        _ => None,
    }
}

fn try_parse_hundred(word: &str) -> Option<i64> {
    match word {
        "ciento" | "cien" => Some(100),
        "doscientos" | "doscientas" => Some(200),
        "trescientos" | "trescientas" => Some(300),
        "cuatrocientos" | "cuatrocientas" => Some(400),
        "quinientos" | "quinientas" => Some(500),
        "seiscientos" | "seiscientas" => Some(600),
        "setecientos" | "setecientas" => Some(700),
        "ochocientos" | "ochocientas" => Some(800),
        "novecientos" | "novecientas" => Some(900),
        _ => None,
    }
}

fn try_parse_single(word: &str) -> Option<i64> {
    match word {
        "cero" => Some(0),
        "uno" | "un" | "una" => Some(1),
        "dos" => Some(2),
        "tres" => Some(3),
        "cuatro" => Some(4),
        "cinco" => Some(5),
        "seis" => Some(6),
        "siete" => Some(7),
        "ocho" => Some(8),
        "nueve" => Some(9),
        "diez" => Some(10),
        "once" => Some(11),
        "doce" => Some(12),
        "trece" => Some(13),
        "catorce" => Some(14),
        "quince" => Some(15),
        "dieciséis" => Some(16),
        "diecisiete" => Some(17),
        "dieciocho" => Some(18),
        "diecinueve" => Some(19),
        "veinte" => Some(20),
        "veintiún" | "veintiuno" => Some(21),
        "veintidós" => Some(22),
        "veintitrés" => Some(23),
        "veinticuatro" => Some(24),
        "veinticinco" => Some(25),
        "veintiséis" => Some(26),
        "veintisiete" => Some(27),
        "veintiocho" => Some(28),
        "veintinueve" => Some(29),
        _ => None,
    }
}

/// Extract scale suffix from end of string
fn extract_scale_suffix(input: &str) -> (&str, Option<&str>) {
    let trimmed = input.trim();
    for &sw in SCALE_WORDS {
        if trimmed.ends_with(sw) {
            let before = trimmed[..trimmed.len() - sw.len()].trim();
            return (before, Some(sw));
        }
    }
    (trimmed, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coma() {
        assert_eq!(parse("uno coma dos seis"), Some("1,26".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("menos uno coma dos seis"), Some("-1,26".to_string()));
    }

    #[test]
    fn test_punto() {
        assert_eq!(parse("uno punto treinta y tres"), Some("1.33".to_string()));
    }

    #[test]
    fn test_scale() {
        assert_eq!(parse("un millón"), Some("1 millón".to_string()));
        assert_eq!(parse("dos millones"), Some("2 millones".to_string()));
    }
}
