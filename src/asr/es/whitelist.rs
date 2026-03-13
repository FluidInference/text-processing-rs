//! Whitelist tagger for Spanish.
//!
//! Maps spoken Spanish titles and phrases to abbreviations:
//! - "doctor" → "Dr."
//! - "señor" → "Sr."
//! - "por ejemplo" → "p.ej."

use lazy_static::lazy_static;

lazy_static! {
    static ref WHITELIST: Vec<(&'static str, &'static str)> = vec![
        ("por ejemplo", "p.ej."),
        ("etcétera", "etc."),
        ("doctor", "Dr."),
        ("doctora", "Dra."),
        ("señor", "Sr."),
        ("señora", "Sra."),
        ("señorita", "Srta."),
        ("usted", "Ud."),
        ("ustedes", "Uds."),
    ];
}

/// Parse spoken Spanish whitelist expression.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    for &(spoken, abbrev) in WHITELIST.iter() {
        if input_trim == spoken {
            return Some(abbrev.to_string());
        }
        // Multi-word: check if input starts with spoken phrase
        if input_trim.starts_with(spoken) {
            let rest = input_trim[spoken.len()..].trim_start();
            if rest.is_empty() {
                return Some(abbrev.to_string());
            }
            return Some(format!("{} {}", abbrev, rest));
        }
    }

    None
}
