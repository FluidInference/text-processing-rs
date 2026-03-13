//! Japanese inverse text normalization.
//!
//! Converts kanji numerals and spoken-form Japanese to written form.
//! Uses a sentence-scanning approach: each processor scans the input
//! for its patterns and replaces kanji number spans in-place.

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod fraction;
pub mod ordinal;
pub mod time;
