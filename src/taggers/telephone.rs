//! Telephone number tagger.
//!
//! Converts spoken phone numbers, IP addresses, and serial numbers to written form:
//! - "one two three one two three five six seven eight" → "123-123-5678"
//! - "plus forty four one two three..." → "+44 123-123-5678"
//! - "one two three dot one two three dot o dot four o" → "123.123.0.40"

use super::cardinal::words_to_number;

/// Parse spoken telephone/serial number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    // Reject input with punctuation (commas, etc.)
    if input_trimmed.contains(',') {
        return None;
    }

    // Try IP address pattern first (contains "dot")
    if input_trimmed.contains(" dot ") {
        return parse_ip_address(input_trimmed);
    }

    // Try SSN pattern (contains "ssn")
    if input_trimmed.contains("ssn") {
        return parse_ssn_in_context(input, input_trimmed);
    }

    // Try alphanumeric product/serial code patterns
    if let Some(result) = parse_alphanumeric_code(input) {
        return Some(result);
    }

    // Must have digit content
    if !has_digit_content(input_trimmed) {
        return None;
    }

    // Don't match if input has scale words (billion, million, etc.)
    if has_scale_words(input_trimmed) {
        return None;
    }

    // Try phone number pattern
    parse_phone_number(input_trimmed)
}

/// Parse IP address pattern: "one two three dot one two three dot o dot four o"
fn parse_ip_address(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.split(" dot ").collect();
    if parts.len() < 2 {
        return None;
    }

    let mut octets = Vec::new();
    for part in parts {
        let octet = parse_ip_octet(part)?;
        octets.push(octet);
    }

    Some(octets.join("."))
}

/// Parse a single IP octet
fn parse_ip_octet(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    // Try parsing as compound number sequence
    // e.g., "one twenty three" = 1 + 23 = "123"
    // e.g., "forty five" = "45"
    // e.g., "double five" = "55"

    let mut result = String::new();
    let mut i = 0;

    while i < words.len() {
        let word = words[i];

        // Handle "double X"
        if word == "double" && i + 1 < words.len() {
            let next = words[i + 1];
            if let Some(d) = word_to_digit(next) {
                result.push(d);
                result.push(d);
                i += 2;
                continue;
            } else if let Some(num) = words_to_number(next) {
                let s = (num as i64).to_string();
                result.push_str(&s);
                result.push_str(&s);
                i += 2;
                continue;
            }
        }

        // Try single digit
        if let Some(d) = word_to_digit(word) {
            result.push(d);
            i += 1;
            continue;
        }

        // Try compound number (e.g., "twenty three", "forty five")
        if i + 1 < words.len() {
            let combined = format!("{} {}", word, words[i + 1]);
            if let Some(num) = words_to_number(&combined) {
                result.push_str(&(num as i64).to_string());
                i += 2;
                continue;
            }
        }

        // Try single number word (e.g., "forty")
        if let Some(num) = words_to_number(word) {
            result.push_str(&(num as i64).to_string());
            i += 1;
            continue;
        }

        i += 1;
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Parse SSN in context: "ssn is seven double nine one two three double one three"
/// Preserves original casing of "SSN" from input
fn parse_ssn_in_context(original_input: &str, input: &str) -> Option<String> {
    // Find where SSN digits start
    let ssn_idx = input.find("ssn")?;
    let prefix = &input[..ssn_idx];
    let after_ssn = &input[ssn_idx + 3..].trim_start();

    // Get original SSN casing from the original input
    let orig_ssn_idx = original_input.to_lowercase().find("ssn")?;
    let orig_ssn = &original_input[orig_ssn_idx..orig_ssn_idx + 3];

    // Skip "is" if present
    let digits_part = if after_ssn.starts_with("is ") {
        &after_ssn[3..]
    } else {
        after_ssn
    };

    let digits = parse_digit_sequence_with_double(digits_part)?;

    // SSN format: XXX-XX-XXXX
    if digits.len() >= 9 {
        let formatted = format!("{}-{}-{}", &digits[0..3], &digits[3..5], &digits[5..9]);
        if prefix.is_empty() {
            Some(format!("{} is {}", orig_ssn, formatted))
        } else {
            Some(format!("{}{} is {}", prefix.trim(), orig_ssn, formatted))
        }
    } else {
        None
    }
}

/// Parse alphanumeric product/serial codes like "x eighty six" → "x86"
fn parse_alphanumeric_code(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }

    // Check if this looks like an alphanumeric pattern (mix of letters and number words)
    let has_letters = words.iter().any(|w| is_single_letter(&w.to_lowercase()));
    let has_numbers = words.iter().any(|w| {
        let wl = w.to_lowercase();
        word_to_digit(&wl).is_some() || is_tens_word(&wl) || is_number_word(&wl)
    });

    if !has_letters || !has_numbers {
        return None;
    }

    // Check for compact serial code pattern: starts with digit word, has interspersed letters
    // e.g., "five w k r a three one" → "5wkra31"
    // vs regular patterns that need spacing like "a thirty six" or "r t x forty fifty t i"
    let first_word_lower = words[0].to_lowercase();
    let starts_with_digit = word_to_digit(&first_word_lower).is_some();
    let is_compact_code = starts_with_digit
        && words.iter().all(|w| {
            let wl = w.to_lowercase();
            is_single_letter(&wl)
                || word_to_digit(&wl).is_some()
                || is_tens_word(&wl)
                || is_number_word(&wl)
        });

    // Build result by parsing each component
    let mut result = String::new();
    let mut i = 0;
    let mut letter_run = String::new();
    let mut prev_was_number = false; // Track if we just output a number

    while i < words.len() {
        let word = words[i];
        let word_lower = word.to_lowercase();

        // Single letter - accumulate in letter_run
        if is_single_letter(&word_lower) {
            letter_run.push_str(&word_lower);
            // Don't reset prev_was_number here - letters may be suffix to previous number
            i += 1;
            continue;
        }

        // Flush letter run before number
        if !letter_run.is_empty() {
            // If previous output was a number, letters join directly (1080p, 4050ti)
            // Otherwise add space before if result is not empty (unless compact code)
            if !prev_was_number && !is_compact_code && !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            // Check if this should be uppercased (common abbreviations)
            if should_uppercase_abbrev(&letter_run) {
                result.push_str(&letter_run.to_uppercase());
            } else {
                result.push_str(&letter_run);
            }
            // Add space after letter run before number (unless compact code or known no-space patterns)
            // Also don't add space if letters came right after number (they're a suffix like "p" in 1080p)
            if !prev_was_number && !is_compact_code && !should_join_letters_to_number(&letter_run) {
                result.push(' ');
            }
            letter_run.clear();
            prev_was_number = false;
        }

        // Check for "X0 Y0" pattern (e.g., "forty fifty" = 4050, "ten eighty" = 1080)
        if i + 1 < words.len() && is_tens_word(&word_lower) {
            let next_word = words[i + 1].to_lowercase();
            if is_tens_word(&next_word) {
                // "forty fifty" → 4050
                if let (Some(tens1), Some(tens2)) = (words_to_number(&word_lower), words_to_number(&next_word)) {
                    let combined = (tens1 / 10) * 1000 + tens2;
                    result.push_str(&combined.to_string());
                    i += 2;
                    prev_was_number = true;
                    continue;
                }
            }
        }

        // Check for "ten eighty" = 1080 pattern (teens + tens)
        if i + 1 < words.len() && is_teen_word(&word_lower) {
            let next_word = words[i + 1].to_lowercase();
            if is_tens_word(&next_word) {
                if let (Some(teen), Some(tens)) = (words_to_number(&word_lower), words_to_number(&next_word)) {
                    let combined = teen * 100 + tens;
                    result.push_str(&combined.to_string());
                    i += 2;
                    prev_was_number = true;
                    continue;
                }
            }
        }

        // Try compound number ("eighty six" = 86)
        if i + 1 < words.len() && is_tens_word(&word_lower) {
            let next_word = words[i + 1].to_lowercase();
            let compound = format!("{} {}", word_lower, next_word);
            if let Some(num) = words_to_number(&compound) {
                // Check it's actually a compound (tens + units) not just tens + something else
                if num > words_to_number(&word_lower).unwrap_or(0) {
                    result.push_str(&num.to_string());
                    i += 2;
                    prev_was_number = true;
                    continue;
                }
            }
        }

        // Single digit word
        if let Some(d) = word_to_digit(&word_lower) {
            result.push(d);
            i += 1;
            prev_was_number = true;
            continue;
        }

        // Single number word (tens or teens)
        if let Some(num) = words_to_number(&word_lower) {
            if num >= 10 && num <= 99 {
                result.push_str(&num.to_string());
                i += 1;
                prev_was_number = true;
                continue;
            }
        }

        // Unknown word - keep as-is with space if needed
        if !result.is_empty() && !result.ends_with(' ') {
            result.push(' ');
        }
        result.push_str(word);
        i += 1;
        prev_was_number = false;
    }

    // Flush remaining letters
    if !letter_run.is_empty() {
        if should_uppercase_abbrev(&letter_run) {
            result.push_str(&letter_run.to_uppercase());
        } else {
            result.push_str(&letter_run);
        }
    }

    if result.is_empty() || result == input {
        None
    } else {
        Some(result)
    }
}

fn is_single_letter(word: &str) -> bool {
    // Single ASCII letter, but NOT 'o' which means zero in phone/serial contexts
    if word.len() != 1 {
        return false;
    }
    let c = word.chars().next().unwrap_or(' ');
    c.is_ascii_alphabetic() && c != 'o' && c != 'O'
}

fn is_number_word(word: &str) -> bool {
    is_tens_word(word) || is_teen_word(word) || word_to_digit(word).is_some()
}

fn is_teen_word(word: &str) -> bool {
    matches!(
        word,
        "ten" | "eleven" | "twelve" | "thirteen" | "fourteen" | "fifteen" | "sixteen"
            | "seventeen" | "eighteen" | "nineteen"
    )
}

fn should_uppercase_abbrev(s: &str) -> bool {
    // Common uppercase abbreviations in product names
    matches!(s, "rtx" | "gtx" | "rx" | "amd" | "cpu" | "gpu" | "usb" | "hdmi")
}

fn should_join_letters_to_number(s: &str) -> bool {
    // Single "x" prefix joins with number (x86, x386)
    // Other letters get a space before the number
    s == "x"
}

/// Parse phone number
fn parse_phone_number(input: &str) -> Option<String> {
    let has_plus = input.starts_with("plus ");

    // Parse prefix and digits
    let (prefix, rest) = extract_phone_prefix(input);
    let digits = parse_digit_sequence_with_double(rest)?;

    // Must have at least 7 digits for phone number (or 3 for short codes)
    if !has_plus && digits.len() < 3 {
        return None;
    }

    // Format the number
    let formatted = format_phone_number(&digits);

    if prefix.is_empty() {
        Some(formatted)
    } else {
        Some(format!("{} {}", prefix, formatted))
    }
}

/// Check if word is a tens word (twenty, thirty, etc.)
fn is_tens_word(word: &str) -> bool {
    matches!(
        word,
        "twenty" | "thirty" | "forty" | "fifty" | "sixty" | "seventy" | "eighty" | "ninety"
    )
}

/// Extract phone prefix (country code with +)
fn extract_phone_prefix(input: &str) -> (String, &str) {
    if !input.starts_with("plus ") {
        return (String::new(), input);
    }

    let rest = &input[5..];
    let words: Vec<&str> = rest.split_whitespace().collect();

    // Try to parse country code (could be "forty four" = 44, or "nine one" = 91)
    // Country codes are 1-3 digits
    let mut code = String::new();
    let mut consumed_words = 0;

    // First, try compound number ONLY for tens+units patterns (e.g., "forty four" = 44)
    // NOT for digit sequences like "nine one" which should stay as "91" (individual digits)
    if words.len() >= 2 && is_tens_word(words[0]) {
        let compound = format!("{} {}", words[0], words[1]);
        if let Some(num) = words_to_number(&compound) {
            if num >= 10 && num <= 999 {
                code = (num as i64).to_string();
                consumed_words = 2;
            }
        }
    }

    // If no compound match, try single tens word or individual digits
    if code.is_empty() {
        for (i, word) in words.iter().enumerate() {
            if let Some(d) = word_to_digit(word) {
                code.push(d);
                consumed_words = i + 1;
                // For individual digit words, limit to 2 digits (common country codes)
                // 3-digit codes like 351 are usually spoken as compound ("three fifty one")
                if code.len() >= 2 {
                    break;
                }
            } else if is_tens_word(word) {
                // Single tens word like "forty" = 40
                if let Some(num) = words_to_number(word) {
                    if code.is_empty() && num >= 10 && num <= 99 {
                        code = (num as i64).to_string();
                        consumed_words = i + 1;
                        break;
                    }
                }
                break;
            } else {
                break;
            }
        }
    }

    if code.is_empty() {
        return (String::new(), input);
    }

    let remaining = words[consumed_words..].join(" ");
    // Find position in original string
    let remaining_start = if remaining.is_empty() {
        input.len()
    } else {
        input.find(&remaining).unwrap_or(input.len())
    };

    (format!("+{}", code), &input[remaining_start..])
}

/// Parse digit sequence handling "double X" patterns
fn parse_digit_sequence_with_double(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < words.len() {
        let word = words[i];

        // Handle "double X"
        if word == "double" && i + 1 < words.len() {
            if let Some(d) = word_to_digit(words[i + 1]) {
                result.push(d);
                result.push(d);
                i += 2;
                continue;
            } else if let Some(num) = words_to_number(words[i + 1]) {
                let s = (num as i64).to_string();
                result.push_str(&s);
                result.push_str(&s);
                i += 2;
                continue;
            }
        }

        // Handle "triple X"
        if word == "triple" && i + 1 < words.len() {
            if let Some(d) = word_to_digit(words[i + 1]) {
                result.push(d);
                result.push(d);
                result.push(d);
                i += 2;
                continue;
            }
        }

        // Handle single digit
        if let Some(d) = word_to_digit(word) {
            result.push(d);
            i += 1;
            continue;
        }

        // Handle compound numbers (twenty three = 23)
        if let Some(num) = words_to_number(word) {
            // Check if next word is a units digit
            if i + 1 < words.len() {
                let combined = format!("{} {}", word, words[i + 1]);
                if let Some(compound) = words_to_number(&combined) {
                    if compound != num {
                        result.push_str(&(compound as i64).to_string());
                        i += 2;
                        continue;
                    }
                }
            }
            result.push_str(&(num as i64).to_string());
            i += 1;
            continue;
        }

        // Skip unknown words
        i += 1;
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Check if input contains digit words
fn has_digit_content(input: &str) -> bool {
    let digit_words = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        "oh", "o", "double", "triple", "ten", "eleven", "twelve", "thirteen", "fourteen",
        "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty", "thirty",
        "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];

    for word in input.split_whitespace() {
        if digit_words.contains(&word) {
            return true;
        }
    }
    false
}

/// Check if input has scale words (indicates cardinal, not phone)
fn has_scale_words(input: &str) -> bool {
    let scale_words = [
        "hundred", "thousand", "million", "billion", "trillion",
        "quadrillion", "quintillion", "sextillion", "crore", "lakh",
    ];

    for word in input.split_whitespace() {
        if scale_words.contains(&word) {
            return true;
        }
    }
    false
}

/// Convert word to single digit
fn word_to_digit(word: &str) -> Option<char> {
    match word {
        "zero" | "o" | "oh" => Some('0'),
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        _ => None,
    }
}

/// Format phone number
fn format_phone_number(digits: &str) -> String {
    let len = digits.len();

    // 11 digits: X XXX-XXX-XXXX (single digit prefix + 10-digit number)
    if len == 11 {
        return format!(
            "{} {}-{}-{}",
            &digits[0..1],
            &digits[1..4],
            &digits[4..7],
            &digits[7..11]
        );
    }

    // 10 digits: XXX-XXX-XXXX
    if len == 10 {
        return format!("{}-{}-{}", &digits[0..3], &digits[3..6], &digits[6..10]);
    }

    // 7 digits: XXX-XXXX
    if len == 7 {
        return format!("{}-{}", &digits[0..3], &digits[3..7]);
    }

    // 3 digits: just return as-is
    if len == 3 {
        return digits.to_string();
    }

    // Other lengths - group as XXX-rest
    if len > 3 {
        return format!("{}-{}", &digits[0..3], &digits[3..]);
    }

    digits.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_phone() {
        assert_eq!(
            parse("one two three one two three five six seven eight"),
            Some("123-123-5678".to_string())
        );
    }

    #[test]
    fn test_with_country_code() {
        assert_eq!(
            parse("plus nine one one two three one two three five six seven eight"),
            Some("+91 123-123-5678".to_string())
        );
    }

    #[test]
    fn test_double_pattern() {
        assert_eq!(
            parse("double oh three one two three five six seven eight"),
            Some("003-123-5678".to_string())
        );
    }

    #[test]
    fn test_three_digits() {
        assert_eq!(parse("seven nine nine"), Some("799".to_string()));
    }

    #[test]
    fn test_ip_address() {
        assert_eq!(
            parse("one two three dot one two three dot o dot four o"),
            Some("123.123.0.40".to_string())
        );
    }
}
