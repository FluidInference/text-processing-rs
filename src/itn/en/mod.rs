//! Inverse Text Normalization taggers for English.
//!
//! Converts spoken-form text to written English:
//! - "two hundred" → "200"
//! - "five dollars and fifty cents" → "$5.50"
//! - "january fifth twenty twenty five" → "January 5, 2025"

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
