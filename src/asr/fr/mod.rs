//! Inverse Text Normalization taggers for French.
//!
//! Converts spoken-form French to written form:
//! - "deux cents" → "200"
//! - "cinq euros et cinquante centimes" → "5,50 €"
//! - "cinq janvier deux mille vingt-cinq" → "5 janvier 2025"

pub mod cardinal;
