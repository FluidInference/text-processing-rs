//! Chinese inverse text normalization.
//!
//! Converts Chinese numerals and spoken-form expressions to written form.
//! Uses a sentence-scanning approach: each processor scans the input
//! for its patterns and replaces Chinese number spans in-place.

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod fraction;
pub mod money;
pub mod ordinal;
pub mod time;
pub mod whitelist;
pub mod word;
