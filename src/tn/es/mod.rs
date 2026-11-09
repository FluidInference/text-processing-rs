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

    // Spanish uses the long scale (RAE): millon (10^6), billon (10^12) and
    // trillon (10^18), each counted up to 999 999 ("mil millones" = 10^9).
    let scales: &[(u64, &str, &str)] = &[
        (1_000_000_000_000_000_000, "trillon", "trillones"),
        (1_000_000_000_000, "billon", "billones"),
        (1_000_000, "millon", "millones"),
    ];

    for &(scale_value, singular, plural) in scales {
        if remaining >= scale_value {
            let count = remaining / scale_value;
            remaining %= scale_value;

            // "un millon", "dos millones", "mil millones"
            if count == 1 {
                parts.push(format!("un {}", singular));
            } else {
                parts.push(format!("{} {}", below_million(count as u32), plural));
            }
        }
    }

    if remaining > 0 {
        parts.push(below_million(remaining as u32));
    }

    parts.join(" ")
}

/// Convert 1..999_999 to Spanish words.
fn below_million(n: u32) -> String {
    let thousands = n / 1000;
    let rest = n % 1000;

    let mut parts: Vec<String> = Vec::new();
    // "mil" never takes "un" prefix: 1000 = "mil", not "un mil"
    if thousands == 1 {
        parts.push("mil".to_string());
    } else if thousands > 1 {
        parts.push(format!("{} mil", chunk_to_words(thousands)));
    }
    if rest > 0 {
        parts.push(chunk_to_words(rest));
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
    fn test_mil_millones() {
        // 10^9 and up used to index past the hundreds table.
        assert_eq!(number_to_words(1_000_000_001), "mil millones uno");
        assert_eq!(
            number_to_words(2_000_000_052),
            "dos mil millones cincuenta y dos"
        );
        assert_eq!(number_to_words(1_000_000_000_000_001), "mil billones uno");
        assert!(number_to_words(i64::MAX).starts_with("nueve trillones"));
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
