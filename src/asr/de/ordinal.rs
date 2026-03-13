//! Ordinal number tagger for German.
//!
//! Converts spoken German ordinal words to written form:
//! - "ein hundertste" → "100."
//! - "erster" → "erster" (pass-through for small ordinals)
//! - "dem ein tausendstem" → "dem 1000."

use super::cardinal;

/// Small ordinals that pass through as words (1-9)
const SMALL_ORDINALS: &[&str] = &[
    "nullte", "nullter", "nulltem", "nulltes",
    "erste", "erster", "erstem", "erstes",
    "zweite", "zweiter", "zweitem", "zweites",
    "dritte", "dritter", "drittem", "drittes",
    "vierte", "vierter", "viertem", "viertes",
    "fünfte", "fünfter", "fünftem", "fünftes",
    "sechste", "sechster", "sechstem", "sechstes",
    "siebte", "siebter", "siebtem", "siebtes",
    "achte", "achter", "achtem", "achtes",
    "neunte", "neunter", "neuntem", "neuntes",
];

/// Parse spoken German ordinal to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Pass-through small ordinals
    if SMALL_ORDINALS.contains(&input_trim) {
        return Some(input_trim.to_string());
    }

    // Check for prefix words: "dem ein tausendstem" → "dem 1000."
    let (prefix, ordinal_part) = extract_prefix(input_trim);

    // Try to parse the ordinal
    if let Some(num) = parse_ordinal_number(ordinal_part) {
        if let Some(p) = prefix {
            return Some(format!("{} {}.", p, num));
        }
        return Some(format!("{}.", num));
    }

    None
}

/// Extract prefix words (like "dem") from ordinal expression
fn extract_prefix(input: &str) -> (Option<&str>, &str) {
    let prefixes = ["dem ", "der ", "des ", "die ", "das ", "den ",
                    "am ", "im ", "vom ", "zum ", "beim "];

    for prefix in &prefixes {
        if input.starts_with(prefix) {
            let rest = &input[prefix.len()..];
            let p = input[..prefix.len() - 1].trim();
            return (Some(p), rest);
        }
    }

    (None, input)
}

/// Parse ordinal number from German ordinal word.
/// Returns the cardinal number if >= 10, None for small numbers.
fn parse_ordinal_number(input: &str) -> Option<i128> {
    // Strip ordinal suffix
    let ordinal_suffixes = ["stem", "stes", "ster", "ste",
                            "tem", "tes", "ter", "te"];

    for &suffix in &ordinal_suffixes {
        if input.ends_with(suffix) {
            let stem = &input[..input.len() - suffix.len()];
            let cardinal = reconstruct_cardinal(stem);
            if let Some(num) = cardinal::words_to_number(&cardinal) {
                if num >= 10 {
                    return Some(num);
                }
            }
        }
    }

    None
}

/// Reconstruct cardinal form from ordinal stem.
fn reconstruct_cardinal(stem: &str) -> String {
    // Handle special stems
    match stem {
        "er" | "ers" => "eins".to_string(),
        "zwei" => "zwei".to_string(),
        "drit" => "drei".to_string(),
        "vier" => "vier".to_string(),
        "fünf" => "fünf".to_string(),
        "sechs" => "sechs".to_string(),
        "sieb" => "sieben".to_string(),
        "ach" => "acht".to_string(),
        "neun" => "neun".to_string(),
        "zehn" => "zehn".to_string(),
        "elf" => "elf".to_string(),
        "zwölf" => "zwölf".to_string(),
        _ => {
            // For compound ordinals, return as-is (already cardinal form)
            // e.g., "ein hundert" from "ein hundertste"
            // "fünf und zwanzig tausend ein hundert elf" from that ordinal
            stem.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        assert_eq!(parse("erster"), Some("erster".to_string()));
        assert_eq!(parse("zweite"), Some("zweite".to_string()));
        assert_eq!(parse("dritter"), Some("dritter".to_string()));
    }

    #[test]
    fn test_large() {
        assert_eq!(parse("ein hundertste"), Some("100.".to_string()));
        assert_eq!(parse("ein tausendstem"), Some("1000.".to_string()));
    }

    #[test]
    fn test_with_prefix() {
        assert_eq!(parse("dem ein tausendstem"), Some("dem 1000.".to_string()));
    }

    #[test]
    fn test_teens() {
        assert_eq!(parse("zehnter"), Some("10.".to_string()));
        assert_eq!(parse("elftem"), Some("11.".to_string()));
        assert_eq!(parse("dreizehntem"), Some("13.".to_string()));
    }
}
