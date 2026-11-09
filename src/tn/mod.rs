//! Text Normalization taggers.
//!
//! Converts written-form text to spoken form (the reverse of ITN):
//! - "200" → "two hundred" (English)
//! - "$5.50" → "five dollars and fifty cents" (English)
//! - "January 5, 2025" → "january fifth twenty twenty five" (English)
//!
//! Supports multiple languages via submodules.

// Languages
pub mod de;
pub mod en;
pub mod es;
pub mod fr;
pub mod hi;
pub mod ja;
pub mod zh;

/// Whether `s` is one written number: ASCII digits with `.`/`,` as decimal or
/// thousands separators (a bare fraction like ".5" too), and spaces only before
/// three-digit thousands groups ("1 000,50"). Taggers that anchor on a trailing
/// unit or currency ("12,50 €", "200 m") check the text before it with this, so
/// a sentence-mode span cannot fold the words or numbers in front into it.
pub(crate) fn is_written_number(s: &str) -> bool {
    let is_separator = |c: char| c == '.' || c == ',';
    // Digits with single separators between them; a bare fraction (".5") may
    // lead with one.
    let is_number_group = |g: &str| {
        let body = g.strip_prefix(is_separator).unwrap_or(g);
        body.starts_with(|c: char| c.is_ascii_digit())
            && body.ends_with(|c: char| c.is_ascii_digit())
            && body.chars().all(|c| c.is_ascii_digit() || is_separator(c))
            && !body
                .as_bytes()
                .windows(2)
                .any(|w| is_separator(w[0] as char) && is_separator(w[1] as char))
    };

    let mut groups = s.split([' ', '\u{a0}', '\u{202f}']);
    let Some(first) = groups.next() else {
        return false;
    };
    if !is_number_group(first) {
        return false;
    }
    // Space-separated groups continue the thousands: three digits, the last
    // optionally followed by a decimal part.
    groups.all(|g| {
        is_number_group(g)
            && g.len() >= 3
            && g[..3].bytes().all(|b| b.is_ascii_digit())
            && (g.len() == 3 || is_separator(g.as_bytes()[3] as char))
    })
}

#[cfg(test)]
mod tests {
    use super::is_written_number;

    #[test]
    fn written_numbers() {
        for s in [
            "5",
            "12,50",
            "1.234,56",
            "1,234.56",
            "1 000",
            "1 234 567,89",
            "0.01",
            ".5",
            ",5",
        ] {
            assert!(is_written_number(s), "{s:?}");
        }
        for s in [
            "",
            "Es kostet 12,50",
            "3.10.2025 12,50",
            "2 12,50",
            "5.",
            "..5",
            "1 .5",
            "1  000",
            "1 0000",
            "12..5",
            "200 m",
        ] {
            assert!(!is_written_number(s), "{s:?}");
        }
    }
}
