//! Decimal number tagger for Hindi.
//!
//! Converts Hindi decimal expressions to Devanagari form:
//! - "दो सौ छह दशमलव दो नौ" → "२०६.२९"
//! - "साढ़े तीन सौ दशमलव दो दो" → "३५०.२२"
//!
//! Uses "दशमलव" as the decimal point marker.
//! Fractional digits are parsed individually.

use super::cardinal;

/// Process decimal patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    // Find "दशमलव" and split into integer part + fractional part
    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        if words[i] == "दशमलव" {
            // Find the integer part before "दशमलव"
            let (int_start, int_val) = find_number_before(&words, &result, i);

            // Find the fractional digits after "दशमलव"
            let (frac_end, frac_digits) = find_frac_digits_after(&words, i + 1);

            if let (Some(int_val), Some(frac_digits)) = (int_val, frac_digits) {
                // Remove integer words from result
                let to_remove = result.len() - int_start;
                for _ in 0..to_remove {
                    result.pop();
                }

                let int_str = cardinal::to_devanagari(int_val);
                let frac_str = frac_digits
                    .iter()
                    .map(|&d| cardinal::to_devanagari_digit(d as u8))
                    .collect::<String>();
                result.push(format!("{}.{}", int_str, frac_str));
                i = frac_end;
                continue;
            }
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Find the number words before position `pos` in the word list.
/// Returns (start_index_in_result, value).
fn find_number_before(words: &[&str], result: &[String], pos: usize) -> (usize, Option<i64>) {
    if pos == 0 {
        return (result.len(), None);
    }

    // Scan backwards to find number words
    let mut start = pos;
    while start > 0 {
        let w = words[start - 1];
        if cardinal::is_hi_number_word(w) || cardinal::is_modifier(w) {
            start -= 1;
        } else {
            break;
        }
    }

    if start == pos {
        return (result.len(), None);
    }

    let num_words: Vec<&str> = words[start..pos].to_vec();
    let val = cardinal::words_to_number(&num_words);
    let result_start = result.len() - (pos - start);

    (result_start, val)
}

/// Find fractional digit words after position `pos`.
/// Returns (end_index, digits).
fn find_frac_digits_after(words: &[&str], start: usize) -> (usize, Option<Vec<i64>>) {
    let mut digits = Vec::new();
    let mut end = start;

    while end < words.len() {
        if let Some(v) = cardinal::word_to_value(words[end]) {
            if v <= 9 {
                digits.push(v);
                end += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if digits.is_empty() {
        (start, None)
    } else {
        (end, Some(digits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("दो सौ छह दशमलव दो नौ"), "२०६.२९");
    }

    #[test]
    fn test_modifier() {
        assert_eq!(process("साढ़े तीन सौ दशमलव दो दो"), "३५०.२२");
    }
}
