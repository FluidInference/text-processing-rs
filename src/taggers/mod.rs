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

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod electronic;
pub mod measure;
pub mod money;
pub mod ordinal;
pub mod telephone;
pub mod time;
pub mod whitelist;
pub mod word;

// TODO: Add remaining taggers
// pub mod fraction;
// pub mod punctuation;
