//! Ordinal number tagger for Korean.
//!
//! Converts Sino-Korean ordinals to Arabic numerals:
//! - "제일" → "제1"
//! - "제이십삼" → "제23"
//! - "일번째" → "1번째"
//!
//! Native-Korean ordinals (첫째, 둘째 …) use native numerals rather
//! than Sino-Korean syllables and are intentionally left untouched.

use super::cardinal;

/// Process ordinal patterns in a string.
/// Handles: 제X → 제N, X번째 → N번째
pub fn process(input: &str) -> String {
    let result = process_je(input);
    process_beonjjae(&result)
}

/// Replace Sino-Korean numbers after 제 with Arabic numerals.
fn process_je(input: &str) -> String {
    let prefix = "제";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(prefix) {
        result.push_str(&remaining[..pos]);
        result.push_str(prefix);

        let after = &remaining[pos + prefix.len()..];
        let chars: Vec<char> = after.chars().collect();

        let mut num_end = 0;
        while num_end < chars.len() && cardinal::is_sino_korean_numeral(chars[num_end]) {
            num_end += 1;
        }

        if num_end > 0 {
            let span: String = chars[..num_end].iter().collect();
            if let Some(num) = cardinal::sino_korean_to_number(&span) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&span);
            }
            remaining = &after[span.len()..];
        } else {
            remaining = after;
        }
    }

    result.push_str(remaining);
    result
}

/// Replace Sino-Korean numbers before 번째 with Arabic numerals.
fn process_beonjjae(input: &str) -> String {
    let suffix = "번째";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let before = &remaining[..pos];
        let chars: Vec<char> = before.chars().collect();

        let mut num_start = chars.len();
        while num_start > 0 && cardinal::is_sino_korean_numeral(chars[num_start - 1]) {
            num_start -= 1;
        }

        if num_start < chars.len() {
            let span: String = chars[num_start..].iter().collect();
            let prefix: String = chars[..num_start].iter().collect();
            result.push_str(&prefix);
            if let Some(num) = cardinal::sino_korean_to_number(&span) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&span);
            }
        } else {
            result.push_str(before);
        }

        result.push_str(suffix);
        remaining = &remaining[pos + suffix.len()..];
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_je() {
        assert_eq!(process("제일"), "제1");
        assert_eq!(process("제이십삼"), "제23");
        assert_eq!(process("제오백"), "제500");
    }

    #[test]
    fn test_beonjjae() {
        assert_eq!(process("일번째"), "1번째");
        assert_eq!(process("이십번째"), "20번째");
    }

    #[test]
    fn test_contextual() {
        assert_eq!(process("제이차 세계대전"), "제2차 세계대전");
    }
}
