//! Korean inverse text normalization.
//!
//! Converts Sino-Korean numerals and spoken-form Korean to written
//! form. Uses a sentence-scanning approach: each processor scans the
//! input for its patterns and replaces number spans in-place.
//!
//! Korean Sino-numeral syllables double as common non-numeric
//! morphemes, so the cardinal catch-all is deliberately conservative
//! (see [`cardinal`]); the date / time / ordinal / decimal / fraction
//! taggers rely on their suffixes for disambiguation.

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod fraction;
pub mod ordinal;
pub mod time;
