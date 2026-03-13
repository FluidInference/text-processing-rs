//! Ordinal number tagger for Chinese.
//!
//! Converts Chinese ordinals to Arabic numerals:
//! - "第一百" → "第100"
//! - "第兩萬一千一百一十一" → "第21111"
//!
//! Uses 第 prefix. Numbers after 第 that have only 万/億-scale and no sub-units
//! still preserve the scale char (e.g., "第两万" → "第2万").

use super::cardinal;

/// Process ordinal patterns in a string.
pub fn process(input: &str) -> String {
    let prefix = "第";
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(prefix) {
        result.push_str(&remaining[..pos]);
        result.push_str(prefix);

        let after = &remaining[pos + prefix.len()..];
        let chars: Vec<char> = after.chars().collect();

        // Find end of Chinese numeral span
        let mut num_end = 0;
        while num_end < chars.len() && cardinal::is_zh_numeral(chars[num_end]) {
            num_end += 1;
        }

        if num_end > 0 {
            let kanji: String = chars[..num_end].iter().collect();
            if let Some(formatted) = cardinal::format_zh_ordinal(&kanji) {
                result.push_str(&formatted);
            } else {
                result.push_str(&kanji);
            }
            let byte_len: usize = chars[..num_end].iter().map(|c| c.len_utf8()).sum();
            remaining = &after[byte_len..];
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
    fn test_basic() {
        assert_eq!(process("第一百"), "第100");
        assert_eq!(process("第五百"), "第500");
    }

    #[test]
    fn test_wan_preserved() {
        assert_eq!(process("第两万"), "第2万");
        assert_eq!(process("第十万"), "第10万");
    }

    #[test]
    fn test_expanded() {
        assert_eq!(process("第兩萬一千一百一十一"), "第21111");
    }
}
