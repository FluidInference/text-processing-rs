//! Ordinal number tagger for Spanish.
//!
//! Converts spoken Spanish ordinals to written form:
//! - "primero" → "primero" (small ordinals stay as words)
//! - "décimo" → "10.º"
//! - "vigésimo primero" → "21.º"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Small ordinals that pass through as words
    static ref PASSTHROUGH: Vec<&'static str> = vec![
        "primero", "primera", "primer", "segundo", "segunda",
        "tercero", "tercera", "tercer", "cuarto", "cuarta",
        "quinto", "quinta", "sexto", "sexta",
        "séptimo", "séptima", "octavo", "octava",
        "noveno", "novena",
    ];

    /// Ordinal word → (value, gender) mappings
    /// Gender: 'm' = masculine, 'f' = feminine, 'r' = abbreviated masculine (ᵉʳ)
    static ref ORDINALS: HashMap<&'static str, (i64, char)> = {
        let mut m = HashMap::new();
        // Tens ordinals
        m.insert("décimo", (10, 'm'));
        m.insert("décima", (10, 'f'));
        m.insert("undécimo", (11, 'm'));
        m.insert("undécima", (11, 'f'));
        m.insert("duodécimo", (12, 'm'));
        m.insert("duodécima", (12, 'f'));
        m.insert("decimotercero", (13, 'm'));
        m.insert("decimotercera", (13, 'f'));
        m.insert("decimocuarto", (14, 'm'));
        m.insert("decimoquinto", (15, 'm'));
        m.insert("decimosexto", (16, 'm'));
        m.insert("decimoséptimo", (17, 'm'));
        m.insert("decimoctavo", (18, 'm'));
        m.insert("decimonoveno", (19, 'm'));
        m.insert("vigésimo", (20, 'm'));
        m.insert("vigésima", (20, 'f'));
        m.insert("vigesimosegundo", (22, 'm'));
        m.insert("vigesimosegunda", (22, 'f'));
        m.insert("vigesimoctavo", (28, 'm'));
        m.insert("trigésimo", (30, 'm'));
        m.insert("trigésima", (30, 'f'));
        m.insert("cuadragésimo", (40, 'm'));
        m.insert("quincuagésimo", (50, 'm'));
        m.insert("sexagésimo", (60, 'm'));
        m.insert("septuagésimo", (70, 'm'));
        m.insert("octogésimo", (80, 'm'));
        m.insert("nonagésimo", (90, 'm'));
        m.insert("centésimo", (100, 'm'));
        m.insert("centésima", (100, 'f'));
        // Compound forms that don't split
        m.insert("decimoprimero", (11, 'm'));
        m.insert("decimoprimera", (11, 'f'));
        m.insert("decimoprimer", (11, 'r'));
        m
    };

    /// Small ordinal components for compound ordinals
    static ref ORDINAL_UNITS: HashMap<&'static str, (i64, char)> = {
        let mut m = HashMap::new();
        m.insert("primero", (1, 'm'));
        m.insert("primera", (1, 'f'));
        m.insert("primer", (1, 'r'));
        m.insert("segundo", (2, 'm'));
        m.insert("segunda", (2, 'f'));
        m.insert("tercero", (3, 'm'));
        m.insert("tercera", (3, 'f'));
        m.insert("tercer", (3, 'r'));
        m.insert("cuarto", (4, 'm'));
        m.insert("cuarta", (4, 'f'));
        m.insert("quinto", (5, 'm'));
        m.insert("quinta", (5, 'f'));
        m.insert("sexto", (6, 'm'));
        m.insert("sexta", (6, 'f'));
        m.insert("séptimo", (7, 'm'));
        m.insert("séptima", (7, 'f'));
        m.insert("octavo", (8, 'm'));
        m.insert("octava", (8, 'f'));
        m.insert("noveno", (9, 'm'));
        m.insert("novena", (9, 'f'));
        m.insert("undécimo", (11, 'm'));
        m.insert("undécima", (11, 'f'));
        m
    };
}

/// Parse spoken Spanish ordinal to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Handle prefix text like "(technically ungrammatical)"
    let (prefix, ordinal_part) = extract_prefix(input_trim);

    // Check passthrough
    if prefix.is_none() && PASSTHROUGH.contains(&ordinal_part) {
        return Some(ordinal_part.to_string());
    }

    // Try single-word ordinal
    if let Some(&(val, gender)) = ORDINALS.get(ordinal_part) {
        let suffix = gender_suffix(gender);
        let result = format!("{}{}", val, suffix);
        return Some(with_prefix(prefix, &result));
    }

    // Try multi-word compound ordinals: "vigésimo primero", "centésimo trigésimo cuarto"
    let tokens: Vec<&str> = ordinal_part.split_whitespace().collect();
    if tokens.len() >= 2 {
        let mut total: i64 = 0;
        let mut last_gender = 'm';

        for &token in &tokens {
            if let Some(&(val, g)) = ORDINALS.get(token) {
                total += val;
                last_gender = g;
            } else if let Some(&(val, g)) = ORDINAL_UNITS.get(token) {
                total += val;
                last_gender = g;
            } else {
                return None;
            }
        }

        if total > 0 {
            let suffix = gender_suffix(last_gender);
            let result = format!("{}{}", total, suffix);
            return Some(with_prefix(prefix, &result));
        }
    }

    None
}

/// Extract prefix like "(technically ungrammatical)" from ordinal input
fn extract_prefix(input: &str) -> (Option<String>, &str) {
    // Check for parenthesized prefix
    if input.starts_with('(') {
        if let Some(close) = input.find(')') {
            let prefix = &input[..close + 1];
            let rest = input[close + 1..].trim();
            return (Some(prefix.to_string()), rest);
        }
    }
    (None, input)
}

fn with_prefix(prefix: Option<String>, result: &str) -> String {
    if let Some(p) = prefix {
        format!("{} {}", p, result)
    } else {
        result.to_string()
    }
}

fn gender_suffix(gender: char) -> &'static str {
    match gender {
        'f' => ".ª",
        'r' => ".ᵉʳ",
        _ => ".º",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        assert_eq!(parse("primero"), Some("primero".to_string()));
        assert_eq!(parse("tercera"), Some("tercera".to_string()));
        assert_eq!(parse("noveno"), Some("noveno".to_string()));
    }

    #[test]
    fn test_simple() {
        assert_eq!(parse("décimo"), Some("10.º".to_string()));
        assert_eq!(parse("undécima"), Some("11.ª".to_string()));
    }

    #[test]
    fn test_compound() {
        assert_eq!(parse("vigésimo primero"), Some("21.º".to_string()));
        assert_eq!(
            parse("centésimo trigésimo cuarto"),
            Some("134.º".to_string())
        );
    }
}
