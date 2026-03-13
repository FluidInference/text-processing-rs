//! Time tagger for Japanese.
//!
//! Converts kanji time expressions to Arabic numeral form:
//! - "七時一分" → "7時1分"
//! - "正午一分前" → "正午1分前"
//! - "零時" → "0時"

use super::cardinal;

/// Process time patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = input.to_string();

    // Process 時 patterns (convert kanji before 時)
    result = process_hour(&result);

    // Process 分 patterns (convert kanji before 分, but not X分の which is fractions)
    result = process_minute(&result);

    result
}

/// Process 時 suffix: convert kanji numbers before 時 to Arabic.
fn process_hour(input: &str) -> String {
    let suffix = "時";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let before = &remaining[..pos];
        let before_chars: Vec<char> = before.chars().collect();

        // Scan backwards for kanji number
        let mut num_start = before_chars.len();
        while num_start > 0 && cardinal::is_kanji_numeral(before_chars[num_start - 1]) {
            num_start -= 1;
        }

        // Also handle 零 (not in is_kanji_numeral but is a valid hour digit)
        while num_start > 0 && before_chars[num_start - 1] == '零' {
            num_start -= 1;
        }

        if num_start < before_chars.len() {
            let prefix: String = before_chars[..num_start].iter().collect();
            let kanji: String = before_chars[num_start..].iter().collect();
            result.push_str(&prefix);

            // Handle 零 specially
            if kanji == "零" {
                result.push('0');
            } else if let Some(num) = cardinal::kanji_to_number(&kanji) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&kanji);
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

/// Process 分 suffix: convert kanji numbers before 分 to Arabic.
/// Skip if followed by の (fraction pattern handled elsewhere).
fn process_minute(input: &str) -> String {
    let suffix = "分";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let after_suffix = &remaining[pos + suffix.len()..];

        // Skip if this is a fraction pattern (分の)
        if after_suffix.starts_with('の') {
            result.push_str(&remaining[..pos + suffix.len()]);
            remaining = after_suffix;
            continue;
        }

        let before = &remaining[..pos];
        let before_chars: Vec<char> = before.chars().collect();

        // Scan backwards for kanji number
        let mut num_start = before_chars.len();
        while num_start > 0 && cardinal::is_kanji_numeral(before_chars[num_start - 1]) {
            num_start -= 1;
        }

        if num_start < before_chars.len() {
            let prefix: String = before_chars[..num_start].iter().collect();
            let kanji: String = before_chars[num_start..].iter().collect();
            result.push_str(&prefix);
            if let Some(num) = cardinal::kanji_to_number(&kanji) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&kanji);
            }
        } else {
            result.push_str(before);
        }

        result.push_str(suffix);
        remaining = after_suffix;
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("七時一分"), "7時1分");
        assert_eq!(process("零時"), "0時");
        assert_eq!(process("三時"), "3時");
    }

    #[test]
    fn test_modifiers() {
        assert_eq!(process("九時十分前"), "9時10分前");
        assert_eq!(process("正午十分過ぎ"), "正午10分過ぎ");
        assert_eq!(process("七時五十分頃"), "7時50分頃");
    }

    #[test]
    fn test_noon() {
        assert_eq!(process("正午一分前"), "正午1分前");
    }

    #[test]
    fn test_skip_fraction() {
        // 分の should not be processed as time
        assert_eq!(process("三分の一"), "三分の一");
    }
}
