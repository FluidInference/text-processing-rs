//! Telephone number tagger for Hindi.
//!
//! After cardinal processing, digit words have been converted to Devanagari digits.
//! This module concatenates sequences of single Devanagari digits into phone numbers:
//! - "१ १ १ १ १ १" → "१११११"
//! - "+९१ ९ ८ ७ ६ ..." → "+९१ ९८७६..."
//! - "०२ ०२ ..." → "०२०२..."
//!
//! Also handles प्लस prefix for international numbers and
//! digit words that cardinal may have left as single-character Devanagari digits.

/// Map English digit word to Devanagari digit.
fn english_digit_to_devanagari(word: &str) -> Option<char> {
    match word {
        "zero" => Some('०'),
        "one" => Some('१'),
        "two" => Some('२'),
        "three" => Some('३'),
        "four" => Some('४'),
        "five" => Some('५'),
        "six" => Some('६'),
        "seven" => Some('७'),
        "eight" => Some('८'),
        "nine" => Some('९'),
        _ => None,
    }
}

/// Check if a string is a single Devanagari digit.
fn is_devanagari_digit(s: &str) -> bool {
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        if chars.next().is_none() {
            return ('०'..='९').contains(&c);
        }
    }
    false
}

/// Check if a string is a multi-digit Devanagari number (already converted by cardinal).
fn is_devanagari_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('०'..='९').contains(&c))
}

/// Process telephone patterns in a string.
/// At this point, cardinal has already converted number words to Devanagari digits.
/// We concatenate sequences of single Devanagari digits (and small multi-digit groups).
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Check for "प्लस" prefix (international format)
        if words[i] == "प्लस" || words[i] == "+" || words[i] == "plus" {
            if let Some((phone_str, consumed)) = try_concat_devanagari_digits(&words, i + 1, 4) {
                // First two digits form country code
                let chars: Vec<char> = phone_str.chars().collect();
                if chars.len() >= 2 {
                    let country_code: String = chars[..2].iter().collect();
                    let rest: String = chars[2..].iter().collect();
                    result.push(format!("+{} {}", country_code, rest));
                } else {
                    result.push(format!("+{}", phone_str));
                }
                i += 1 + consumed;
                continue;
            }
        }

        // Check for sequence of Devanagari digit tokens (single digits or small numbers)
        if is_devanagari_digit(words[i]) || is_devanagari_number(words[i]) {
            if let Some((phone_str, consumed)) = try_concat_devanagari_digits(&words, i, 4) {
                result.push(phone_str);
                i += consumed;
                continue;
            }
        }

        // Check for English digit word sequences
        if english_digit_to_devanagari(words[i]).is_some() {
            if let Some((phone_str, consumed)) = try_concat_english_digits(&words, i, 4) {
                result.push(phone_str);
                i += consumed;
                continue;
            }
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Try to concatenate a sequence of English digit words into Devanagari digits.
fn try_concat_english_digits(words: &[&str], start: usize, min_digits: usize) -> Option<(String, usize)> {
    let mut digits = String::new();
    let mut i = start;

    while i < words.len() {
        if let Some(d) = english_digit_to_devanagari(words[i]) {
            digits.push(d);
            i += 1;
        } else {
            break;
        }
    }

    let digit_count = digits.chars().count();
    if digit_count >= min_digits {
        Some((digits, i - start))
    } else {
        None
    }
}

/// Try to concatenate a sequence of Devanagari digit tokens.
/// Each token should be a single Devanagari digit or small Devanagari number.
/// Requires at least `min_digits` total digits to form a phone number.
fn try_concat_devanagari_digits(words: &[&str], start: usize, min_digits: usize) -> Option<(String, usize)> {
    let mut digits = String::new();
    let mut i = start;

    while i < words.len() {
        if is_devanagari_number(words[i]) {
            digits.push_str(words[i]);
            i += 1;
        } else {
            break;
        }
    }

    let digit_count = digits.chars().count();
    if digit_count >= min_digits {
        Some((digits, i - start))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // After cardinal, "एक एक एक एक एक एक" → "१ १ १ १ १ १"
        assert_eq!(process("१ १ १ १ १ १"), "११११११");
        assert_eq!(process("१ २ ३ ४ ५ ६"), "१२३४५६");
    }

    #[test]
    fn test_international() {
        assert_eq!(
            process("प्लस ९ १ ९ ८ ७ ६ ५ ४ ३ २ १ ०"),
            "+९१ ९८७६५४३२१०"
        );
    }
}
