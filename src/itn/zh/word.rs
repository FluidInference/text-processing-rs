//! Word tagger for Chinese ITN.
//!
//! Pass-through: returns input unchanged.
//! This module exists for completeness — the word test cases verify
//! that non-numeric Chinese text passes through unmodified.

/// Process word patterns (pass-through).
pub fn process(input: &str) -> String {
    input.to_string()
}
