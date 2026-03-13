//! Taggers for inverse text normalization.
//!
//! Each tagger handles a specific category of spoken text:
//! - cardinal: number words → digits
//! - money: currency expressions
//! - ordinal: ordinal numbers (first, second, etc.)
//! - date: date expressions
//! - time: time expressions
//! - decimal: decimal numbers
//! - measure: measurements with units
//! - telephone: phone numbers
//! - electronic: URLs and emails
//! - fraction: fractional numbers
//! - punctuation: spoken punctuation
//! - whitelist: pass-through words

// Languages
pub mod en;
pub mod fr;
pub mod hi;
