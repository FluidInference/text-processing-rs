//! Date tagger for Chinese.
//!
//! Converts Chinese date expressions to Arabic numeral form:
//! - "一七九八年五月三十日" → "1798年5月30日"
//! - "公元一八三五年" → "公元1835年"
//! - "公元前一九九四年一月二日" → "公元前1994年1月2日"
//! - "纪元前一九三四年一月二日" → "公元前1934年1月2日"
//! - "纪元二零五六年二月三日" → "公元2056年2月3日"
//!
//! Year digits are parsed individually (一七九八 → 1798),
//! month and day use compound parsing (三十 → 30).

use super::cardinal;

/// Process date patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = input.to_string();

    // Normalize 纪元前 → 公元前, 纪元 → 公元 (must do 纪元前 first)
    result = result.replace("纪元前", "公元前");
    result = result.replace("纪元", "公元");

    // Process 年 patterns (year digits individually)
    result = process_year(&result);

    // Process 月 patterns
    result = process_suffix(&result, "月");

    // Process 日 patterns
    result = process_suffix(&result, "日");

    result
}

/// Process year: digits before 年 are parsed individually (one digit per kanji).
fn process_year(input: &str) -> String {
    let suffix = "年";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let before = &remaining[..pos];
        let before_chars: Vec<char> = before.chars().collect();

        // Scan backwards for Chinese digits (individual year digits)
        let mut num_start = before_chars.len();
        while num_start > 0 && cardinal::zh_digit(before_chars[num_start - 1]).is_some() {
            num_start -= 1;
        }

        if num_start < before_chars.len() {
            let prefix: String = before_chars[..num_start].iter().collect();
            result.push_str(&prefix);

            // Convert each digit individually
            for &c in &before_chars[num_start..] {
                if let Some(d) = cardinal::zh_digit(c) {
                    result.push_str(&d.to_string());
                } else {
                    result.push(c);
                }
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

/// Process generic suffix (月, 日): kanji number before suffix is compound-parsed.
fn process_suffix(input: &str, suffix: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let before = &remaining[..pos];
        let before_chars: Vec<char> = before.chars().collect();

        // Scan backwards for Chinese numerals
        let mut num_start = before_chars.len();
        while num_start > 0 && cardinal::is_zh_numeral(before_chars[num_start - 1]) {
            num_start -= 1;
        }

        if num_start < before_chars.len() {
            let prefix: String = before_chars[..num_start].iter().collect();
            let kanji: String = before_chars[num_start..].iter().collect();
            result.push_str(&prefix);
            if let Some(num) = cardinal::zh_to_number(&kanji) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_date() {
        assert_eq!(process("一七九八年五月三十日"), "1798年5月30日");
    }

    #[test]
    fn test_partial() {
        assert_eq!(process("五月三十日"), "5月30日");
        assert_eq!(process("一七九八年"), "1798年");
        assert_eq!(process("八月"), "8月");
    }

    #[test]
    fn test_gongyuan() {
        assert_eq!(
            process("公元一七九八年五月三十日"),
            "公元1798年5月30日"
        );
        assert_eq!(process("公元前一七九八年"), "公元前1798年");
    }

    #[test]
    fn test_jiyuan() {
        assert_eq!(
            process("纪元前一九三四年一月二日"),
            "公元前1934年1月2日"
        );
        assert_eq!(
            process("纪元二零五六年二月三日"),
            "公元2056年2月3日"
        );
    }
}
