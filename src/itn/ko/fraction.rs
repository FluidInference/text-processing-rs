//! Fraction tagger for Korean.
//!
//! Converts spoken Korean fractions to written form:
//! - "삼분의 일" → "1/3"
//! - "사분의삼" → "3/4"
//! - "마이너스 삼분의 일" → "-1/3"
//! - "이와 삼분의 일" → "2 1/3"
//!
//! Korean fractions use `X분의 Y` where X is the denominator and Y is
//! the numerator (the same denominator-first order as Japanese 分の).
//! A single space between 분의 and the numerator is tolerated.

use super::cardinal;

const BUNUI: &str = "분의";
const MINUS: &str = "마이너스";

/// Process fraction patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(bunui_pos) = remaining.find(BUNUI) {
        let before = &remaining[..bunui_pos];
        let after = &remaining[bunui_pos + BUNUI.len()..];

        // Denominator: Sino-Korean run immediately before 분의.
        let before_chars: Vec<char> = before.chars().collect();
        let mut denom_start = before_chars.len();
        while denom_start > 0 && cardinal::is_sino_korean_numeral(before_chars[denom_start - 1]) {
            denom_start -= 1;
        }
        if denom_start >= before_chars.len() {
            result.push_str(&remaining[..bunui_pos + BUNUI.len()]);
            remaining = after;
            continue;
        }
        let denom_span: String = before_chars[denom_start..].iter().collect();
        let denom = match cardinal::sino_korean_to_number(&denom_span) {
            Some(d) => d,
            None => {
                result.push_str(&remaining[..bunui_pos + BUNUI.len()]);
                remaining = after;
                continue;
            }
        };

        // Numerator: Sino-Korean run after 분의 (one optional space).
        let after_trimmed = after.trim_start();
        let after_chars: Vec<char> = after_trimmed.chars().collect();
        let mut numer_end = 0;
        while numer_end < after_chars.len()
            && cardinal::is_sino_korean_numeral(after_chars[numer_end])
        {
            numer_end += 1;
        }
        if numer_end == 0 {
            result.push_str(&remaining[..bunui_pos + BUNUI.len()]);
            remaining = after;
            continue;
        }
        let numer_span: String = after_chars[..numer_end].iter().collect();
        let numer = match cardinal::sino_korean_to_number(&numer_span) {
            Some(n) => n,
            None => {
                result.push_str(&remaining[..bunui_pos + BUNUI.len()]);
                remaining = after;
                continue;
            }
        };

        let prefix_before_denom: String = before_chars[..denom_start].iter().collect();

        if let Some((real_prefix, whole, negative)) = find_mixed_prefix(&prefix_before_denom) {
            result.push_str(real_prefix);
            if negative {
                result.push_str(&format!("-{} {}/{}", whole, numer, denom));
            } else {
                result.push_str(&format!("{} {}/{}", whole, numer, denom));
            }
        } else if let Some(p) = prefix_before_denom.trim_end().strip_suffix(MINUS) {
            // `마이너스` may be separated from the denominator by a space.
            result.push_str(p);
            result.push_str(&format!("-{}/{}", numer, denom));
        } else {
            result.push_str(&prefix_before_denom);
            result.push_str(&format!("{}/{}", numer, denom));
        }

        let consumed = after.len() - after_trimmed.len() + numer_span.len();
        remaining = &after[consumed..];
    }

    result.push_str(remaining);
    result
}

/// Check for a mixed-number prefix (`X와` / `X과`) before the
/// denominator. Returns (text_before_whole, whole_number, is_negative).
fn find_mixed_prefix(before_denom: &str) -> Option<(&str, i64, bool)> {
    for separator in &["와", "과"] {
        if let Some(sep_pos) = before_denom.rfind(separator) {
            let before_sep = before_denom[..sep_pos].trim_end();
            let before_sep_chars: Vec<char> = before_sep.chars().collect();

            let mut num_start = before_sep_chars.len();
            while num_start > 0 && cardinal::is_sino_korean_numeral(before_sep_chars[num_start - 1])
            {
                num_start -= 1;
            }

            if num_start < before_sep_chars.len() {
                let span: String = before_sep_chars[num_start..].iter().collect();
                if let Some(whole) = cardinal::sino_korean_to_number(&span) {
                    let prefix = &before_sep[..before_sep.len() - span.len()];
                    let (real_prefix, is_negative) = match prefix.trim_end().strip_suffix(MINUS) {
                        Some(p) => (p, true),
                        None => (prefix, false),
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
        assert_eq!(process("삼분의 일"), "1/3");
        assert_eq!(process("사분의삼"), "3/4");
        assert_eq!(process("이분의 일"), "1/2");
    }

    #[test]
    fn test_negative() {
        assert_eq!(process("마이너스 삼분의 일"), "-1/3");
    }

    #[test]
    fn test_mixed() {
        assert_eq!(process("이와 삼분의 일"), "2 1/3");
        assert_eq!(process("삼과 사분의 삼"), "3 3/4");
    }

    #[test]
    fn test_contextual() {
        assert_eq!(
            process("삼분의 일의 사람들이 떠났다"),
            "1/3의 사람들이 떠났다"
        );
    }
}
