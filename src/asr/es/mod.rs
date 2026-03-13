//! Inverse Text Normalization taggers for Spanish.
//!
//! Converts spoken-form Spanish to written form:
//! - "doscientos cincuenta y uno" → "251"
//! - "doce dólares y cinco centavos" → "$12,05"
//! - "primero de enero" → "1 de enero"

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
