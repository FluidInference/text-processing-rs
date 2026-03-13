//! Word tagger for German.
//!
//! Handles pass-through words and special cases:
//! - "yahoo!" → "yahoo!" (pass-through)
//! - "zwanzig!" → "20 !" (cardinal + punctuation)
//! - Regular words pass through unchanged

/// Parse is not used directly - word handling is done in the normalize pipeline.
/// This module exists for symmetry with the French implementation.
pub fn parse(_input: &str) -> Option<String> {
    // Word tagger doesn't actively transform anything.
    // Pass-through and "cardinal!" patterns are handled by normalize_lang_de.
    None
}
