//! Text Normalization taggers.
//!
//! Converts written-form text to spoken form (the reverse of ITN):
//! - "200" → "two hundred" (English)
//! - "$5.50" → "five dollars and fifty cents" (English)
//! - "January 5, 2025" → "january fifth twenty twenty five" (English)
//!
//! Supports multiple languages via submodules.

// Languages
pub mod de;
pub mod en;
pub mod es;
pub mod fr;
pub mod hi;
pub mod ja;
pub mod zh;
