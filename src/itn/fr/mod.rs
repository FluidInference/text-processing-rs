//! Inverse Text Normalization taggers for French.
//!
//! Converts spoken-form French to written form:
//! - "deux cents" → "200"
//! - "cinq euros et cinquante centimes" → "5,50 €"
//! - "cinq janvier deux mille vingt-cinq" → "5 janvier 2025"

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod electronic;
pub mod fraction;
pub mod measure;
pub mod money;
pub mod ordinal;
pub mod punctuation;
pub mod telephone;
pub mod time;
pub mod whitelist;
pub mod word;
