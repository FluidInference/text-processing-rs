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
//! Digit reading follows NeMo's two graphs: a run glued to letters
//! (num_graph_alnum) reads 1–2 digits / single-then-zeros as a cardinal and a
//! 3-digit-not-"00" or 4+ run digit-by-digit, while a pure delimiter-separated
//! group (num_graph_pure) reads up to four digits as a cardinal. A few cases
//! that route through other graphs (the "card ending in" cc-cue for "8876",
//! the range-style "1-8090", verbalizer casing for "4s") are not reproduced.

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

    // Render each delimiter-free segment, keeping "-" and "/" literal. A digit
    // run's reading depends on whether its segment mixes letters (NeMo's
    // num_graph_alnum) or is a pure numeric group (num_graph_pure).
    let mut out = String::new();
    let mut seg = String::new();
    for c in token.chars() {
        if c == '-' || c == '/' {
            render_segment(&seg, &mut out)?;
            seg.clear();
            out.push(c);
        } else {
            seg.push(c);
        }
    }
    render_segment(&seg, &mut out)?;

    if out == token {
        return None;
    }
    Some(out)
}

/// Render one delimiter-free segment, appending to `out` with the serial
/// spacing rules (space between runs, no space after a glue delimiter).
fn render_segment(seg: &str, out: &mut String) -> Option<()> {
    if seg.is_empty() {
        return Some(());
    }
    // A segment mixing letters and digits uses NeMo's alnum digit reading;
    // a pure numeric group uses the delimiter-group reading.
    let alnum = seg.chars().any(|c| c.is_ascii_alphabetic());
    let mut chars = seg.chars().peekable();
    while let Some(&c) = chars.peek() {
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
            if alnum {
                read_alnum_digits(&run)?
            } else {
                read_pure_digits(&run)?
            }
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
    Some(())
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

/// Read a digit run inside a pure numeric group (delimiter-separated):
/// digit-by-digit with a leading zero or beyond four digits, otherwise a
/// cardinal ("133" → "one hundred thirty three", "2021" → "two thousand
/// twenty one", "261788" → "two six one seven eight eight").
fn read_pure_digits(run: &str) -> Option<String> {
    if run.starts_with('0') || run.len() > 4 {
        Some(spell_digits(run))
    } else {
        Some(number_to_words(run.parse().ok()?))
    }
}

/// Read a digit run glued to letters (NeMo num_graph_alnum): a leading zero
/// reads digit-by-digit; 1–2 digits or a single non-zero digit followed only
/// by zeros read as a cardinal ("2000" → "two thousand"); a 3-digit run not
/// ending in "00", or 4+ digits, reads digit-by-digit ("9453" → "nine four
/// five three", "321" → "three two one").
fn read_alnum_digits(run: &str) -> Option<String> {
    if run.starts_with('0') {
        return Some(spell_digits(run));
    }
    if run.len() <= 2 {
        return Some(number_to_words(run.parse().ok()?));
    }
    let bytes = run.as_bytes();
    let single_then_zeros = bytes[0] != b'0' && bytes[1..].iter().all(|&b| b == b'0');
    if single_then_zeros {
        Some(number_to_words(run.parse().ok()?))
    } else {
        Some(spell_digits(run))
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
