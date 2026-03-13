//! Inverse Text Normalization taggers for German.
//!
//! Converts spoken-form German to written form:
//! - "einhundert" → "100"
//! - "zwei euro und zwanzig cent" → "€2,20"
//! - "vierundzwanzigster juli zwei tausend dreizehn" → "24. Jul. 2013"

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
