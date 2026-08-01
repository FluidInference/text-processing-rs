//! Time tagger for Korean.
//!
//! Converts spoken Korean time expressions to written form:
//! - "세시" → "3시"
//! - "세시 삼십분" → "3시 30분"
//! - "열두시 십오분 삼십초" → "12시 15분 30초"
//! - "영시" → "0시"
//!
//! Korean reads the *hour* with **native** numerals (한, 두, 세 …) and
//! the *minute* / *second* with **Sino-Korean** numerals (일, 이, 삼 …),
//! so the hour tagger uses a dedicated native lookup table while
//! minutes and seconds reuse the Sino-Korean cardinal parser.

use super::cardinal;

/// Native-Korean hour words → value. Sorted longest-first at use sites
/// so `열한` / `열두` outrank `열`, and `한` / `두` lose to them.
const NATIVE_HOURS: &[(&str, i64)] = &[
    ("열한", 11),
    ("열두", 12),
    ("열", 10),
    ("한", 1),
    ("두", 2),
    ("세", 3),
    ("네", 4),
    ("다섯", 5),
    ("여섯", 6),
    ("일곱", 7),
    ("여덟", 8),
    ("아홉", 9),
    ("영", 0),
    ("공", 0),
];

/// Process time patterns in a string.
pub fn process(input: &str) -> String {
    let result = process_hour(input);
    let result = process_unit(&result, "분", true);
    process_unit(&result, "초", false)
}

/// Process 시 suffix. Hours use native Korean numerals; a Sino-Korean
/// run is also accepted as a fallback (some ASR output mixes them).
fn process_hour(input: &str) -> String {
    let suffix = "시";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let before = &remaining[..pos];

        // Longest native hour word that `before` ends with.
        let native = NATIVE_HOURS
            .iter()
            .filter(|(word, _)| before.ends_with(word))
            .max_by_key(|(word, _)| word.len());

        if let Some((word, value)) = native {
            let prefix = &before[..before.len() - word.len()];
            result.push_str(prefix);
            result.push_str(&value.to_string());
        } else {
            // Fallback: Sino-Korean run before 시.
            let chars: Vec<char> = before.chars().collect();
            let mut num_start = chars.len();
            while num_start > 0 && cardinal::is_sino_korean_numeral(chars[num_start - 1]) {
                num_start -= 1;
            }
            if num_start < chars.len() {
                let span: String = chars[num_start..].iter().collect();
                let prefix: String = chars[..num_start].iter().collect();
                result.push_str(&prefix);
                match cardinal::sino_korean_to_number(&span) {
                    Some(num) => result.push_str(&num.to_string()),
                    None => result.push_str(&span),
                }
            } else {
                result.push_str(before);
            }
        }

        result.push_str(suffix);
        remaining = &remaining[pos + suffix.len()..];
    }

    result.push_str(remaining);
    result
}

/// Process a Sino-Korean-numbered unit suffix (분 / 초).
///
/// When `skip_fraction` is set, a 분 immediately followed by 의 is left
/// alone — that is the fraction pattern (분의), handled elsewhere.
fn process_unit(input: &str, suffix: &str, skip_fraction: bool) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        let after = &remaining[pos + suffix.len()..];
        if skip_fraction && after.starts_with('의') {
            result.push_str(&remaining[..pos + suffix.len()]);
            remaining = after;
            continue;
        }

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
            match cardinal::sino_korean_to_number(&span) {
                Some(num) => result.push_str(&num.to_string()),
                None => result.push_str(&span),
            }
        } else {
            result.push_str(before);
        }

        result.push_str(suffix);
        remaining = after;
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hour() {
        assert_eq!(process("세시"), "3시");
        assert_eq!(process("열두시"), "12시");
        assert_eq!(process("영시"), "0시");
        assert_eq!(process("열한시"), "11시");
    }

    #[test]
    fn test_hour_minute_second() {
        assert_eq!(process("세시 삼십분"), "3시 30분");
        assert_eq!(process("열두시 십오분 삼십초"), "12시 15분 30초");
    }

    #[test]
    fn test_skip_fraction() {
        // 분의 is a fraction pattern, not minutes.
        assert_eq!(process("삼분의 일"), "삼분의 일");
    }

    #[test]
    fn test_contextual() {
        assert_eq!(
            process("회의는 두시 십분에 시작한다"),
            "회의는 2시 10분에 시작한다"
        );
    }
}
