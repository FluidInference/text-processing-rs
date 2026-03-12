//! Text Normalization taggers for Hindi (romanized).
//!
//! Converts written-form text to spoken Hindi in romanized transliteration:
//! - "200" → "do sau"
//! - "5.50 ₹" → "paanch rupaye aur pachaas paise"
//! - "5 January 2025" → "paanch janvari do hazaar pachees"

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

/// Hindi unique words for 0-99 in romanized form.
/// Hindi has distinct words for every number from 0 to 99.
const HINDI_0_TO_99: [&str; 100] = [
    "shunya",      // 0
    "ek",          // 1
    "do",          // 2
    "teen",        // 3
    "chaar",       // 4
    "paanch",      // 5
    "chhah",       // 6
    "saat",        // 7
    "aath",        // 8
    "nau",         // 9
    "das",         // 10
    "gyaarah",     // 11
    "baarah",      // 12
    "terah",       // 13
    "chaudah",     // 14
    "pandrah",     // 15
    "solah",       // 16
    "satrah",      // 17
    "atthaarah",   // 18
    "unees",       // 19
    "bees",        // 20
    "ikkees",      // 21
    "baees",       // 22
    "teis",        // 23
    "chaubees",    // 24
    "pachchees",   // 25
    "chhabees",    // 26
    "sattaees",    // 27
    "atthaees",    // 28
    "untees",      // 29
    "tees",        // 30
    "ikattees",    // 31
    "battees",     // 32
    "taintees",    // 33
    "chautees",    // 34
    "paintees",    // 35
    "chhattees",   // 36
    "saintees",    // 37
    "adtees",      // 38
    "untaalees",   // 39
    "chaalis",     // 40
    "iktaalees",   // 41
    "bayaalees",   // 42
    "taintaalees", // 43
    "chauvaalees", // 44
    "paintaalees", // 45
    "chhiyaalees", // 46
    "saintaalees", // 47
    "adtaalees",   // 48
    "unchaas",     // 49
    "pachaas",     // 50
    "ikyaavan",    // 51
    "baavan",      // 52
    "tirpan",      // 53
    "chauvan",     // 54
    "pachpan",     // 55
    "chhappan",    // 56
    "sattaavan",   // 57
    "atthaavan",   // 58
    "unsath",      // 59
    "saath",       // 60
    "iksath",      // 61
    "baasath",     // 62
    "tirsath",     // 63
    "chausath",    // 64
    "painsath",    // 65
    "chhiyaasath", // 66
    "sarsath",     // 67
    "adsath",      // 68
    "unhattar",    // 69
    "sattar",      // 70
    "ikhattar",    // 71
    "bahattar",    // 72
    "tihattar",    // 73
    "chauhattar",  // 74
    "pachahattar", // 75
    "chhihattar",  // 76
    "satahattar",  // 77
    "athahattar",  // 78
    "unyaasi",     // 79
    "assi",        // 80
    "ikyaasi",     // 81
    "bayaasi",     // 82
    "tiraasi",     // 83
    "chauraasi",   // 84
    "pachaasi",    // 85
    "chhiyaasi",   // 86
    "sataasi",     // 87
    "athaasi",     // 88
    "navaasi",     // 89
    "nabbe",       // 90
    "ikyaanbe",    // 91
    "baanbe",      // 92
    "tiraanbe",    // 93
    "chauraanbe",  // 94
    "pachaanbe",   // 95
    "chhiyaanbe",  // 96
    "sataanbe",    // 97
    "athaanbe",    // 98
    "ninyaanbe",   // 99
];

/// Digit words indexed 0-9 for spell_digits.
const DIGIT_WORDS: [&str; 10] = [
    "shunya", "ek", "do", "teen", "chaar", "paanch", "chhah", "saat", "aath", "nau",
];

/// Convert an integer to romanized Hindi words.
///
/// Uses the Indian numbering system: lakh (1,00,000) and crore (1,00,00,000).
///
/// Examples:
/// - `0` -> `"shunya"`
/// - `21` -> `"ikkees"`
/// - `100` -> `"ek sau"`
/// - `1000` -> `"ek hazaar"`
/// - `100000` -> `"ek lakh"`
/// - `10000000` -> `"ek crore"`
/// - `-42` -> `"rhin bayaalees"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "shunya".to_string();
    }

    if n < 0 {
        let abs_val = (n as u64).wrapping_neg();
        return format!("rhin {}", unsigned_to_words(abs_val));
    }

    unsigned_to_words(n as u64)
}

fn unsigned_to_words(n: u64) -> String {
    if n == 0 {
        return "shunya".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    // Indian numbering: crore (10^7), lakh (10^5), hazaar (10^3), sau (10^2)
    // Above crore we use "arab" (10^9), "kharab" (10^11) etc. but for simplicity
    // we handle up to crores by repeating crore groups.
    let scales: &[(u64, &str)] = &[
        (1_00_00_00_00_00_000, "kharab"), // 10^12 (lakh crore)
        (1_00_00_00_00_000, "arab"),      // 10^9 (hundred crore)
        (1_00_00_000, "crore"),           // 10^7
        (1_00_000, "lakh"),               // 10^5
        (1_000, "hazaar"),                // 10^3
    ];

    for &(scale_value, scale_name) in scales {
        if remaining >= scale_value {
            let chunk = remaining / scale_value;
            remaining %= scale_value;
            let chunk_words = small_number_to_words(chunk);
            parts.push(format!("{} {}", chunk_words, scale_name));
        }
    }

    // Handle hundreds (sau)
    if remaining >= 100 {
        let hundreds = remaining / 100;
        remaining %= 100;
        parts.push(format!("{} sau", HINDI_0_TO_99[hundreds as usize]));
    }

    // Handle 1-99
    if remaining > 0 {
        parts.push(HINDI_0_TO_99[remaining as usize].to_string());
    }

    parts.join(" ")
}

/// Convert a number that can appear as a chunk before a scale word.
/// This handles numbers up to 99 (for lakh/crore grouping which uses 2-digit groups),
/// but also needs to handle up to 999 for the hazaar group (3 digits from right).
fn small_number_to_words(n: u64) -> String {
    if n == 0 {
        return "shunya".to_string();
    }
    if n < 100 {
        return HINDI_0_TO_99[n as usize].to_string();
    }

    // For numbers >= 100, recursively handle
    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    if remaining >= 100 {
        let hundreds = remaining / 100;
        remaining %= 100;
        if hundreds < 100 {
            parts.push(format!("{} sau", HINDI_0_TO_99[hundreds as usize]));
        } else {
            parts.push(format!("{} sau", small_number_to_words(hundreds)));
        }
    }

    if remaining > 0 {
        parts.push(HINDI_0_TO_99[remaining as usize].to_string());
    }

    parts.join(" ")
}

/// Spell each digit of a string individually in romanized Hindi.
///
/// "14" -> "ek chaar"
pub fn spell_digits(s: &str) -> String {
    s.chars()
        .filter_map(|c| c.to_digit(10).map(|d| DIGIT_WORDS[d as usize]))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(number_to_words(0), "shunya");
        assert_eq!(number_to_words(1), "ek");
        assert_eq!(number_to_words(10), "das");
        assert_eq!(number_to_words(16), "solah");
        assert_eq!(number_to_words(19), "unees");
        assert_eq!(number_to_words(20), "bees");
        assert_eq!(number_to_words(21), "ikkees");
        assert_eq!(number_to_words(50), "pachaas");
        assert_eq!(number_to_words(99), "ninyaanbe");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(number_to_words(100), "ek sau");
        assert_eq!(number_to_words(200), "do sau");
        assert_eq!(number_to_words(123), "ek sau teis");
        assert_eq!(number_to_words(999), "nau sau ninyaanbe");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(number_to_words(1000), "ek hazaar");
        assert_eq!(number_to_words(2000), "do hazaar");
        assert_eq!(number_to_words(2025), "do hazaar pachchees");
        assert_eq!(number_to_words(10000), "das hazaar");
    }

    #[test]
    fn test_lakhs_and_crores() {
        assert_eq!(number_to_words(100000), "ek lakh");
        assert_eq!(number_to_words(200000), "do lakh");
        assert_eq!(
            number_to_words(1234567),
            "baarah lakh chautees hazaar paanch sau sarsath"
        );
        assert_eq!(number_to_words(10000000), "ek crore");
        assert_eq!(
            number_to_words(12345678),
            "ek crore teis lakh paintaalees hazaar chhah sau athahattar"
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(number_to_words(-42), "rhin bayaalees");
    }

    #[test]
    fn test_spell_digits() {
        assert_eq!(spell_digits("14"), "ek chaar");
        assert_eq!(spell_digits("0"), "shunya");
        assert_eq!(spell_digits("987"), "nau aath saat");
    }
}
