//! Word TN tagger — last-resort spell-out for leftover symbol / alphanumeric
//! tokens NeMo's `word` class handles:
//! - standalone symbols: "$" → "dollar", "%" → "percent"
//! - letter+digit runs:  "es3" → "es three"
//! - mixed symbol runs:  "/$#" → "slash dollar hash"
//!
//! To avoid over-firing on natural language it only matches a single token
//! whose every non-alphanumeric character is a known symbol, and only when the
//! token carries a spellable symbol or is a letter+digit mix.

use super::spell_digits;

/// Spoken name for a standalone symbol.
fn symbol_word(c: char) -> Option<&'static str> {
    Some(match c {
        '$' => "dollar",
        '€' => "euro",
        '₩' => "won",
        '£' => "pound",
        '¥' => "yen",
        '#' => "hash",
        '%' => "percent",
        '/' => "slash",
        '&' => "and",
        '+' => "plus",
        '=' => "equals",
        '@' => "at",
        _ => return None,
    })
}

/// Parse a leftover word/symbol token to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let token = input.trim();
    if token.is_empty() || token.contains(char::is_whitespace) {
        return None;
    }
    // "and/or" is kept literal by NeMo's whitelist.
    if token.eq_ignore_ascii_case("and/or") {
        return None;
    }

    let mut out: Vec<String> = Vec::new();
    let mut has_symbol = false;
    let mut has_letter = false;
    let mut has_digit = false;
    let mut chars = token.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            has_letter = true;
            let mut run = String::new();
            while matches!(chars.peek(), Some(d) if d.is_ascii_alphabetic()) {
                run.push(chars.next().unwrap());
            }
            out.push(run);
        } else if c.is_ascii_digit() {
            has_digit = true;
            let mut run = String::new();
            while matches!(chars.peek(), Some(d) if d.is_ascii_digit()) {
                run.push(chars.next().unwrap());
            }
            out.push(spell_digits(&run));
        } else if let Some(word) = symbol_word(c) {
            has_symbol = true;
            out.push(word.to_string());
            chars.next();
        } else {
            // An unmappable symbol (hyphen, punctuation, …): leave the token to
            // other taggers rather than mangling it.
            return None;
        }
    }

    // Fire for a spellable symbol, or a letter+digit mixture that contains a
    // lower-case letter. The lower-case requirement leaves all-upper tokens
    // (ARPABET phonemes like "AH0", acronyms like "C24") to other taggers,
    // which NeMo's higher-priority classes handle.
    let has_lower = token.chars().any(|c| c.is_ascii_lowercase());
    if !(has_symbol || (has_letter && has_digit && has_lower)) {
        return None;
    }
    let result = out.join(" ");
    if result == token {
        return None;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_standalone_symbols() {
        assert_eq!(parse("$"), Some("dollar".to_string()));
        assert_eq!(parse("%"), Some("percent".to_string()));
    }

    #[test]
    fn test_letter_digit_mix() {
        assert_eq!(parse("es3"), Some("es three".to_string()));
    }

    #[test]
    fn test_symbol_run() {
        assert_eq!(
            parse("/$€₩£BB¥#%AA"),
            Some("slash dollar euro won pound BB yen hash percent AA".to_string())
        );
    }

    #[test]
    fn test_leaves_plain_and_hyphenated() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse("a-b"), None);
        assert_eq!(parse("covid-19"), None);
    }
}
