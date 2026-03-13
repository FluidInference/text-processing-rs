//! Fraction tagger for Chinese.
//!
//! Converts Chinese fractions to Arabic numeral form:
//! - "五分之一" → "1/5"
//! - "一又二分之一" → "1又1/2"
//!
//! Chinese fractions use X分之Y where X is denominator and Y is numerator.
//! Mixed numbers use X又Y分之Z → X又Z/Y.

use super::cardinal;

/// Process fraction patterns in a string.
pub fn process(input: &str) -> String {
    let fen_zhi = "分之";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(fz_pos) = remaining.find(fen_zhi) {
        let before_fz = &remaining[..fz_pos];
        let after_fz = &remaining[fz_pos + fen_zhi.len()..];

        // Parse denominator: Chinese numerals immediately before 分之
        let before_chars: Vec<char> = before_fz.chars().collect();
        let mut denom_start = before_chars.len();
        while denom_start > 0 && cardinal::is_zh_numeral(before_chars[denom_start - 1]) {
            denom_start -= 1;
        }

        if denom_start >= before_chars.len() {
            // No Chinese numeral before 分之, pass through
            result.push_str(&remaining[..fz_pos + fen_zhi.len()]);
            remaining = after_fz;
            continue;
        }

        let denom_kanji: String = before_chars[denom_start..].iter().collect();
        let denom = match cardinal::zh_to_number(&denom_kanji) {
            Some(d) => d,
            None => {
                result.push_str(&remaining[..fz_pos + fen_zhi.len()]);
                remaining = after_fz;
                continue;
            }
        };

        // Parse numerator: Chinese numerals immediately after 分之
        let after_chars: Vec<char> = after_fz.chars().collect();
        let mut numer_end = 0;
        while numer_end < after_chars.len() && cardinal::is_zh_numeral(after_chars[numer_end]) {
            numer_end += 1;
        }

        if numer_end == 0 {
            result.push_str(&remaining[..fz_pos + fen_zhi.len()]);
            remaining = after_fz;
            continue;
        }

        let numer_kanji: String = after_chars[..numer_end].iter().collect();
        let numer = match cardinal::zh_to_number(&numer_kanji) {
            Some(n) => n,
            None => {
                result.push_str(&remaining[..fz_pos + fen_zhi.len()]);
                remaining = after_fz;
                continue;
            }
        };

        let numer_byte_len: usize = after_chars[..numer_end].iter().map(|c| c.len_utf8()).sum();

        // Build prefix before denominator
        let prefix: String = before_chars[..denom_start].iter().collect();

        // Check for mixed number: X又Y分之Z
        if prefix.ends_with('又') {
            let before_you = &prefix[..prefix.len() - '又'.len_utf8()];
            let by_chars: Vec<char> = before_you.chars().collect();
            let mut whole_start = by_chars.len();
            while whole_start > 0 && cardinal::is_zh_numeral(by_chars[whole_start - 1]) {
                whole_start -= 1;
            }

            if whole_start < by_chars.len() {
                let whole_kanji: String = by_chars[whole_start..].iter().collect();
                if let Some(whole) = cardinal::zh_to_number(&whole_kanji) {
                    let real_prefix: String = by_chars[..whole_start].iter().collect();
                    result.push_str(&real_prefix);
                    result.push_str(&format!("{}又{}/{}", whole, numer, denom));
                    remaining = &after_fz[numer_byte_len..];
                    continue;
                }
            }
        }

        // Simple fraction
        result.push_str(&prefix);
        result.push_str(&format!("{}/{}", numer, denom));
        remaining = &after_fz[numer_byte_len..];
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("五分之一"), "1/5");
        assert_eq!(process("二分之一"), "1/2");
        assert_eq!(process("十分之五"), "5/10");
    }

    #[test]
    fn test_mixed() {
        assert_eq!(process("三又五分之一"), "3又1/5");
        assert_eq!(process("一又二分之一"), "1又1/2");
    }
}
