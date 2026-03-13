//! Whitelist tagger for Hindi.
//!
//! Maps specific Hindi phrases to their abbreviated forms:
//! - "मास्टर निखिल तनिष" → "मा. निखिल तनिष"
//! - "श्रीमती ज्योत्सना" → "स्मि. ज्योत्सना"
//! - "डॉक्टर" → "डॉ."
//! - "पाव" → "१/४"
//! - "आधा कप चाय" → "१/२ कप चाय"

/// Whitelist entries: (input phrase, output)
/// Sorted longest first to avoid partial matches.
const WHITELIST: &[(&str, &str)] = &[
    ("श्रीमान", "श्री."),
    ("श्रीमती", "स्मि."),
    ("मास्टर", "मा."),
    ("डॉक्टर", "डॉ."),
    ("कुमारी", "कु."),
    ("पाव", "१/४"),
    ("आधा", "१/२"),
];

/// Process whitelist patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        let mut matched = false;

        for &(term, replacement) in WHITELIST {
            let term_words: Vec<&str> = term.split_whitespace().collect();
            let term_len = term_words.len();

            if i + term_len <= words.len() {
                let matches = term_words.iter().enumerate().all(|(j, &tw)| words[i + j] == tw);
                if matches {
                    result.push(replacement.to_string());
                    i += term_len;
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            result.push(words[i].to_string());
            i += 1;
        }
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("डॉक्टर"), "डॉ.");
        assert_eq!(process("कुमारी"), "कु.");
    }

    #[test]
    fn test_with_name() {
        assert_eq!(process("डॉक्टर प्रशांत"), "डॉ. प्रशांत");
    }
}
