//! Date tagger for Korean.
//!
//! Converts Sino-Korean dates to Arabic numeral form:
//! - "일월이십이일" → "1월22일"
//! - "구십년대" → "90년대"
//! - "이천이십육년" → "2026년"
//! - "삼월일일월요일" → "3월1일(월)"
//!
//! Korean irregular month names 유월 (June) and 시월 (October) — which
//! drop the expected 육/십 — are handled explicitly.
//!
//! Date *ranges* ("3일에서 5일") are intentionally not handled yet: the
//! Korean range connectors (에서 / 부터) are extremely common
//! non-range particles, and the 일 day-marker is itself a numeral, so a
//! reliable range tagger needs more context than this pass provides.

use super::cardinal;

/// Day-of-week mappings: full form → abbreviated form.
const WEEKDAYS: &[(&str, &str)] = &[
    ("월요일", "(월)"),
    ("화요일", "(화)"),
    ("수요일", "(수)"),
    ("목요일", "(목)"),
    ("금요일", "(금)"),
    ("토요일", "(토)"),
    ("일요일", "(일)"),
];

/// Irregular month names that do not follow the digit + 월 pattern.
const IRREGULAR_MONTHS: &[(&str, &str)] = &[("유월", "6월"), ("시월", "10월")];

/// Process date patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = input.to_string();

    // Weekdays first so their 월 / 일 are not re-processed.
    for &(full, abbr) in WEEKDAYS {
        result = result.replace(full, abbr);
    }

    // Irregular month names before the generic 월 pass.
    for &(full, replacement) in IRREGULAR_MONTHS {
        result = result.replace(full, replacement);
    }

    result = process_suffix(&result, "세기");
    result = process_suffix(&result, "년대");
    result = process_year(&result);
    result = process_suffix(&result, "월");
    result = process_day(&result);

    result
}

/// Generic suffix: convert the Sino-Korean number immediately before `suffix`.
fn process_suffix(input: &str, suffix: &str) -> String {
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
            let prefix: String = chars[..num_start].iter().collect();
            let span: String = chars[num_start..].iter().collect();
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

/// Process 년 suffix, skipping 년대 (already handled).
fn process_year(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find('년') {
        let after_year = &remaining[pos + '년'.len_utf8()..];
        if after_year.starts_with('대') {
            result.push_str(&remaining[..pos + '년'.len_utf8()]);
            remaining = after_year;
            continue;
        }

        let before = &remaining[..pos];
        let chars: Vec<char> = before.chars().collect();
        let mut num_start = chars.len();
        while num_start > 0 && cardinal::is_sino_korean_numeral(chars[num_start - 1]) {
            num_start -= 1;
        }

        if num_start < chars.len() {
            let prefix: String = chars[..num_start].iter().collect();
            let span: String = chars[num_start..].iter().collect();
            result.push_str(&prefix);
            if let Some(num) = cardinal::sino_korean_to_number(&span) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&span);
            }
        } else {
            result.push_str(before);
        }

        result.push('년');
        remaining = after_year;
    }

    result.push_str(remaining);
    result
}

/// Process 일 suffix, skipping the day-of-week abbreviation form `(일)`.
///
/// 일 is both the Sino-Korean digit 1 and the day marker, so in a run
/// of consecutive 일 (e.g. 일일 = "1일") the day marker is the *last*
/// one — the earlier 일 belong to the numeral. The inner loop advances
/// to that final 일 before scanning the numeral span behind it.
fn process_day(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;
    let il_len = '일'.len_utf8();

    while let Some(found) = remaining.find('일') {
        let mut pos = found;
        while remaining[pos + il_len..].starts_with('일') {
            pos += il_len;
        }

        let before = &remaining[..pos];
        if before.ends_with('(') {
            result.push_str(&remaining[..pos + il_len]);
            remaining = &remaining[pos + il_len..];
            continue;
        }

        let chars: Vec<char> = before.chars().collect();
        let mut num_start = chars.len();
        while num_start > 0 && cardinal::is_sino_korean_numeral(chars[num_start - 1]) {
            num_start -= 1;
        }

        if num_start < chars.len() {
            let prefix: String = chars[..num_start].iter().collect();
            let span: String = chars[num_start..].iter().collect();
            result.push_str(&prefix);
            if let Some(num) = cardinal::sino_korean_to_number(&span) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&span);
            }
        } else {
            result.push_str(before);
        }

        result.push('일');
        remaining = &remaining[pos + il_len..];
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("일월"), "1월");
        assert_eq!(process("일월이십이일"), "1월22일");
        assert_eq!(process("이천이십육년"), "2026년");
    }

    #[test]
    fn test_irregular_months() {
        assert_eq!(process("유월"), "6월");
        assert_eq!(process("시월십오일"), "10월15일");
    }

    #[test]
    fn test_weekday() {
        assert_eq!(process("삼월일일월요일"), "3월1일(월)");
        assert_eq!(process("사월삼십일일요일"), "4월30일(일)");
    }

    #[test]
    fn test_decade_and_century() {
        assert_eq!(process("구십년대"), "90년대");
        assert_eq!(process("이십일세기"), "21세기");
    }
}
