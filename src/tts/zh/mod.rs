//! Text Normalization taggers for Mandarin Chinese.
//!
//! Converts written-form text to spoken Mandarin in pinyin:
//! - "200" -> "er bai"
//! - "3.14" -> "san dian yi si"
//! - "2025年1月5日" -> "er ling er wu nian yi yue wu ri"

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
const DIGITS: [&str; 10] = [
    "ling", "yi", "er", "san", "si", "wu", "liu", "qi", "ba", "jiu",
];

/// Convert an integer to Mandarin Chinese words (pinyin).
///
/// Uses the Chinese grouping system based on wan (10,000) and yi (100,000,000)
/// instead of the Western thousand-based system.
///
/// Examples:
/// - `0` -> `"ling"`
/// - `21` -> `"er shi yi"`
/// - `123` -> `"yi bai er shi san"`
/// - `10000` -> `"yi wan"`
/// - `100000000` -> `"yi yi"`
/// - `-42` -> `"fu si shi er"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "ling".to_string();
    }

    if n < 0 {
        let abs_val = (n as u64).wrapping_neg();
        return format!("fu {}", unsigned_to_words(abs_val));
    }

    unsigned_to_words(n as u64)
}

fn unsigned_to_words(n: u64) -> String {
    if n == 0 {
        return "ling".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    // Chinese scale units: yi (亿, 10^8) and wan (万, 10^4)
    let scales: &[(u64, &str)] = &[
        (1_000_000_000_000_000_0, "jing"), // 京, 10^16
        (1_000_000_000_000, "zhao"),       // 兆, 10^12
        (100_000_000, "yi"),               // 亿, 10^8
        (10_000, "wan"),                   // 万, 10^4
    ];

    for &(scale_value, scale_name) in scales {
        if remaining >= scale_value {
            let chunk = remaining / scale_value;
            remaining %= scale_value;

            // The chunk within a scale is always < 10000 (a wan-group)
            let chunk_words = wan_group_to_words(chunk as u32);
            parts.push(format!("{} {}", chunk_words, scale_name));
        }
    }

    // Remainder (0..9999)
    if remaining > 0 {
        // Insert "ling" if there's a gap: e.g. 10003 = "yi wan ling san"
        if !parts.is_empty() && remaining < 1000 {
            parts.push("ling".to_string());
        }
        parts.push(wan_group_to_words(remaining as u32));
    }

    parts.join(" ")
}

/// Convert a number 1..9999 to Mandarin pinyin words.
/// This handles a single wan-group (4-digit group).
fn wan_group_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n <= 9999);
    let mut parts: Vec<String> = Vec::new();

    let qian = n / 1000;
    let bai = (n % 1000) / 100;
    let shi = (n % 100) / 10;
    let ge = n % 10;

    if qian > 0 {
        parts.push(format!("{} qian", DIGITS[qian as usize]));
    }

    if bai > 0 {
        parts.push(format!("{} bai", DIGITS[bai as usize]));
    } else if qian > 0 && (shi > 0 || ge > 0) {
        // Zero placeholder: yi qian ling san shi (1030)
        parts.push("ling".to_string());
    }

    if shi > 0 {
        if shi == 1 && qian == 0 && bai == 0 {
            // For numbers 10-19 at the top level, just say "shi" not "yi shi"
            parts.push("shi".to_string());
        } else {
            parts.push(format!("{} shi", DIGITS[shi as usize]));
        }
    } else if bai > 0 && ge > 0 {
        // Zero placeholder: yi bai ling san (103)
        parts.push("ling".to_string());
    }

    if ge > 0 {
        parts.push(DIGITS[ge as usize].to_string());
    }

    parts.join(" ")
}

/// Spell each digit of a string individually in Mandarin pinyin.
///
/// "14" -> "yi si"
pub fn spell_digits(s: &str) -> String {
    s.chars()
        .filter_map(|c| c.to_digit(10).map(|d| DIGITS[d as usize]))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(number_to_words(0), "ling");
        assert_eq!(number_to_words(1), "yi");
        assert_eq!(number_to_words(10), "shi");
        assert_eq!(number_to_words(11), "shi yi");
        assert_eq!(number_to_words(20), "er shi");
        assert_eq!(number_to_words(21), "er shi yi");
        assert_eq!(number_to_words(99), "jiu shi jiu");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(number_to_words(100), "yi bai");
        assert_eq!(number_to_words(103), "yi bai ling san");
        assert_eq!(number_to_words(110), "yi bai yi shi");
        assert_eq!(number_to_words(200), "er bai");
        assert_eq!(number_to_words(999), "jiu bai jiu shi jiu");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(number_to_words(1000), "yi qian");
        assert_eq!(number_to_words(1030), "yi qian ling san shi");
        assert_eq!(number_to_words(1003), "yi qian ling san");
        assert_eq!(number_to_words(2025), "er qian ling er shi wu");
        assert_eq!(number_to_words(9999), "jiu qian jiu bai jiu shi jiu");
    }

    #[test]
    fn test_wan() {
        assert_eq!(number_to_words(10000), "yi wan");
        assert_eq!(number_to_words(10003), "yi wan ling san");
        assert_eq!(number_to_words(50000), "wu wan");
        assert_eq!(number_to_words(12345), "yi wan er qian san bai si shi wu");
    }

    #[test]
    fn test_yi_unit() {
        assert_eq!(number_to_words(100_000_000), "yi yi");
        assert_eq!(number_to_words(200_000_000), "er yi");
    }

    #[test]
    fn test_negative() {
        assert_eq!(number_to_words(-42), "fu si shi er");
        assert_eq!(number_to_words(-1000), "fu yi qian");
    }

    #[test]
    fn test_spell_digits() {
        assert_eq!(spell_digits("14"), "yi si");
        assert_eq!(spell_digits("0"), "ling");
        assert_eq!(spell_digits("2025"), "er ling er wu");
    }
}
