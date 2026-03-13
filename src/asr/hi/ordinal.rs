//! Ordinal number tagger for Hindi.
//!
//! Converts Hindi ordinal expressions to Devanagari form:
//! - "सौवां" → "१००वां"
//! - "दसवीं" → "१०वीं"
//! - "एक सौ उन्नीसवें" → "११९वें"

use super::cardinal;

/// Ordinal suffixes in Hindi: वां, वीं, वें
const ORDINAL_SUFFIXES: &[&str] = &["वीं", "वां", "वें"];

/// Process ordinal patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Look for a word ending with an ordinal suffix
        if let Some((suffix, base_end)) = find_ordinal_suffix(words[i]) {
            // Try to parse the base word (the part before the suffix)
            // First, try the current word alone
            let base_word = &words[i][..base_end];

            // Try building a multi-word number ending with this ordinal word
            let mut best_start = i;
            let mut best_val: Option<i64> = None;

            // Try spans ending at i
            let min_start = if i >= 10 { i - 10 } else { 0 };
            for start in min_start..=i {
                // All words from start to i-1 must be number words, plus the base of word[i]
                let mut num_words: Vec<&str> = Vec::new();
                let mut valid = true;

                for j in start..i {
                    if cardinal::is_hi_number_word(words[j]) || cardinal::is_modifier(words[j]) {
                        num_words.push(words[j]);
                    } else {
                        valid = false;
                        break;
                    }
                }

                if !valid {
                    continue;
                }

                // Add the base part of the ordinal word
                if !base_word.is_empty() {
                    num_words.push(base_word);
                }

                if num_words.is_empty() {
                    continue;
                }

                // Try to parse as a number
                // For ordinals, the last word might have the suffix stripped
                // We need to handle cases like "सौवां" where base="सौ"
                if let Some(val) = cardinal::words_to_number(&num_words) {
                    best_start = start;
                    best_val = Some(val);
                    break; // Take the longest span
                }
            }

            if let Some(val) = best_val {
                // Remove previously added words that are part of this number
                let to_remove = i - best_start;
                for _ in 0..to_remove {
                    result.pop();
                }
                result.push(format!("{}{}", cardinal::to_devanagari(val), suffix));
                i += 1;
                continue;
            }
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Find an ordinal suffix at the end of a word.
/// Returns (suffix, byte_position_where_suffix_starts) if found.
fn find_ordinal_suffix(word: &str) -> Option<(&'static str, usize)> {
    for &suffix in ORDINAL_SUFFIXES {
        if word.ends_with(suffix) {
            let base_end = word.len() - suffix.len();
            if base_end > 0 {
                return Some((suffix, base_end));
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
        assert_eq!(process("सौवां"), "१००वां");
        assert_eq!(process("दसवीं"), "१०वीं");
        assert_eq!(process("दसवें"), "१०वें");
    }

    #[test]
    fn test_compound() {
        assert_eq!(process("एक सौ उन्नीसवां"), "११९वां");
    }
}
