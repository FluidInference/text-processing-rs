//! Word tagger for Hindi.
//!
//! Pass-through: returns input unchanged.
//! Handles words that should not be normalized.

/// Process word patterns (pass-through).
pub fn process(input: &str) -> String {
    input.to_string()
}
