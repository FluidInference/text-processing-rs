//! Text Normalization taggers for French.
//!
//! Converts written-form text to spoken French:
//! - "200" → "deux cents"
//! - "5,50 €" → "cinq euros et cinquante centimes"
//! - "5 janvier 2025" → "cinq janvier deux mille vingt-cinq"

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

/// Ones words indexed by value (0..20).
const ONES: [&str; 20] = [
    "zero", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze",
    "douze", "treize", "quatorze", "quinze", "seize", "dix-sept", "dix-huit", "dix-neuf",
];

/// Tens words indexed by tens digit (2..7 → index 0..5).
/// French uses special forms for 70, 80, 90.
const TENS: [&str; 5] = ["vingt", "trente", "quarante", "cinquante", "soixante"];

/// Convert an integer to French words.
///
/// Examples:
/// - `0` → `"zero"`
/// - `21` → `"vingt et un"`
/// - `71` → `"soixante et onze"`
/// - `80` → `"quatre-vingts"`
/// - `91` → `"quatre-vingt-onze"`
/// - `200` → `"deux cents"`
/// - `-42` → `"moins quarante-deux"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    if n < 0 {
        let abs_val = (n as u64).wrapping_neg();
        return format!("moins {}", unsigned_to_words(abs_val));
    }

    unsigned_to_words(n as u64)
}

fn unsigned_to_words(n: u64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    let scales: &[(u64, &str)] = &[
        (1_000_000_000_000_000_000, "trillion"),
        (1_000_000_000_000_000, "billiard"),
        (1_000_000_000_000, "billion"),
        (1_000_000_000, "milliard"),
        (1_000_000, "million"),
        (1_000, "mille"),
    ];

    for &(scale_value, scale_name) in scales {
        if remaining >= scale_value {
            let chunk = remaining / scale_value;
            remaining %= scale_value;

            if scale_name == "mille" {
                if chunk == 1 {
                    parts.push("mille".to_string());
                } else {
                    parts.push(format!("{} mille", chunk_to_words(chunk as u32, false)));
                }
            } else {
                let chunk_words = chunk_to_words(chunk as u32, false);
                if chunk == 1 {
                    parts.push(format!("un {}", scale_name));
                } else {
                    parts.push(format!("{} {}s", chunk_words, scale_name));
                }
            }
        }
    }

    if remaining > 0 {
        parts.push(chunk_to_words(
            remaining as u32,
            remaining < 1000 && parts.is_empty(),
        ));
    }

    parts.join(" ")
}

/// Convert a number 1..999 to French words.
/// `standalone_cents`: if true and value is exact hundreds, add 's' to "cent" (deux cents).
fn chunk_to_words(n: u32, standalone_cents: bool) -> String {
    debug_assert!(n > 0 && n < 1000);
    let hundreds = n / 100;
    let rest = n % 100;

    let mut result = String::new();

    if hundreds > 0 {
        if hundreds == 1 {
            result.push_str("cent");
        } else {
            result.push_str(ONES[hundreds as usize]);
            if rest == 0 && standalone_cents {
                result.push_str(" cents");
            } else {
                result.push_str(" cent");
            }
        }
    }

    if rest > 0 {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&two_digit_to_words(rest));
    }

    result
}

/// Convert 1..99 to French words.
fn two_digit_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n < 100);

    if n < 20 {
        return ONES[n as usize].to_string();
    }

    if n < 70 {
        let tens_idx = (n / 10 - 2) as usize;
        let ones = n % 10;
        if ones == 0 {
            TENS[tens_idx].to_string()
        } else if ones == 1 {
            format!("{} et un", TENS[tens_idx])
        } else {
            format!("{}-{}", TENS[tens_idx], ONES[ones as usize])
        }
    } else if n < 80 {
        // 70-79: soixante-dix, soixante et onze, soixante-douze...
        let ones = n - 60;
        if ones == 10 {
            "soixante-dix".to_string()
        } else if ones == 11 {
            "soixante et onze".to_string()
        } else {
            format!("soixante-{}", ONES[ones as usize])
        }
    } else if n == 80 {
        "quatre-vingts".to_string()
    } else {
        // 81-99: quatre-vingt-un, quatre-vingt-deux... quatre-vingt-dix, quatre-vingt-onze...
        let ones = n - 80;
        format!("quatre-vingt-{}", ONES[ones as usize])
    }
}

/// Spell each digit of a string individually in French.
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
        assert_eq!(number_to_words(1), "un");
        assert_eq!(number_to_words(10), "dix");
        assert_eq!(number_to_words(16), "seize");
        assert_eq!(number_to_words(17), "dix-sept");
        assert_eq!(number_to_words(20), "vingt");
        assert_eq!(number_to_words(21), "vingt et un");
        assert_eq!(number_to_words(22), "vingt-deux");
    }

    #[test]
    fn test_french_special() {
        assert_eq!(number_to_words(70), "soixante-dix");
        assert_eq!(number_to_words(71), "soixante et onze");
        assert_eq!(number_to_words(72), "soixante-douze");
        assert_eq!(number_to_words(79), "soixante-dix-neuf");
        assert_eq!(number_to_words(80), "quatre-vingts");
        assert_eq!(number_to_words(81), "quatre-vingt-un");
        assert_eq!(number_to_words(90), "quatre-vingt-dix");
        assert_eq!(number_to_words(91), "quatre-vingt-onze");
        assert_eq!(number_to_words(99), "quatre-vingt-dix-neuf");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(number_to_words(100), "cent");
        assert_eq!(number_to_words(200), "deux cents");
        assert_eq!(number_to_words(201), "deux cent un");
        assert_eq!(number_to_words(999), "neuf cent quatre-vingt-dix-neuf");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(number_to_words(1000), "mille");
        assert_eq!(number_to_words(2000), "deux mille");
        assert_eq!(number_to_words(2025), "deux mille vingt-cinq");
    }

    #[test]
    fn test_negative() {
        assert_eq!(number_to_words(-42), "moins quarante-deux");
    }
}
