//! Cardinal number tagger for Spanish.
//!
//! Converts spoken Spanish number words to digits:
//! - "doscientos cincuenta y uno" → "251"
//! - "un millón ciento cincuenta y seis mil" → "1156000"
//! - "menos veintitrés" → "-23"
//! - "mil millones uno" → "1000000001"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Numbers 0-29 (including veinti- compounds)
    static ref ONES: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("cero", 0);
        m.insert("uno", 1);
        m.insert("un", 1);
        m.insert("una", 1);
        m.insert("dos", 2);
        m.insert("tres", 3);
        m.insert("cuatro", 4);
        m.insert("cinco", 5);
        m.insert("seis", 6);
        m.insert("siete", 7);
        m.insert("ocho", 8);
        m.insert("nueve", 9);
        m.insert("diez", 10);
        m.insert("once", 11);
        m.insert("doce", 12);
        m.insert("trece", 13);
        m.insert("catorce", 14);
        m.insert("quince", 15);
        m.insert("dieciséis", 16);
        m.insert("diecisiete", 17);
        m.insert("dieciocho", 18);
        m.insert("diecinueve", 19);
        m.insert("veinte", 20);
        m.insert("veintiún", 21);
        m.insert("veintiuno", 21);
        m.insert("veintiuna", 21);
        m.insert("veintidós", 22);
        m.insert("veintitrés", 23);
        m.insert("veinticuatro", 24);
        m.insert("veinticinco", 25);
        m.insert("veintiséis", 26);
        m.insert("veintisiete", 27);
        m.insert("veintiocho", 28);
        m.insert("veintinueve", 29);
        m
    };

    /// Tens (30-90)
    static ref TENS: HashMap<&'static str, i64> = {
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

    /// Hundreds
    static ref HUNDREDS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("cien", 100);
        m.insert("ciento", 100);
        m.insert("doscientos", 200);
        m.insert("doscientas", 200);
        m.insert("trescientos", 300);
        m.insert("trescientas", 300);
        m.insert("cuatrocientos", 400);
        m.insert("cuatrocientas", 400);
        m.insert("quinientos", 500);
        m.insert("quinientas", 500);
        m.insert("seiscientos", 600);
        m.insert("seiscientas", 600);
        m.insert("setecientos", 700);
        m.insert("setecientas", 700);
        m.insert("ochocientos", 800);
        m.insert("ochocientas", 800);
        m.insert("novecientos", 900);
        m.insert("novecientas", 900);
        m
    };

    /// Scale words (Spanish long scale)
    static ref SCALES: HashMap<&'static str, i128> = {
        let mut m = HashMap::new();
        m.insert("mil", 1_000);
        m.insert("millón", 1_000_000);
        m.insert("millones", 1_000_000);
        m.insert("millardo", 1_000_000_000);
        m.insert("millardos", 1_000_000_000);
        m.insert("billón", 1_000_000_000_000);
        m.insert("billones", 1_000_000_000_000);
        m.insert("trillón", 1_000_000_000_000_000_000);
        m.insert("trillones", 1_000_000_000_000_000_000);
        m.insert("cuatrillón", 1_000_000_000_000_000_000_000_000);
        m.insert("cuatrillones", 1_000_000_000_000_000_000_000_000);
        m
    };

    /// Small numbers that pass through as words (0-9)
    static ref PASSTHROUGH: Vec<&'static str> = vec![
        "cero", "uno", "una", "dos", "tres", "cuatro",
        "cinco", "seis", "siete", "ocho", "nueve",
    ];
}

/// Parse spoken Spanish cardinal number to string representation.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    if input_trim.is_empty() {
        return None;
    }

    // Handle ", X" passthrough patterns
    if input_trim.starts_with(", ") {
        let rest = &input_trim[2..];
        if PASSTHROUGH.contains(&rest) {
            return Some(format!(", {}", rest));
        }
        // Try to parse as a number
        if let Some(num) = words_to_number(rest) {
            return Some(format!(", {}", num));
        }
        return None;
    }

    // Handle "entre X y Y" pattern
    if input_trim.starts_with("entre ") && input_trim.contains(" y ") {
        return parse_entre(input_trim);
    }

    // Pass-through single small numbers (0-9)
    if PASSTHROUGH.contains(&input_trim) {
        return Some(input_trim.to_string());
    }

    // Don't parse space-separated sequences that look like phone digit sequences.
    // Require at least one "heavy" structural word (hundreds, scales) for long inputs,
    // or any structural word for shorter inputs.
    if input_trim.contains(' ') {
        if !contains_structure_word(input_trim) {
            return None;
        }
        // Long inputs (4+ tokens excluding "y") without heavy structure are likely phone numbers.
        // E.g., "uno veintitrés cincuenta y seis setenta y ocho" is a phone number, not 182.
        let non_y_tokens: Vec<&str> = input_trim
            .split_whitespace()
            .filter(|t| *t != "y")
            .collect();
        if non_y_tokens.len() >= 4 && !contains_heavy_structure(input_trim) {
            return None;
        }
    }

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("menos ") {
        (true, &input_trim[6..])
    } else {
        (false, input_trim)
    };

    let num = words_to_number(rest)?;

    if is_negative {
        Some(format!("-{}", num))
    } else {
        Some(num.to_string())
    }
}

/// Parse "entre X y Y" → "entre N1 y N2"
fn parse_entre(input: &str) -> Option<String> {
    let rest = &input[6..]; // after "entre "
    let y_pos = rest.find(" y ")?;
    let first = &rest[..y_pos];
    let second = &rest[y_pos + 3..];

    let n1 = words_to_number(first)?;
    let n2 = words_to_number(second)?;

    Some(format!("entre {} y {}", n1, n2))
}

/// Check if input contains structure words that indicate a compound number
/// (not just a list of digit words)
fn contains_structure_word(input: &str) -> bool {
    let structure_words = [
        "cien",
        "ciento",
        "doscientos",
        "doscientas",
        "trescientos",
        "trescientas",
        "cuatrocientos",
        "cuatrocientas",
        "quinientos",
        "quinientas",
        "seiscientos",
        "seiscientas",
        "setecientos",
        "setecientas",
        "ochocientos",
        "ochocientas",
        "novecientos",
        "novecientas",
        "mil",
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
        "y",
        "menos",
        "entre",
        // veinti- compounds and tens are considered structure too
        "diez",
        "once",
        "doce",
        "trece",
        "catorce",
        "quince",
        "dieciséis",
        "diecisiete",
        "dieciocho",
        "diecinueve",
        "veinte",
        "veintiún",
        "veintiuno",
        "veintiuna",
        "veintidós",
        "veintitrés",
        "veinticuatro",
        "veinticinco",
        "veintiséis",
        "veintisiete",
        "veintiocho",
        "veintinueve",
        "treinta",
        "cuarenta",
        "cincuenta",
        "sesenta",
        "setenta",
        "ochenta",
        "noventa",
    ];
    let tokens: Vec<&str> = input.split_whitespace().collect();
    tokens.iter().any(|t| structure_words.contains(t))
}

/// Check if input contains "heavy" structure words: hundreds or scale words.
/// These are required for longer multi-word inputs to distinguish from phone numbers.
fn contains_heavy_structure(input: &str) -> bool {
    let heavy_words = [
        "cien",
        "ciento",
        "doscientos",
        "doscientas",
        "trescientos",
        "trescientas",
        "cuatrocientos",
        "cuatrocientas",
        "quinientos",
        "quinientas",
        "seiscientos",
        "seiscientas",
        "setecientos",
        "setecientas",
        "ochocientos",
        "ochocientas",
        "novecientos",
        "novecientas",
        "mil",
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
    let tokens: Vec<&str> = input.split_whitespace().collect();
    tokens.iter().any(|t| heavy_words.contains(t))
}

/// Convert Spanish number words to a number value.
pub fn words_to_number(input: &str) -> Option<i128> {
    let input_trim = input.trim();
    if input_trim.is_empty() {
        return None;
    }

    // Handle "mil millones" as a special compound scale (= 10^9)
    // Replace "mil millones" with a placeholder before tokenizing
    let normalized = input_trim
        .replace("mil trillones", "MIL_TRILLONES")
        .replace("mil billones", "MIL_BILLONES")
        .replace("mil millones", "MIL_MILLONES");

    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    // Filter out "y" connectors (but keep the structure)
    let tokens: Vec<&str> = tokens.iter().filter(|&&t| t != "y").copied().collect();

    if tokens.is_empty() {
        return None;
    }

    let mut result: i128 = 0;
    let mut sub: i128 = 0; // current accumulator below scale

    for &token in &tokens {
        // Check for special compound scales
        if token == "MIL_MILLONES" {
            let multiplier = if sub == 0 { 1 } else { sub };
            result += multiplier * 1_000_000_000;
            sub = 0;
            continue;
        }
        if token == "MIL_BILLONES" {
            let multiplier = if sub == 0 { 1 } else { sub };
            result += multiplier * 1_000_000_000_000_000;
            sub = 0;
            continue;
        }
        if token == "MIL_TRILLONES" {
            let multiplier = if sub == 0 { 1 } else { sub };
            result += multiplier * 1_000_000_000_000_000_000_000;
            sub = 0;
            continue;
        }

        if let Some(&scale) = SCALES.get(token) {
            if scale == 1000 {
                // "mil": flush sub as multiplier for thousands
                if sub == 0 {
                    sub = 1;
                }
                sub *= 1000;
            } else {
                // millón+: flush sub as multiplier for this scale
                let multiplier = if sub == 0 { 1 } else { sub };
                result += multiplier * scale;
                sub = 0;
            }
        } else if let Some(&val) = HUNDREDS.get(token) {
            sub += val as i128;
        } else if let Some(&val) = ONES.get(token) {
            sub += val as i128;
        } else if let Some(&val) = TENS.get(token) {
            sub += val as i128;
        } else {
            return None; // Unknown token
        }
    }

    result += sub;

    if result == 0 {
        // Only return 0 if input was literally "cero"
        if input_trim == "cero" {
            return Some(0);
        }
        return None;
    }

    Some(result)
}

/// Convert a single digit word to its numeric value.
/// Used by electronic and telephone taggers.
pub fn word_to_digit(word: &str) -> Option<u8> {
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
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        assert_eq!(parse("cero"), Some("cero".to_string()));
        assert_eq!(parse("uno"), Some("uno".to_string()));
        assert_eq!(parse("nueve"), Some("nueve".to_string()));
    }

    #[test]
    fn test_basic() {
        assert_eq!(parse("diez"), Some("10".to_string()));
        assert_eq!(parse("cien"), Some("100".to_string()));
        assert_eq!(parse("doscientos cincuenta y uno"), Some("251".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("menos veintitrés"), Some("-23".to_string()));
    }

    #[test]
    fn test_large() {
        assert_eq!(parse("mil millones uno"), Some("1000000001".to_string()));
    }
}
