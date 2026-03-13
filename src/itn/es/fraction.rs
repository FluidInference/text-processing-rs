//! Fraction tagger for Spanish.
//!
//! Converts spoken Spanish fractions to written form:
//! - "ocho tercios" → "8/3"
//! - "dos y dos tercios" → "2 2/3"
//! - "menos diez veinteavos" → "-10/20"

use super::cardinal;

/// Parse spoken Spanish fraction to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Passthrough small fractions
    match input_trim {
        "medio" | "media" | "un medio" | "una media" => return Some(input_trim.to_string()),
        "un cuarto" | "una cuarta" => return Some(input_trim.to_string()),
        "un tercio" => return Some(input_trim.to_string()),
        _ => {}
    }

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("menos ") {
        (true, input_trim.strip_prefix("menos ")?)
    } else {
        (false, input_trim)
    };

    let sign = if is_negative { "-" } else { "" };

    // Try mixed fraction: "dos y dos tercios" → "2 2/3"
    if let Some(result) = parse_mixed_fraction(rest) {
        return Some(format!("{}{}", sign, result));
    }

    // Try simple fraction: "ocho tercios" → "8/3"
    if let Some(result) = parse_simple_fraction(rest) {
        return Some(format!("{}{}", sign, result));
    }

    None
}

/// Parse mixed fraction: "dos y dos tercios" → "2 2/3"
/// Pattern: "WHOLE y NUMER DENOM"
fn parse_mixed_fraction(input: &str) -> Option<String> {
    // Look for " y " separator for mixed fractions
    // "cuatro y un quinto" → whole=4, frac=1/5
    let y_pos = input.find(" y ")?;
    let whole_part = &input[..y_pos];
    let frac_part = &input[y_pos + 3..];

    // Try parsing frac_part as a simple fraction
    let frac = parse_simple_fraction(frac_part)?;

    // Parse whole part as a number
    let whole = cardinal::words_to_number(whole_part)?;

    Some(format!("{} {}", whole, frac))
}

/// Parse simple fraction: "ocho tercios" → "8/3"
fn parse_simple_fraction(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    let last = *tokens.last()?;

    // Parse denominator
    let denom = parse_denominator(last)?;

    // Parse numerator
    let numer_str = tokens[..tokens.len() - 1].join(" ");
    let numer = parse_numerator(&numer_str)?;

    Some(format!("{}/{}", numer, denom))
}

/// Parse numerator
fn parse_numerator(input: &str) -> Option<i128> {
    let trimmed = input.trim();
    if trimmed == "un" || trimmed == "una" || trimmed == "uno" {
        return Some(1);
    }
    cardinal::words_to_number(trimmed)
}

/// Parse denominator word to numeric value
fn parse_denominator(word: &str) -> Option<i128> {
    match word {
        "medio" | "media" | "medios" | "medias" => Some(2),
        "tercio" | "tercios" => Some(3),
        "cuarto" | "cuartos" | "cuarta" | "cuartas" => Some(4),
        "quinto" | "quintos" | "quinta" | "quintas" => Some(5),
        "sexto" | "sextos" => Some(6),
        "séptimo" | "séptimos" => Some(7),
        "octavo" | "octavos" => Some(8),
        "noveno" | "novenos" => Some(9),
        "décimo" | "décimos" => Some(10),
        "onceavo" | "onceavos" => Some(11),
        "doceavo" | "doceavos" => Some(12),
        "treceavo" | "treceavos" => Some(13),
        "catorceavo" | "catorceavos" => Some(14),
        "quinceavo" | "quinceavos" => Some(15),
        "dieciseisavo" | "dieciseisavos" => Some(16),
        "diecisieteavo" | "diecisieteavos" => Some(17),
        "dieciochoavo" | "dieciochoavos" => Some(18),
        "diecinueveavo" | "diecinueveavos" => Some(19),
        "veinteavo" | "veinteavos" => Some(20),
        "vigésimo" | "vigésimos" => Some(20),
        "treintavo" | "treintavos" => Some(30),
        "cuarentavo" | "cuarentavos" => Some(40),
        "cincuentavo" | "cincuentavos" => Some(50),
        _ => parse_compound_denominator(word),
    }
}

/// Parse compound denominator like "cientounavos" → 101, "cuarentiunavo" → 41
fn parse_compound_denominator(word: &str) -> Option<i128> {
    // Try stripping -avo/-avos/-ava/-avas suffix
    let stem = if let Some(s) = word.strip_suffix("avos") {
        s
    } else if let Some(s) = word.strip_suffix("avo") {
        s
    } else if let Some(s) = word.strip_suffix("avas") {
        s
    } else if let Some(s) = word.strip_suffix("ava") {
        s
    } else {
        return None;
    };

    // Try to parse the stem as a number
    // "cientoun" → "ciento un" → 101
    // "cuarentiun" → "cuarenta y un" → 41
    parse_denom_stem(stem)
}

/// Parse a denominator stem to a number
fn parse_denom_stem(stem: &str) -> Option<i128> {
    // Common compound patterns
    match stem {
        "cientoun" => Some(101),
        "cuarentiun" => Some(41),
        "treintaiun" | "treintaun" => Some(31),
        _ => {
            // Try splitting compound forms
            // "cientoun" already handled above
            // Try "ciento" + rest
            if stem.starts_with("ciento") {
                let rest = &stem[6..];
                let unit = parse_denom_unit(rest)?;
                return Some(100 + unit);
            }
            if stem.starts_with("cien") && stem.len() > 4 {
                let rest = &stem[4..];
                let unit = parse_denom_unit(rest)?;
                return Some(100 + unit);
            }
            None
        }
    }
}

fn parse_denom_unit(s: &str) -> Option<i128> {
    match s {
        "un" | "uno" | "una" => Some(1),
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
    fn test_simple() {
        assert_eq!(parse("ocho tercios"), Some("8/3".to_string()));
        assert_eq!(parse("dos quintos"), Some("2/5".to_string()));
    }

    #[test]
    fn test_passthrough() {
        assert_eq!(parse("medio"), Some("medio".to_string()));
        assert_eq!(parse("un cuarto"), Some("un cuarto".to_string()));
    }

    #[test]
    fn test_mixed() {
        assert_eq!(parse("dos y dos tercios"), Some("2 2/3".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("menos diez veinteavos"), Some("-10/20".to_string()));
    }

    #[test]
    fn test_compound_denom() {
        assert_eq!(parse("once cientounavos"), Some("11/101".to_string()));
        assert_eq!(parse("un cuarentiunavo"), Some("1/41".to_string()));
    }
}
