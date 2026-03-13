//! Money tagger for French.
//!
//! Converts spoken French currency expressions to written form:
//! - "cinq euros" → "5 €"
//! - "cinq euros et cinquante centimes" → "5,50 €"
//! - "cinquante centimes" → "0,50 €"
//! - "un euro" → "1 €"
//! - "deux dollars vingt" → "2,20 $"
//! - "quatre-vingt mille won" → "80 000 ₩"
//! - "deux-millions de dollars" → "2 millions de dollars"

use super::cardinal::words_to_number;

/// Currency definition
struct Currency {
    /// Main unit words (plural, singular)
    main_words: &'static [&'static str],
    /// Symbol
    symbol: &'static str,
    /// Cent/subunit words
    cent_words: &'static [&'static str],
    /// Whether cents are represented as fraction of main unit
    cent_is_fraction: bool,
}

const CURRENCIES: &[Currency] = &[
    Currency {
        main_words: &["euros", "euro"],
        symbol: "€",
        cent_words: &["centimes", "centime"],
        cent_is_fraction: true,
    },
    Currency {
        main_words: &["dollars", "dollar"],
        symbol: "$",
        cent_words: &[], // "cent(s)" conflicts with French number word for 100
        cent_is_fraction: false,
    },
    Currency {
        main_words: &["livres", "livre"],
        symbol: "£",
        cent_words: &["pence"],
        cent_is_fraction: true,
    },
    Currency {
        main_words: &["francs suisses", "franc suisse"],
        symbol: "CHF",
        cent_words: &["centimes", "centime"],
        cent_is_fraction: true,
    },
    Currency {
        main_words: &["wons", "won"],
        symbol: "₩",
        cent_words: &[],
        cent_is_fraction: false,
    },
    Currency {
        main_words: &["yens", "yen"],
        symbol: "¥",
        cent_words: &[],
        cent_is_fraction: false,
    },
];

/// Parse spoken French money expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Check for scale expressions first: "X-millions de dollars" → "X millions de dollars"
    if let Some(result) = parse_scale_currency(&input_lower) {
        return Some(result);
    }

    // Try each currency
    for currency in CURRENCIES {
        if let Some(result) = try_currency(&input_lower, currency) {
            return Some(result);
        }
    }

    None
}

/// Parse scale expressions: "deux-millions de dollars" → "2 millions de dollars"
/// "quatre virgule quatre-vingt milliards d'euros" → "4,80 milliards d'euros"
fn parse_scale_currency(input: &str) -> Option<String> {
    let scale_words = [
        "trillions", "trillion",
        "billiards", "billiard",
        "billions", "billion",
        "milliards", "milliard",
        "millions", "million",
    ];

    // Normalize hyphens around scale words to spaces for matching
    let mut normalized = input.to_string();
    for &scale in &scale_words {
        let hyphen_pattern = format!("-{}", scale);
        let space_pattern = format!(" {}", scale);
        normalized = normalized.replace(&hyphen_pattern, &space_pattern);
    }

    for &scale in &scale_words {
        // Pattern: "X scale de CURRENCY" or "X scale d'CURRENCY"
        let de_pattern = format!(" {} de ", scale);
        let d_pattern = format!(" {} d'", scale);
        let d_pattern_curly = format!(" {} d\u{2019}", scale); // right single quote

        for pattern in &[&de_pattern, &d_pattern, &d_pattern_curly] {
            if let Some(scale_pos) = normalized.find(pattern.as_str()) {
                let num_part = &normalized[..scale_pos];
                // Parse the number
                let num_str = parse_money_number(num_part)?;
                // Return with scale and currency name preserved
                let suffix = &normalized[scale_pos + 1..]; // "millions de dollars"
                return Some(format!("{} {}", num_str, suffix));
            }
        }
    }

    None
}

/// Try to parse with a specific currency
fn try_currency(input: &str, currency: &Currency) -> Option<String> {
    // Try "X MAIN et Y CENT" pattern
    for &main_word in currency.main_words {
        let et_pattern = format!(" {} et ", main_word);
        if let Some(main_pos) = input.find(&et_pattern) {
            let num_part = &input[..main_pos];
            let cent_part = &input[main_pos + et_pattern.len()..];

            // Check if cent_part ends with a cent word
            for &cent_word in currency.cent_words {
                if cent_part.ends_with(cent_word) {
                    let cent_num_part = cent_part.strip_suffix(cent_word)?.trim();
                    let main_num = parse_money_number(num_part)?;
                    let cent_num = parse_money_number(cent_num_part)?;
                    return Some(format!("{},{:0>2} {}", main_num, cent_num, currency.symbol));
                }
            }

            // "cinq euro et soixante" → "5,60 €" (cent amount without cent word)
            if let Some(cent_num) = parse_money_number(cent_part) {
                return Some(format!("{},{:0>2} {}", parse_money_number(num_part)?, cent_num, currency.symbol));
            }
        }

        // Try "X MAIN Y" pattern (no "et", cents implied by second number)
        // "vingt euro cinq" → "20,05 €", "deux dollars vingt" → "2,20 $"
        let main_pattern = format!(" {} ", main_word);
        if let Some(main_pos) = input.find(&main_pattern) {
            let num_part = &input[..main_pos];
            let after_main = &input[main_pos + main_pattern.len()..];

            // The part after the main word should be a cent value
            if !after_main.is_empty() {
                if let Some(main_num) = parse_money_number(num_part) {
                    if let Some(cent_num) = parse_money_number(after_main) {
                        return Some(format!("{},{:0>2} {}", main_num, cent_num, currency.symbol));
                    }
                }
            }
        }

        // Try "X MAIN" pattern (main unit only, at end of string)
        let end_pattern = format!(" {}", main_word);
        if input.ends_with(&end_pattern) {
            let num_part = input.strip_suffix(&end_pattern)?.trim();
            let main_num = parse_money_number(num_part)?;
            return Some(format!("{} {}", main_num, currency.symbol));
        }
    }

    // Try cent-only pattern: "X CENT_WORD" → "0,XX SYMBOL"
    // Only match if cent value is ≤99 (avoids "mille cent" = 1100 being parsed as $10.00)
    if currency.cent_is_fraction {
        for &cent_word in currency.cent_words {
            let end_pattern = format!(" {}", cent_word);
            if input.ends_with(&end_pattern) {
                let num_part = input.strip_suffix(&end_pattern)?.trim();
                // Validate the number before "cent(s)" is a small cents amount
                if let Some(num) = words_to_number(&num_part.to_lowercase()) {
                    let n = num as i64;
                    if n >= 0 && n <= 99 {
                        return Some(format!("0,{:0>2} {}", n, currency.symbol));
                    }
                }
                // If > 99 or not parseable, skip (probably "mille cent" = 1100)
            }
        }
    }

    None
}

/// Parse number from money context (handles "zéro" and compound numbers)
fn parse_money_number(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_lowercase();

    // Handle "zéro"
    if lower == "zéro" || lower == "zero" {
        return Some("0".to_string());
    }

    // Handle decimal: "quatre virgule quatre-vingt" → "4,80"
    if lower.contains(" virgule ") || lower.contains("virgule ") {
        return super::decimal::parse(&lower);
    }

    let num = words_to_number(&lower)?;
    let n = num as i64;

    // Format with French space separators for large numbers
    Some(format_with_spaces(n))
}

/// Format number with French space separators
fn format_with_spaces(n: i64) -> String {
    let abs_n = n.unsigned_abs();
    let s = abs_n.to_string();

    if s.len() <= 3 {
        return if n < 0 {
            format!("-{}", s)
        } else {
            s
        };
    }

    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }

    if n < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euros() {
        assert_eq!(parse("cinq euros"), Some("5 €".to_string()));
        assert_eq!(parse("un euro"), Some("1 €".to_string()));
        assert_eq!(parse("vingt euros"), Some("20 €".to_string()));
        assert_eq!(parse("zéro euro"), Some("0 €".to_string()));
    }

    #[test]
    fn test_euros_and_centimes() {
        assert_eq!(
            parse("deux euros et vingt centimes"),
            Some("2,20 €".to_string())
        );
        assert_eq!(
            parse("cinq euro et soixante"),
            Some("5,60 €".to_string())
        );
        assert_eq!(
            parse("vingt euro cinq"),
            Some("20,05 €".to_string())
        );
        assert_eq!(
            parse("zéro euro quatre-vingt"),
            Some("0,80 €".to_string())
        );
    }

    #[test]
    fn test_centimes_only() {
        assert_eq!(parse("cinquante centimes"), Some("0,50 €".to_string()));
        assert_eq!(parse("un centime"), Some("0,01 €".to_string()));
        assert_eq!(parse("vingt centimes"), Some("0,20 €".to_string()));
    }

    #[test]
    fn test_dollars() {
        assert_eq!(parse("deux dollars"), Some("2 $".to_string()));
        assert_eq!(parse("deux dollars vingt"), Some("2,20 $".to_string()));
    }

    #[test]
    fn test_other_currencies() {
        assert_eq!(parse("un franc suisse"), Some("1 CHF".to_string()));
        assert_eq!(parse("trois livre"), Some("3 £".to_string()));
        assert_eq!(parse("trois pence"), Some("0,03 £".to_string()));
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(
            parse("quatre-vingt mille won"),
            Some("80 000 ₩".to_string())
        );
        assert_eq!(
            parse("quatre-vingt-mille won"),
            Some("80 000 ₩".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("cinq"), None);
    }
}
