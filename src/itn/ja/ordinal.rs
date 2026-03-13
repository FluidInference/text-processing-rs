//! Ordinal number tagger for Japanese.
//!
//! Converts kanji ordinals to Arabic numerals:
//! - "一番目" → "1番目"
//! - "第一" → "第1"

use super::cardinal;

/// Process ordinal patterns in a string.
/// Handles: X番目 → N番目, 第X → 第N
pub fn process(input: &str) -> String {
    let mut result = input.to_string();

    // Process 番目 patterns: find kanji numbers before 番目
    result = process_banme(&result);

    // Process 第 patterns: find kanji numbers after 第
    result = process_dai(&result);

    result
}

/// Replace kanji numbers before 番目 with Arabic numerals.
fn process_banme(input: &str) -> String {
    let suffix = "番目";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(suffix) {
        // Find the kanji number span ending just before 番目
        let before = &remaining[..pos];
        let chars: Vec<char> = before.chars().collect();

        // Scan backwards from end to find start of kanji number
        let mut num_start = chars.len();
        while num_start > 0 && cardinal::is_kanji_numeral(chars[num_start - 1]) {
            num_start -= 1;
        }

        if num_start < chars.len() {
            // Found kanji number before 番目
            let prefix: String = chars[..num_start].iter().collect();
            let kanji: String = chars[num_start..].iter().collect();
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

/// Replace kanji numbers after 第 with Arabic numerals.
fn process_dai(input: &str) -> String {
    let prefix = "第";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(prefix) {
        result.push_str(&remaining[..pos]);
        result.push_str(prefix);

        let after = &remaining[pos + prefix.len()..];
        let chars: Vec<char> = after.chars().collect();

        // Find end of kanji number span
        let mut num_end = 0;
        while num_end < chars.len() && cardinal::is_kanji_numeral(chars[num_end]) {
            num_end += 1;
        }

        if num_end > 0 {
            let kanji: String = chars[..num_end].iter().collect();
            if let Some(num) = cardinal::kanji_to_number(&kanji) {
                result.push_str(&num.to_string());
            } else {
                result.push_str(&kanji);
            }
            remaining = &after[kanji.len()..];
        } else {
            remaining = after;
        }
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banme() {
        assert_eq!(process("一番目"), "1番目");
        assert_eq!(process("三千三百三十番目"), "3330番目");
    }

    #[test]
    fn test_dai() {
        assert_eq!(process("第一"), "第1");
        assert_eq!(process("第七万二千六"), "第72006");
    }
}
