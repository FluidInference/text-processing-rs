//! Address tagger for Hindi.
//!
//! After cardinal processing, digit words have been converted to Devanagari digits.
//! This module concatenates sequences of Devanagari digits in address contexts:
//! - "७ ० ०" → "७००"
//! - "६ ६ - ४," → "६६-४,"
//! - "१ ४ / ३," → "१४/३,"
//!
//! Also handles comma-separated digit sequences and
//! हाइफ़न/बटा between digit groups.

/// Check if a string is a Devanagari digit sequence (one or more digits).
fn is_devanagari_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('०'..='९').contains(&c))
}

/// Check if a string is a Devanagari digit with trailing comma (like "०,").
fn strip_trailing_comma(s: &str) -> Option<&str> {
    if s.ends_with(',') {
        let core = &s[..s.len() - 1];
        if is_devanagari_number(core) {
            return Some(core);
        }
    }
    None
}

/// Process address patterns in a string.
/// At this point, cardinal has already converted number words to Devanagari digits.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Check for Devanagari digit sequences that should be concatenated
        if is_devanagari_number(words[i]) || strip_trailing_comma(words[i]).is_some() {
            let mut digits = String::new();
            let mut trailing_comma = false;

            while i < words.len() {
                if is_devanagari_number(words[i]) {
                    digits.push_str(words[i]);
                    i += 1;
                } else if let Some(core) = strip_trailing_comma(words[i]) {
                    // Digit with trailing comma — add digit, mark comma, stop sequence
                    digits.push_str(core);
                    trailing_comma = true;
                    i += 1;
                    break;
                } else if words[i] == "हाइफ़न" || words[i] == "हाइफन" || words[i] == "-" {
                    // Hyphen separator
                    if i + 1 < words.len() && (is_devanagari_number(words[i + 1]) || strip_trailing_comma(words[i + 1]).is_some()) {
                        digits.push('-');
                        i += 1;
                    } else {
                        break;
                    }
                } else if words[i] == "बटा" || words[i] == "/" {
                    // Slash separator (address fraction)
                    if i + 1 < words.len() && (is_devanagari_number(words[i + 1]) || strip_trailing_comma(words[i + 1]).is_some()) {
                        digits.push('/');
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if !digits.is_empty() {
                if trailing_comma {
                    result.push(format!("{},", digits));
                } else {
                    result.push(digits);
                }
                continue;
            }
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("७ ० ० ओक स्ट्रीट"), "७०० ओक स्ट्रीट");
    }

    #[test]
    fn test_hyphen() {
        assert_eq!(
            process("६ ६ हाइफ़न ४, पार्कहर्स्ट रोड"),
            "६६-४, पार्कहर्स्ट रोड"
        );
    }

    #[test]
    fn test_slash() {
        assert_eq!(
            process("१ ४ बटा ३, मथुरा रोड"),
            "१४/३, मथुरा रोड"
        );
    }

    #[test]
    fn test_comma_separated() {
        assert_eq!(
            process("बूथ ७०, सेक्टर ८, चंडीगढ़"),
            "बूथ ७०, सेक्टर ८, चंडीगढ़"
        );
    }
}
