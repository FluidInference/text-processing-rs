//! Punctuation tagger for Spanish.
//!
//! Converts spoken Spanish punctuation words to symbols:
//! - "punto" → "."
//! - "coma" → ","
//! - "signo de interrogación" → "?"

use lazy_static::lazy_static;

lazy_static! {
    static ref PUNCTUATION: Vec<(&'static str, &'static str)> = vec![
        ("signo de interrogación", "?"),
        ("signo de exclamación", "!"),
        ("dos puntos", ":"),
        ("punto y coma", ";"),
        ("punto", "."),
        ("coma", ","),
        ("guión", "-"),
    ];
}

/// Parse spoken Spanish punctuation to symbol.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    for &(spoken, symbol) in PUNCTUATION.iter() {
        if input_trim == spoken {
            return Some(symbol.to_string());
        }
    }

    None
}
