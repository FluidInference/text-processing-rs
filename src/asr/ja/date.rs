//! Date tagger for Japanese.
//!
//! Converts kanji dates to Arabic numeral form:
//! - "一月二十二日" → "1月22日"
//! - "七十年代" → "70年代"
//! - "三月一日水曜日" → "3月1日(水)"
//! - "五から九日" → "5〜9日"

use super::cardinal;

/// Day-of-week mappings: full form → abbreviated form
const WEEKDAYS: &[(&str, &str)] = &[
    ("月曜日", "(月)"),
    ("火曜日", "(火)"),
    ("水曜日", "(水)"),
    ("木曜日", "(木)"),
    ("金曜日", "(金)"),
    ("土曜日", "(土)"),
    ("日曜日", "(日)"),
];

/// Process date patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = input.to_string();

    // Process day-of-week patterns first (before 日 processing)
    // e.g., "三月一日水曜日" → "三月一日(水)"
    for &(full, abbr) in WEEKDAYS {
        result = result.replace(full, abbr);
    }

    // Process range patterns: XからY日, XからY月, XからY年代
    result = process_ranges(&result);

    // Process 世紀 patterns
    result = process_suffix(&result, "世紀");

    // Process 年代 patterns
    result = process_suffix(&result, "年代");

    // Process 年 patterns (but not 年代)
    result = process_year(&result);

    // Process 月 patterns (but not 月曜日 which is already handled)
    result = process_suffix(&result, "月");

    // Process 日 patterns (but not 日曜日 etc.)
    result = process_day(&result);

    result
}

/// Process range patterns: "XからY日" → "X〜Y日"
fn process_ranges(input: &str) -> String {
    let kara = "から";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(kara_pos) = remaining.find(kara) {
        let before_kara = &remaining[..kara_pos];
        let after_kara = &remaining[kara_pos + kara.len()..];

        // Find kanji number before から
        let before_chars: Vec<char> = before_kara.chars().collect();
        let mut num_start = before_chars.len();
        while num_start > 0 && cardinal::is_kanji_numeral(before_chars[num_start - 1]) {
            num_start -= 1;
        }

        // Find kanji number + suffix after から
        let after_chars: Vec<char> = after_kara.chars().collect();
        let mut num_end = 0;
        while num_end < after_chars.len() && cardinal::is_kanji_numeral(after_chars[num_end]) {
            num_end += 1;
        }

        // Check if followed by a date suffix (日, 月, 年代)
        let after_num: String = after_chars[num_end..].iter().collect();
        let has_date_suffix = after_num.starts_with('日')
            || after_num.starts_with('月')
            || after_num.starts_with("年代");

        if num_start < before_chars.len() && num_end > 0 && has_date_suffix {
            let prefix: String = before_chars[..num_start].iter().collect();
            let num1_kanji: String = before_chars[num_start..].iter().collect();
            let num2_kanji: String = after_chars[..num_end].iter().collect();

            if let (Some(n1), Some(n2)) = (
                cardinal::kanji_to_number(&num1_kanji),
                cardinal::kanji_to_number(&num2_kanji),
            ) {
                result.push_str(&prefix);
                result.push_str(&n1.to_string());
                result.push('〜');
                result.push_str(&n2.to_string());
                remaining = &after_kara[num2_kanji.len()..];
                continue;
            }
        }

        // No match, pass through
        result.push_str(before_kara);
        result.push_str(kara);
        remaining = after_kara;
    }

    result.push_str(remaining);
    result
}

/// Process generic suffix: find kanji number before suffix and convert.
fn process_suffix(input: &str, suffix: &str) -> String {
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
        remaining = &remaining[pos + suffix.len()..];
    }

    result.push_str(remaining);
    result
}

/// Process 年 suffix, but avoid matching 年代 (already handled).
fn process_year(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find('年') {
        let after_year = &remaining[pos + '年'.len_utf8()..];

        // Skip if this is 年代 (already handled)
        if after_year.starts_with('代') {
            result.push_str(&remaining[..pos + '年'.len_utf8()]);
            remaining = after_year;
            continue;
        }

        let before = &remaining[..pos];
        let before_chars: Vec<char> = before.chars().collect();

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

        result.push('年');
        remaining = after_year;
    }

    result.push_str(remaining);
    result
}

/// Process 日 suffix, but avoid matching day-of-week abbreviations like (日).
fn process_day(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find('日') {
        // Check if this 日 is part of a day-of-week abbreviation (日)
        // or if it's preceded by ( — skip those
        let before = &remaining[..pos];
        if before.ends_with('(') || before.ends_with('（') {
            result.push_str(&remaining[..pos + '日'.len_utf8()]);
            remaining = &remaining[pos + '日'.len_utf8()..];
            continue;
        }

        let before_chars: Vec<char> = before.chars().collect();

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

        result.push('日');
        remaining = &remaining[pos + '日'.len_utf8()..];
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("一月"), "1月");
        assert_eq!(process("一月二十二日"), "1月22日");
    }

    #[test]
    fn test_weekday() {
        assert_eq!(process("三月一日水曜日"), "3月1日(水)");
    }

    #[test]
    fn test_range() {
        assert_eq!(process("五から九日"), "5〜9日");
        assert_eq!(process("七十から八十年代"), "70〜80年代");
    }

    #[test]
    fn test_century() {
        assert_eq!(process("二十一世紀"), "21世紀");
    }
}
