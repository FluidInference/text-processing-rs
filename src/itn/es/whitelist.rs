//! Whitelist tagger for Spanish.
//!
//! Maps spoken Spanish titles and phrases to abbreviations:
//! - "doctor" → "Dr."
//! - "señor" → "Sr."
//! - "por ejemplo" → "p.ej."
//! - "ustedes" → "Uds."
//! - "estados unidos" → "EE. UU."

use lazy_static::lazy_static;

lazy_static! {
    /// Whitelist entries ordered longest-first to prevent prefix conflicts
    /// (e.g., "ustedes" must match before "usted").
    static ref WHITELIST: Vec<(&'static str, &'static str)> = vec![
        ("estados unidos", "EE. UU."),
        ("por ejemplo", "p.ej."),
        ("etcétera", "etc."),
        ("doctora", "Dra."),
        ("doctor", "Dr."),
        ("señorita", "Srta."),
        ("señora", "Sra."),
        ("señor", "Sr."),
        ("ustedes", "Uds."),
        ("usted", "Ud."),
    ];
}

/// Parse spoken Spanish whitelist expression.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Exact match
    for &(spoken, abbrev) in WHITELIST.iter() {
        if input_trim == spoken {
            return Some(abbrev.to_string());
        }
    }

    // Word-boundary match within sentences
    for &(spoken, abbrev) in WHITELIST.iter() {
        if let Some(result) = replace_word_boundary(input_trim, spoken, abbrev) {
            return Some(result);
        }
    }

    None
}

/// Replace `spoken` with `abbrev` in `input`, respecting word boundaries.
fn replace_word_boundary(input: &str, spoken: &str, abbrev: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(pos) = input[search_from..].find(spoken) {
        let abs_pos = search_from + pos;
        let end_pos = abs_pos + spoken.len();

        // Check word boundary before
        let start_ok = abs_pos == 0
            || input
                .as_bytes()
                .get(abs_pos - 1)
                .map_or(true, |&b| b == b' ' || b == b',' || b == b'.');

        // Check word boundary after
        let end_ok = end_pos == input.len()
            || input
                .as_bytes()
                .get(end_pos)
                .map_or(true, |&b| b == b' ' || b == b',' || b == b'.');

        if start_ok && end_ok {
            let mut result = String::new();
            result.push_str(&input[..abs_pos]);
            result.push_str(abbrev);
            result.push_str(&input[end_pos..]);
            return Some(result);
        }

        search_from = abs_pos + 1;
    }
    None
}
