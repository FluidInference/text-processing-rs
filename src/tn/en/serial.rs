//! Serial TN tagger — reads alphanumeric serial codes NeMo's `serial` class
//! handles, e.g. part numbers and mixed letter/digit/symbol tokens:
//! - "B2A23C" → "B two A twenty three C"
//! - "133-ABC" → "one hundred thirty three-ABC"
//! - "$12@12%" → "dollar twelve at twelve percent"
//!
//! Letter runs are kept verbatim; digit runs read as cardinals (digit-by-digit
//! when they carry a leading zero or exceed four digits); `-` and `/` are kept
//! literally and glue their neighbours, while other symbols spell out as words.
//!
//! NeMo's serial grammar is internally inconsistent in places (a hyphen is kept
//! in "133-ABC" but spaced in "1-8090" and dropped in "7-eleven"); those
//! irregular forms are not reproduced.

use super::{number_to_words, spell_digits};

/// Spoken name for a spell-out symbol (glue symbols `-` and `/` are handled
/// separately and kept literal).
fn symbol_word(c: char) -> Option<&'static str> {
    Some(match c {
        '$' => "dollar",
        '€' => "euro",
        '£' => "pound",
        '¥' => "yen",
        '₩' => "won",
        '#' => "hash",
        '%' => "percent",
        '@' => "at",
        '*' => "asterisk",
        '+' => "plus",
        '&' => "and",
        '=' => "equals",
        _ => return None,
    })
}

/// True for characters a serial code may contain.
fn is_serial_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '/' || symbol_word(c).is_some()
}

/// Parse a serial code to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let token = input.trim();
    if token.is_empty() || !token.chars().all(is_serial_char) {
        return None;
    }
    // Require a digit so plain words, hyphenated words ("well-known"), and
    // pure-symbol tokens (left to the `word` tagger, which reads "/" as "slash")
    // are not swallowed.
    if !token.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    // Leave ARPABET phonemes ("AH0", "OW1") to be kept verbatim.
    if is_arpabet(token) {
        return None;
    }

    // Rate notation: "<digits>/<single letter>" → "<cardinal> per <LETTER>".
    if let Some((num, unit)) = token.split_once('/') {
        if !num.is_empty()
            && num.chars().all(|c| c.is_ascii_digit())
            && unit.len() == 1
            && unit.chars().all(|c| c.is_ascii_alphabetic())
        {
            return Some(format!(
                "{} per {}",
                number_to_words(num.parse().ok()?),
                unit.to_ascii_uppercase()
            ));
        }
    }

    let mut out = String::new();
    let mut chars = token.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == '-' || c == '/' {
            // Glue: keep literal, no surrounding spaces.
            out.push(c);
            chars.next();
            continue;
        }
        let piece = if c.is_ascii_alphabetic() {
            let mut run = String::new();
            while matches!(chars.peek(), Some(d) if d.is_ascii_alphabetic()) {
                run.push(chars.next().unwrap());
            }
            run
        } else if c.is_ascii_digit() {
            let mut run = String::new();
            while matches!(chars.peek(), Some(d) if d.is_ascii_digit()) {
                run.push(chars.next().unwrap());
            }
            read_digits(&run)?
        } else {
            let word = symbol_word(c)?.to_string();
            chars.next();
            word
        };
        if !out.is_empty() && !out.ends_with(['-', '/']) {
            out.push(' ');
        }
        out.push_str(&piece);
    }

    if out == token {
        return None;
    }
    Some(out)
}

/// An ARPABET phoneme token: upper-case letters followed by a single stress
/// digit (0/1/2), e.g. "AH0", "OW1".
fn is_arpabet(token: &str) -> bool {
    let bytes = token.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    let (letters, last) = bytes.split_at(bytes.len() - 1);
    letters.iter().all(|b| b.is_ascii_uppercase()) && matches!(last[0], b'0'..=b'2')
}

/// Read a digit run: digit-by-digit with a leading zero or beyond four digits,
/// otherwise a cardinal ("25" → "twenty five", "2000" → "two thousand").
fn read_digits(run: &str) -> Option<String> {
    if run.starts_with('0') || run.len() > 4 {
        Some(spell_digits(run))
    } else {
        Some(number_to_words(run.parse().ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_alphanumeric() {
        assert_eq!(parse("B2A23C"), Some("B two A twenty three C".to_string()));
        assert_eq!(parse("C24"), Some("C twenty four".to_string()));
        assert_eq!(
            parse("25d08A"),
            Some("twenty five d zero eight A".to_string())
        );
    }

    #[test]
    fn test_hyphen_kept() {
        assert_eq!(
            parse("133-ABC"),
            Some("one hundred thirty three-ABC".to_string())
        );
        assert_eq!(parse("covid-19"), Some("covid-nineteen".to_string()));
        assert_eq!(
            parse("t-0t25d12-f"),
            Some("t-zero t twenty five d twelve-f".to_string())
        );
    }

    #[test]
    fn test_symbols() {
        assert_eq!(
            parse("$12@12%"),
            Some("dollar twelve at twelve percent".to_string())
        );
        assert_eq!(parse("2*8"), Some("two asterisk eight".to_string()));
        // Pure-symbol tokens (no digit) are left to the `word` tagger.
        assert_eq!(parse("#mytext#"), None);
    }

    #[test]
    fn test_leaves_plain() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("well-known"), None);
        assert_eq!(parse("1.2.3"), None); // contains a dot
    }
}
