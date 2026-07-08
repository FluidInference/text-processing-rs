//! Math TN tagger.
//!
//! Converts simple written math expressions to spoken form:
//! - "1-2=5" → "one minus two equals five"
//! - "y=x +1" → "y equals x plus one"
//!
//! Requires an `=` so it never fires on bare hyphenated/serial tokens.

use super::number_to_words;

/// Parse a math expression to spoken form. Operands are integers or
/// single-word variables; operators are `+ - * / =`.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if !trimmed.contains('=') {
        return None;
    }

    // Space out the operators so a plain whitespace split yields tokens.
    let spaced = trimmed
        .replace('=', " = ")
        .replace('+', " + ")
        .replace('-', " - ")
        .replace('*', " * ")
        .replace('/', " / ");

    let mut words: Vec<String> = Vec::new();
    for token in spaced.split_whitespace() {
        let word = match token {
            "=" => "equals".to_string(),
            "+" => "plus".to_string(),
            "-" => "minus".to_string(),
            "*" => "times".to_string(),
            "/" => "divided by".to_string(),
            t if t.bytes().all(|b| b.is_ascii_digit()) => {
                let n: i64 = t.parse().ok()?;
                number_to_words(n)
            }
            // A bare variable (letters only) is kept verbatim.
            t if t.bytes().all(|b| b.is_ascii_alphabetic()) => t.to_string(),
            // Anything mixed (e.g. "x86", "5.4") is not a clean math token.
            _ => return None,
        };
        words.push(word);
    }

    if words.is_empty() {
        return None;
    }
    Some(words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            parse("1-2=5"),
            Some("one minus two equals five".to_string())
        );
        assert_eq!(
            parse("1- 2 = 5"),
            Some("one minus two equals five".to_string())
        );
    }

    #[test]
    fn test_variables() {
        assert_eq!(parse("y=x +1"), Some("y equals x plus one".to_string()));
        assert_eq!(parse("x +1 = y"), Some("x plus one equals y".to_string()));
    }

    #[test]
    fn test_requires_equals() {
        // No "=" → not this tagger's job (avoids ranges/serials).
        assert_eq!(parse("2-5"), None);
        assert_eq!(parse("hello"), None);
    }

    #[test]
    fn test_rejects_mixed_tokens() {
        assert_eq!(parse("x86 = y"), None);
    }
}
