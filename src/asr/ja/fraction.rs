//! Fraction tagger for Japanese.
//!
//! Converts kanji fractions to Arabic numeral form:
//! - "八分の五" → "5/8"
//! - "マイナス八分の五" → "-5/8"
//! - "一と四分の三" → "1 3/4"
//! - "一荷四分の三" → "1 3/4"
//!
//! Japanese fractions use X分のY where X is denominator and Y is numerator.

use super::cardinal;

/// Process fraction patterns in a string.
pub fn process(input: &str) -> String {
    let bun_no = "分の";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(bun_no_pos) = remaining.find(bun_no) {
        let before_bun_no = &remaining[..bun_no_pos];
        let after_bun_no = &remaining[bun_no_pos + bun_no.len()..];

        // Parse denominator: kanji number immediately before 分の
        let before_chars: Vec<char> = before_bun_no.chars().collect();
        let mut denom_start = before_chars.len();
        while denom_start > 0 && cardinal::is_kanji_numeral(before_chars[denom_start - 1]) {
            denom_start -= 1;
        }

        if denom_start >= before_chars.len() {
            // No kanji number before 分の, pass through
            result.push_str(&remaining[..bun_no_pos + bun_no.len()]);
            remaining = after_bun_no;
            continue;
        }

        let denom_kanji: String = before_chars[denom_start..].iter().collect();
        let denom = match cardinal::kanji_to_number(&denom_kanji) {
            Some(d) => d,
            None => {
                result.push_str(&remaining[..bun_no_pos + bun_no.len()]);
                remaining = after_bun_no;
                continue;
            }
        };

        // Parse numerator: kanji number immediately after 分の
        let after_chars: Vec<char> = after_bun_no.chars().collect();
        let mut numer_end = 0;
        while numer_end < after_chars.len() && cardinal::is_kanji_numeral(after_chars[numer_end]) {
            numer_end += 1;
        }

        if numer_end == 0 {
            // No kanji number after 分の, pass through
            result.push_str(&remaining[..bun_no_pos + bun_no.len()]);
            remaining = after_bun_no;
            continue;
        }

        let numer_kanji: String = after_chars[..numer_end].iter().collect();
        let numer = match cardinal::kanji_to_number(&numer_kanji) {
            Some(n) => n,
            None => {
                result.push_str(&remaining[..bun_no_pos + bun_no.len()]);
                remaining = after_bun_no;
                continue;
            }
        };

        let numer_byte_len: usize = after_chars[..numer_end].iter().map(|c| c.len_utf8()).sum();

        // Build prefix before denominator
        let prefix_before_denom: String = before_chars[..denom_start].iter().collect();

        // Check for mixed number: XとY分のZ or X荷Y分のZ
        if let Some((real_prefix, whole, negative)) =
            find_mixed_prefix(&prefix_before_denom)
        {
            result.push_str(real_prefix);
            if negative {
                result.push_str(&format!("-{} {}/{}", whole, numer, denom));
            } else {
                result.push_str(&format!("{} {}/{}", whole, numer, denom));
            }
        } else if prefix_before_denom.ends_with("マイナス") {
            // Negative fraction
            let prefix = &prefix_before_denom[..prefix_before_denom.len() - "マイナス".len()];
            result.push_str(prefix);
            result.push_str(&format!("-{}/{}", numer, denom));
        } else {
            // Simple fraction
            result.push_str(&prefix_before_denom);
            result.push_str(&format!("{}/{}", numer, denom));
        }

        remaining = &after_bun_no[numer_byte_len..];
    }

    result.push_str(remaining);
    result
}

/// Check for mixed number prefix (XとY or X荷Y) in the text before the denominator.
/// Returns (text_before_whole, whole_number, is_negative) if found.
fn find_mixed_prefix(before_denom: &str) -> Option<(&str, i64, bool)> {
    for separator in &["と", "荷"] {
        if let Some(sep_pos) = before_denom.rfind(separator) {
            let before_sep = &before_denom[..sep_pos];
            let before_sep_chars: Vec<char> = before_sep.chars().collect();

            // Find kanji number before separator
            let mut num_start = before_sep_chars.len();
            while num_start > 0 && cardinal::is_kanji_numeral(before_sep_chars[num_start - 1]) {
                num_start -= 1;
            }

            if num_start < before_sep_chars.len() {
                let kanji: String = before_sep_chars[num_start..].iter().collect();
                if let Some(whole) = cardinal::kanji_to_number(&kanji) {
                    let prefix = &before_sep[..before_sep.len() - kanji.len()];

                    let (real_prefix, is_negative) = if prefix.ends_with("マイナス") {
                        (&prefix[..prefix.len() - "マイナス".len()], true)
                    } else {
                        (prefix, false)
                    };

                    return Some((real_prefix, whole, is_negative));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("八分の五"), "5/8");
        assert_eq!(process("三分の一"), "1/3");
    }

    #[test]
    fn test_negative() {
        assert_eq!(process("マイナス八分の五"), "-5/8");
    }

    #[test]
    fn test_mixed() {
        assert_eq!(process("一と四分の三"), "1 3/4");
        assert_eq!(process("マイナス一荷四分の三"), "-1 3/4");
    }

    #[test]
    fn test_contextual() {
        assert_eq!(process("答えはマイナス八分の五"), "答えは-5/8");
        assert_eq!(
            process("三分の一の人がその場を離れた"),
            "1/3の人がその場を離れた"
        );
    }
}
