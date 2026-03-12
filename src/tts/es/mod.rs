//! Text Normalization taggers for Spanish.
//!
//! Converts written-form text to spoken Spanish:
//! - "200" → "doscientos"
//! - "5,50 €" → "cinco euros con cincuenta centimos"
//! - "5 de enero de 2025" → "cinco de enero de dos mil veinticinco"

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

/// Ones words indexed by value (0..30).
/// Spanish has unique words for 0-15, and single-word forms for 16-29.
const ONES: [&str; 30] = [
    "cero",
    "uno",
    "dos",
    "tres",
    "cuatro",
    "cinco",
    "seis",
    "siete",
    "ocho",
    "nueve",
    "diez",
    "once",
    "doce",
    "trece",
    "catorce",
    "quince",
    "dieciseis",
    "diecisiete",
    "dieciocho",
    "diecinueve",
    "veinte",
    "veintiuno",
    "veintidos",
    "veintitres",
    "veinticuatro",
    "veinticinco",
    "veintiseis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];

/// Tens words indexed by tens digit (3..9 → index 0..6).
/// 20 and below are handled by ONES. 30-90 use these base words.
const TENS: [&str; 7] = [
    "treinta",
    "cuarenta",
    "cincuenta",
    "sesenta",
    "setenta",
    "ochenta",
    "noventa",
];

/// Hundreds words indexed by value (1..9 → index 0..8).
const HUNDREDS: [&str; 9] = [
    "ciento",
    "doscientos",
    "trescientos",
    "cuatrocientos",
    "quinientos",
    "seiscientos",
    "setecientos",
    "ochocientos",
    "novecientos",
];

/// Convert an integer to Spanish words.
///
/// Examples:
/// - `0` → `"cero"`
/// - `21` → `"veintiuno"`
/// - `31` → `"treinta y uno"`
/// - `100` → `"cien"`
/// - `200` → `"doscientos"`
/// - `1000` → `"mil"`
/// - `-42` → `"menos cuarenta y dos"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "cero".to_string();
    }

    if n < 0 {
        let abs_val = (n as u64).wrapping_neg();
        return format!("menos {}", unsigned_to_words(abs_val));
    }

    unsigned_to_words(n as u64)
}

fn unsigned_to_words(n: u64) -> String {
    if n == 0 {
        return "cero".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    // Spanish uses long scale: billion = 10^12 in some contexts,
    // but we follow the standard RAE convention:
    // millon (10^6), mil millones (10^9 - "billon" is 10^12 in Spanish)
    let scales: &[(u64, &str, &str)] = &[
        (1_000_000_000_000, "billon", "billones"),
        (1_000_000, "millon", "millones"),
        (1_000, "mil", "mil"),
    ];

    for &(scale_value, singular, plural) in scales {
        if remaining >= scale_value {
            let chunk = remaining / scale_value;
            remaining %= scale_value;

            if singular == "mil" {
                // "mil" never takes "un" prefix: 1000 = "mil", not "un mil"
                if chunk == 1 {
                    parts.push("mil".to_string());
                } else {
                    parts.push(format!("{} mil", chunk_to_words(chunk as u32)));
                }
            } else {
                // millon/billon: "un millon", "dos millones"
                let chunk_words = chunk_to_words(chunk as u32);
                if chunk == 1 {
                    parts.push(format!("un {}", singular));
                } else {
                    parts.push(format!("{} {}", chunk_words, plural));
                }
            }
        }
    }

    if remaining > 0 {
        parts.push(chunk_to_words(remaining as u32));
    }

    parts.join(" ")
}

/// Convert a number 1..999 to Spanish words.
fn chunk_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n < 1000);
    let hundreds = n / 100;
    let rest = n % 100;

    let mut result = String::new();

    if hundreds > 0 {
        if hundreds == 1 && rest == 0 {
            // 100 standalone = "cien"
            return "cien".to_string();
        }
        result.push_str(HUNDREDS[(hundreds - 1) as usize]);
    }

    if rest > 0 {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&two_digit_to_words(rest));
    }

    result
}

/// Convert 1..99 to Spanish words.
fn two_digit_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n < 100);

    // 1-29 have unique/compound single-word forms
    if n < 30 {
        return ONES[n as usize].to_string();
    }

    // 30-99: tens + " y " + ones
    let tens_idx = (n / 10 - 3) as usize;
    let ones = n % 10;
    if ones == 0 {
        TENS[tens_idx].to_string()
    } else {
        format!("{} y {}", TENS[tens_idx], ONES[ones as usize])
    }
}

/// Spell each digit of a string individually in Spanish.
pub fn spell_digits(s: &str) -> String {
    s.chars()
        .filter_map(|c| c.to_digit(10).map(|d| ONES[d as usize]))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(number_to_words(0), "cero");
        assert_eq!(number_to_words(1), "uno");
        assert_eq!(number_to_words(10), "diez");
        assert_eq!(number_to_words(15), "quince");
        assert_eq!(number_to_words(16), "dieciseis");
        assert_eq!(number_to_words(19), "diecinueve");
        assert_eq!(number_to_words(20), "veinte");
        assert_eq!(number_to_words(21), "veintiuno");
        assert_eq!(number_to_words(25), "veinticinco");
        assert_eq!(number_to_words(29), "veintinueve");
    }

    #[test]
    fn test_tens_with_y() {
        assert_eq!(number_to_words(30), "treinta");
        assert_eq!(number_to_words(31), "treinta y uno");
        assert_eq!(number_to_words(42), "cuarenta y dos");
        assert_eq!(number_to_words(55), "cincuenta y cinco");
        assert_eq!(number_to_words(67), "sesenta y siete");
        assert_eq!(number_to_words(78), "setenta y ocho");
        assert_eq!(number_to_words(89), "ochenta y nueve");
        assert_eq!(number_to_words(99), "noventa y nueve");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(number_to_words(100), "cien");
        assert_eq!(number_to_words(101), "ciento uno");
        assert_eq!(number_to_words(200), "doscientos");
        assert_eq!(number_to_words(500), "quinientos");
        assert_eq!(number_to_words(999), "novecientos noventa y nueve");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(number_to_words(1000), "mil");
        assert_eq!(number_to_words(2000), "dos mil");
        assert_eq!(number_to_words(2025), "dos mil veinticinco");
        assert_eq!(number_to_words(10000), "diez mil");
    }

    #[test]
    fn test_millions() {
        assert_eq!(number_to_words(1000000), "un millon");
        assert_eq!(number_to_words(2000000), "dos millones");
        assert_eq!(number_to_words(2000003), "dos millones tres");
    }

    #[test]
    fn test_negative() {
        assert_eq!(number_to_words(-42), "menos cuarenta y dos");
    }

    #[test]
    fn test_spell_digits() {
        assert_eq!(spell_digits("14"), "uno cuatro");
        assert_eq!(spell_digits("0"), "cero");
        assert_eq!(spell_digits("987"), "nueve ocho siete");
    }
}
