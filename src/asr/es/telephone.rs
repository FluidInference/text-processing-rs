//! Telephone tagger for Spanish.
//!
//! Converts spoken Spanish phone number to written form:
//! - "uno dos tres uno dos tres cinco seis siete ocho" → "123-123-5678"
//! - "más uno uno dos tres ..." → "+1-123-123-5678"
//! - "triple tres ..." → "333-..."

use super::cardinal;

/// Parse spoken Spanish phone number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Must have spaces (multiple words)
    if !input_trim.contains(' ') {
        return None;
    }

    let tokens: Vec<&str> = input_trim.split_whitespace().collect();

    // Extract extension if present
    let (main_tokens, extension) = extract_extension(&tokens);

    // Extract international prefix
    let (prefix, digit_tokens) = extract_prefix(main_tokens);

    // Convert tokens to digit groups
    let digits = tokens_to_digits(digit_tokens)?;

    if digits.is_empty() {
        return None;
    }

    // Format the number
    let formatted = format_phone_number(&digits)?;

    let mut result = String::new();
    if let Some(p) = prefix {
        result.push_str(&format!("+{}-", p));
    }
    result.push_str(&formatted);

    if let Some(ext) = extension {
        result.push_str(&format!(" ext. {}", ext));
    }

    Some(result)
}

/// Extract extension: "extensión doce" → (tokens, Some("12"))
fn extract_extension<'a>(tokens: &'a [&'a str]) -> (&'a [&'a str], Option<String>) {
    for (i, &t) in tokens.iter().enumerate() {
        if t == "extensión" {
            let ext_words = &tokens[i + 1..];
            let ext_str = ext_words.join(" ");
            if let Some(num) = cardinal::words_to_number(&ext_str) {
                return (&tokens[..i], Some(num.to_string()));
            }
        }
    }
    (tokens, None)
}

/// Extract international prefix: "más uno" → (Some("1"), rest)
/// Also handles multi-digit codes: "más cincuenta y cuatro" → (Some("54"), rest)
fn extract_prefix<'a>(tokens: &'a [&'a str]) -> (Option<String>, &'a [&'a str]) {
    if tokens.is_empty() {
        return (None, tokens);
    }

    if tokens[0] == "más" && tokens.len() > 1 {
        // Try single digit first: "más uno" → 1
        if let Some(d) = single_digit(tokens[1]) {
            return (Some(d.to_string()), &tokens[2..]);
        }

        // Try multi-word country code: "más cincuenta y cuatro" → 54
        // Try longest match first (up to 3 tokens), require the rest to start
        // with a parseable digit token to avoid consuming phone digits
        let remaining = &tokens[1..];
        let max_cc = 3.min(remaining.len());
        for end in (1..=max_cc).rev() {
            let candidate = remaining[..end].join(" ");
            if let Some(num) = cardinal::words_to_number(&candidate) {
                let num = num as i64;
                if num >= 10 && num <= 999 {
                    // Verify the next token after the country code is a digit
                    let after = &remaining[end..];
                    if !after.is_empty()
                        && (single_digit(after[0]).is_some()
                            || cardinal::words_to_number(after[0]).is_some()
                            || after[0] == "triple")
                    {
                        return (Some(num.to_string()), after);
                    }
                }
            }
        }
    }

    (None, tokens)
}

/// Convert word tokens to digit groups
fn tokens_to_digits(tokens: &[&str]) -> Option<Vec<u8>> {
    let mut digits = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let t = tokens[i];

        // Handle "triple X" → XXX
        if t == "triple" && i + 1 < tokens.len() {
            let next = tokens[i + 1];
            if let Some(d) = single_digit(next) {
                digits.push(d);
                digits.push(d);
                digits.push(d);
                i += 2;
                continue;
            }
        }

        // Try compound number (veintitrés → 23, cincuenta y seis → 56)
        // First try multi-word: "cincuenta y seis" (3 tokens)
        if i + 2 < tokens.len() && tokens[i + 1] == "y" {
            let compound = format!("{} y {}", t, tokens[i + 2]);
            if let Some(num) = cardinal::words_to_number(&compound) {
                let num = num as u64;
                if num >= 10 && num <= 99 {
                    digits.push((num / 10) as u8);
                    digits.push((num % 10) as u8);
                    i += 3;
                    continue;
                }
            }
        }

        // Single compound word (veintitrés → 23)
        if let Some(num) = cardinal::words_to_number(t) {
            let num = num as u64;
            if num >= 10 && num <= 99 {
                digits.push((num / 10) as u8);
                digits.push((num % 10) as u8);
            } else if num <= 9 {
                digits.push(num as u8);
            } else {
                return None;
            }
            i += 1;
            continue;
        }

        // Single digit word
        if let Some(d) = single_digit(t) {
            digits.push(d);
            i += 1;
            continue;
        }

        return None;
    }

    Some(digits)
}

/// Parse single digit word
fn single_digit(word: &str) -> Option<u8> {
    match word {
        "cero" => Some(0),
        "uno" | "un" | "una" => Some(1),
        "dos" => Some(2),
        "tres" => Some(3),
        "cuatro" => Some(4),
        "cinco" => Some(5),
        "seis" => Some(6),
        "siete" => Some(7),
        "ocho" => Some(8),
        "nueve" => Some(9),
        _ => None,
    }
}

/// Format phone digits into standard format
fn format_phone_number(digits: &[u8]) -> Option<String> {
    let s: String = digits.iter().map(|d| d.to_string()).collect();

    match digits.len() {
        10 => Some(format!("{}-{}-{}", &s[..3], &s[3..6], &s[6..10])),
        9 => Some(format!("{}-{}-{}", &s[..3], &s[3..6], &s[6..9])),
        8 => Some(format!("{}-{}", &s[..4], &s[4..8])),
        7 => Some(format!("{}-{}", &s[..3], &s[3..7])),
        _ => Some(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            parse("uno dos tres uno dos tres cinco seis siete ocho"),
            Some("123-123-5678".to_string())
        );
    }

    #[test]
    fn test_international() {
        assert_eq!(
            parse("más uno uno dos tres uno dos tres cinco seis siete ocho"),
            Some("+1-123-123-5678".to_string())
        );
    }

    #[test]
    fn test_triple() {
        assert_eq!(
            parse("triple tres uno dos tres cinco seis siete ocho"),
            Some("333-123-5678".to_string())
        );
    }
}
