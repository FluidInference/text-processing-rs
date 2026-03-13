//! Fraction tagger for German.
//!
//! Converts spoken German fractions to written form:
//! - "ein halb" → "1/2"
//! - "ein drittel" → "1/3"
//! - "ein ein halb" → "1 1/2"
//! - "minus ein zwei und zwanzigstel" → "-1/22"

use super::cardinal;

/// Parse spoken German fraction to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("minus ") {
        (true, input_trim.strip_prefix("minus ")?)
    } else {
        (false, input_trim)
    };

    let sign = if is_negative { "-" } else { "" };

    // Try simple fraction first: "ein halb" → "1/2"
    // This also handles compound denominators: "ein zwei und zwanzigstel" → "1/22"
    // and "ein ein hundertstel" → "1/100" (compound denom "ein hundertstel" = 100)
    if let Some(result) = parse_simple_fraction(rest) {
        return Some(format!("{}{}", sign, result));
    }

    // Try mixed fraction: "ein ein halb" → "1 1/2"
    if let Some(result) = parse_mixed_fraction(rest) {
        return Some(format!("{}{}", sign, result));
    }

    None
}

/// Parse mixed fraction: "ein ein halb" → "1 1/2"
/// Only matches when the fraction part uses a simple (single-word) denominator.
/// Compound denominators like "ein hundertstel" are left to parse_simple_fraction
/// so that "ein ein hundertstel" parses as "1/100" (numer=1, denom="ein hundertstel"=100).
fn parse_mixed_fraction(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    // Only try mixed when the last token is a simple denominator word
    let last = *tokens.last()?;
    if parse_denominator(last).is_none() {
        return None;
    }

    // The fraction part is exactly 2 tokens: "NUMER DENOM"
    // E.g., "ein halb", "zwei drittel"
    if tokens.len() >= 3 {
        let frac_part = tokens[tokens.len() - 2..].join(" ");
        if let Some(frac) = parse_simple_fraction(&frac_part) {
            let whole_part = tokens[..tokens.len() - 2].join(" ");
            let whole = cardinal::words_to_number(&whole_part)?;
            return Some(format!("{} {}", whole, frac));
        }
    }

    None
}

/// Parse simple fraction: "ein halb" → "1/2", "vier halbe" → "4/2"
fn parse_simple_fraction(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let last = *tokens.last()?;
    let last_idx = tokens.len() - 1;

    // Try compound denominator FIRST (handles "ein hundertstel", "zwei und zwanzigstel")
    // This takes priority because "hundertstel" as a simple denom = 100, but
    // "ein hundertstel" as compound denom = 100 with the "ein" being part of the denom
    if last.ends_with("stel") || last.ends_with("halb") || last.ends_with("halbe")
        || last.ends_with("halbes") || last.ends_with("halber") || last.ends_with("halben") {
        // Try compound denominators with increasing scope
        for j in 1..=last_idx {
            let denom_str = tokens[j..].join(" ");
            if let Some(denom) = parse_compound_denominator(&denom_str) {
                let numer_tokens = &tokens[..j];
                if numer_tokens.is_empty() {
                    continue;
                }
                let numer_str = numer_tokens.join(" ");
                if let Some(numer) = parse_numerator(&numer_str) {
                    return Some(format!("{}/{}", numer, denom));
                }
            }
        }
    }

    // Simple denominator: last token is a known fraction word.
    // Only accept single-token numerators here to avoid "ein ein" → 2 misparse.
    // Multi-token numerators with simple denoms go through mixed fraction instead.
    if let Some(denom) = parse_denominator(last) {
        if last_idx == 1 {
            // Exactly one numerator token
            let numer_str = tokens[0];
            if let Some(numer) = parse_numerator(numer_str) {
                return Some(format!("{}/{}", numer, denom));
            }
        }
    }

    None
}

/// Parse a numerator (number word or "null")
fn parse_numerator(input: &str) -> Option<i128> {
    if input == "null" {
        return Some(0);
    }
    cardinal::words_to_number(input)
}

/// Parse a denominator word to its numeric value
fn parse_denominator(word: &str) -> Option<i128> {
    match word {
        "halb" | "halbe" | "halbes" | "halber" | "halben" | "halbem" => Some(2),
        "drittel" | "drittels" => Some(3),
        "viertel" | "viertels" => Some(4),
        "fünftel" | "fünftels" => Some(5),
        "sechstel" | "sechstels" => Some(6),
        "siebtel" | "siebtels" => Some(7),
        "achtel" | "achtels" => Some(8),
        "neuntel" | "neuntels" => Some(9),
        "zehntel" | "zehntels" => Some(10),
        "elftel" | "elftels" => Some(11),
        "zwölftel" | "zwölftels" => Some(12),
        "dreizehntel" => Some(13),
        "vierzehntel" => Some(14),
        "fünfzehntel" => Some(15),
        "sechzehntel" => Some(16),
        "siebzehntel" => Some(17),
        "achtzehntel" => Some(18),
        "neunzehntel" => Some(19),
        "zwanzigstel" => Some(20),
        "dreißigstel" | "dreissigstel" => Some(30),
        "vierzigstel" => Some(40),
        "fünfzigstel" => Some(50),
        "sechzigstel" => Some(60),
        "siebzigstel" => Some(70),
        "achtzigstel" => Some(80),
        "neunzigstel" => Some(90),
        "hundertstel" => Some(100),
        "nulltel" => Some(0),
        _ => None,
    }
}

/// Parse compound denominator: "zwei und zwanzigstel" → 22
/// Only handles multi-token denominators. Single-token denominators
/// are handled by parse_denominator in the simple path.
fn parse_compound_denominator(input: &str) -> Option<i128> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() <= 1 {
        return None;
    }

    // Pattern: "X und Ystel" → reconstruct number
    // E.g., "zwei und zwanzigstel" → "zwei und zwanzig" → 22
    let last = *tokens.last()?;

    // Try to extract the base number from the -stel suffix
    if let Some(stem) = last.strip_suffix("stel") {
        // Reconstruct: everything before last token + stem
        let mut num_parts: Vec<&str> = tokens[..tokens.len() - 1].to_vec();
        num_parts.push(stem);
        let num_str = num_parts.join(" ");
        return cardinal::words_to_number(&num_str);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fractions() {
        assert_eq!(parse("ein halb"), Some("1/2".to_string()));
        assert_eq!(parse("ein drittel"), Some("1/3".to_string()));
        assert_eq!(parse("ein viertel"), Some("1/4".to_string()));
        assert_eq!(parse("zwei neuntel"), Some("2/9".to_string()));
    }

    #[test]
    fn test_mixed() {
        assert_eq!(parse("ein ein halb"), Some("1 1/2".to_string()));
    }

    #[test]
    fn test_compound_denom() {
        assert_eq!(parse("ein zwei und zwanzigstel"), Some("1/22".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("minus ein zwei und zwanzigstel"), Some("-1/22".to_string()));
    }

    #[test]
    fn test_null() {
        assert_eq!(parse("null nulltel"), Some("0/0".to_string()));
    }
}
