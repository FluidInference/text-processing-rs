//! Text Normalization taggers for Japanese (romaji output).
//!
//! Converts written-form text to spoken Japanese in romaji:
//! - "200" → "ni hyaku"
//! - "5000円" → "go sen en"
//! - "2025年1月5日" → "ni sen ni juu go nen ichi gatsu itsuka"

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

/// Digit words indexed by value (0..10).
const ONES: [&str; 10] = [
    "zero", "ichi", "ni", "san", "yon", "go", "roku", "nana", "hachi", "kyuu",
];

/// Convert an integer to Japanese words in romaji.
///
/// Groups by man (10,000) and oku (100,000,000) following the Japanese
/// number system. Handles special sound changes (rendaku):
/// - 300 = sanbyaku, 600 = roppyaku, 800 = happyaku
/// - 3000 = sanzen, 8000 = hassen
///
/// Examples:
/// - `0` → `"zero"`
/// - `21` → `"ni juu ichi"`
/// - `123` → `"hyaku ni juu san"`
/// - `10000` → `"ichi man"`
/// - `-42` → `"mainasu yon juu ni"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    if n < 0 {
        let abs_val = (n as u64).wrapping_neg();
        return format!("mainasu {}", unsigned_to_words(abs_val));
    }

    unsigned_to_words(n as u64)
}

fn unsigned_to_words(n: u64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    // Japanese groups by man (10,000) and oku (100,000,000)

    // Process oku groups
    if remaining >= 100_000_000 {
        let oku_count = remaining / 100_000_000;
        remaining %= 100_000_000;
        // oku_count itself could be large, convert it recursively using sub-man grouping
        let oku_words = sub_oku_to_words(oku_count);
        parts.push(format!("{} oku", oku_words));
    }

    // Process man groups
    if remaining >= 10_000 {
        let man_count = remaining / 10_000;
        remaining %= 10_000;
        let man_words = chunk_to_words(man_count as u32);
        parts.push(format!("{} man", man_words));
    }

    // Remainder (0..9999)
    if remaining > 0 {
        parts.push(chunk_to_words(remaining as u32));
    }

    parts.join(" ")
}

/// Convert a number that will precede "oku" — it could be up to 9999 man range
/// but for oku prefix we just need 1..9999 range of sub-man grouping.
fn sub_oku_to_words(n: u64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    // The oku prefix could itself be in the man range
    if remaining >= 10_000 {
        let man_count = remaining / 10_000;
        remaining %= 10_000;
        let man_words = chunk_to_words(man_count as u32);
        parts.push(format!("{} man", man_words));
    }

    if remaining > 0 {
        parts.push(chunk_to_words(remaining as u32));
    }

    parts.join(" ")
}

/// Convert a number 1..9999 to Japanese words in romaji.
/// Handles sen (1000), hyaku (100), juu (10), and ones.
fn chunk_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n <= 9999);
    let mut parts: Vec<String> = Vec::new();

    let thousands = n / 1000;
    let rest_after_thou = n % 1000;
    let hundreds = rest_after_thou / 100;
    let rest_after_hund = rest_after_thou % 100;
    let tens = rest_after_hund / 10;
    let ones = rest_after_hund % 10;

    // Thousands (sen) with special sound changes
    if thousands > 0 {
        parts.push(sen_word(thousands));
    }

    // Hundreds (hyaku) with special sound changes
    if hundreds > 0 {
        parts.push(hyaku_word(hundreds));
    }

    // Tens (juu)
    if tens > 0 {
        if tens == 1 {
            parts.push("juu".to_string());
        } else {
            parts.push(format!("{} juu", ONES[tens as usize]));
        }
    }

    // Ones
    if ones > 0 {
        parts.push(ONES[ones as usize].to_string());
    }

    parts.join(" ")
}

/// Convert thousands digit to the appropriate sen form.
/// Special: 3000=sanzen, 8000=hassen, 1000=sen
fn sen_word(thousands: u32) -> String {
    match thousands {
        1 => "sen".to_string(),
        3 => "sanzen".to_string(),
        8 => "hassen".to_string(),
        _ => format!("{} sen", ONES[thousands as usize]),
    }
}

/// Convert hundreds digit to the appropriate hyaku form.
/// Special: 300=sanbyaku, 600=roppyaku, 800=happyaku, 100=hyaku
fn hyaku_word(hundreds: u32) -> String {
    match hundreds {
        1 => "hyaku".to_string(),
        3 => "sanbyaku".to_string(),
        6 => "roppyaku".to_string(),
        8 => "happyaku".to_string(),
        _ => format!("{} hyaku", ONES[hundreds as usize]),
    }
}

/// Spell each digit of a string individually in Japanese romaji.
///
/// "14" → "ichi yon"
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
        assert_eq!(number_to_words(0), "zero");
        assert_eq!(number_to_words(1), "ichi");
        assert_eq!(number_to_words(5), "go");
        assert_eq!(number_to_words(10), "juu");
        assert_eq!(number_to_words(11), "juu ichi");
        assert_eq!(number_to_words(15), "juu go");
        assert_eq!(number_to_words(20), "ni juu");
        assert_eq!(number_to_words(21), "ni juu ichi");
        assert_eq!(number_to_words(99), "kyuu juu kyuu");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(number_to_words(100), "hyaku");
        assert_eq!(number_to_words(200), "ni hyaku");
        assert_eq!(number_to_words(300), "sanbyaku");
        assert_eq!(number_to_words(600), "roppyaku");
        assert_eq!(number_to_words(800), "happyaku");
        assert_eq!(number_to_words(123), "hyaku ni juu san");
        assert_eq!(number_to_words(999), "kyuu hyaku kyuu juu kyuu");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(number_to_words(1000), "sen");
        assert_eq!(number_to_words(2000), "ni sen");
        assert_eq!(number_to_words(3000), "sanzen");
        assert_eq!(number_to_words(8000), "hassen");
        assert_eq!(number_to_words(1500), "sen go hyaku");
        assert_eq!(number_to_words(9999), "kyuu sen kyuu hyaku kyuu juu kyuu");
    }

    #[test]
    fn test_man() {
        assert_eq!(number_to_words(10000), "ichi man");
        assert_eq!(number_to_words(20000), "ni man");
        assert_eq!(number_to_words(50000), "go man");
        assert_eq!(
            number_to_words(12345),
            "ichi man ni sen sanbyaku yon juu go"
        );
    }

    #[test]
    fn test_oku() {
        assert_eq!(number_to_words(100_000_000), "ichi oku");
        assert_eq!(number_to_words(200_000_000), "ni oku");
        assert_eq!(
            number_to_words(123_456_789),
            "ichi oku ni sen sanbyaku yon juu go man roku sen nana hyaku hachi juu kyuu"
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(number_to_words(-42), "mainasu yon juu ni");
        assert_eq!(number_to_words(-1000), "mainasu sen");
    }

    #[test]
    fn test_spell_digits() {
        assert_eq!(spell_digits("14"), "ichi yon");
        assert_eq!(spell_digits("0"), "zero");
        assert_eq!(spell_digits("987"), "kyuu hachi nana");
    }
}
