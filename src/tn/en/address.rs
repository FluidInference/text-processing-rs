//! Address TN tagger — reads US street addresses NeMo's `address` class handles,
//! e.g. "1428 Elm St" → "fourteen twenty eight Elm Street" and
//! "708 N 1st St, San City" → "seven zero eight North first Street, San City".
//!
//! An address is a leading house number (read year-style, with "zero" for a
//! mid "oh"), an optional directional, street words, and a recognized street
//! suffix, followed optionally by comma-separated city, state, and ZIP.

use super::date::verbalize_year;
use super::{ordinal, spell_digits};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Street-type suffixes (matched case-insensitively).
    static ref SUFFIX: HashMap<&'static str, &'static str> = [
        ("st", "Street"), ("ave", "Avenue"), ("blvd", "Boulevard"), ("rd", "Road"),
        ("dr", "Drive"), ("ln", "Lane"), ("ct", "Court"), ("pl", "Place"),
        ("ter", "Terrace"), ("cir", "Circle"), ("way", "Way"), ("pkwy", "Parkway"),
        ("hwy", "Highway"), ("expy", "Expressway"), ("sq", "Square"), ("trl", "Trail"),
    ]
    .into_iter()
    .collect();

    /// Directional prefixes.
    static ref DIRECTIONAL: HashMap<&'static str, &'static str> = [
        ("N", "North"), ("S", "South"), ("E", "East"), ("W", "West"),
        ("NE", "Northeast"), ("NW", "Northwest"), ("SE", "Southeast"), ("SW", "Southwest"),
    ]
    .into_iter()
    .collect();

    /// US state postal codes.
    static ref STATE: HashMap<&'static str, &'static str> = [
        ("AL", "Alabama"), ("AK", "Alaska"), ("AZ", "Arizona"), ("AR", "Arkansas"),
        ("CA", "California"), ("CO", "Colorado"), ("CT", "Connecticut"), ("DE", "Delaware"),
        ("FL", "Florida"), ("GA", "Georgia"), ("HI", "Hawaii"), ("ID", "Idaho"),
        ("IL", "Illinois"), ("IN", "Indiana"), ("IA", "Iowa"), ("KS", "Kansas"),
        ("KY", "Kentucky"), ("LA", "Louisiana"), ("ME", "Maine"), ("MD", "Maryland"),
        ("MA", "Massachusetts"), ("MI", "Michigan"), ("MN", "Minnesota"), ("MS", "Mississippi"),
        ("MO", "Missouri"), ("MT", "Montana"), ("NE", "Nebraska"), ("NV", "Nevada"),
        ("NH", "New Hampshire"), ("NJ", "New Jersey"), ("NM", "New Mexico"), ("NY", "New York"),
        ("NC", "North Carolina"), ("ND", "North Dakota"), ("OH", "Ohio"), ("OK", "Oklahoma"),
        ("OR", "Oregon"), ("PA", "Pennsylvania"), ("RI", "Rhode Island"), ("SC", "South Carolina"),
        ("SD", "South Dakota"), ("TN", "Tennessee"), ("TX", "Texas"), ("UT", "Utah"),
        ("VT", "Vermont"), ("VA", "Virginia"), ("WA", "Washington"), ("WV", "West Virginia"),
        ("WI", "Wisconsin"), ("WY", "Wyoming"),
    ]
    .into_iter()
    .collect();
}

/// Parse a street address to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let span = input.trim();
    // Reject spans padded with trailing punctuation so a following "." is not
    // swallowed (the shorter, address-only span wins instead).
    if span
        .chars()
        .last()
        .is_some_and(|c| !c.is_ascii_alphanumeric())
    {
        return None;
    }

    // Separate commas into their own tokens.
    let spaced = span.replace(',', " , ");
    let tokens: Vec<&str> = spaced.split_whitespace().collect();
    if tokens.len() < 2 || !tokens[0].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let mut pieces: Vec<String> = vec![read_house_number(tokens[0])?];
    let mut i = 1;

    // Optional directional right after the house number.
    if let Some(&dir) = DIRECTIONAL.get(tokens[i]) {
        pieces.push(dir.to_string());
        i += 1;
    }

    // Street words up to and including a recognized suffix.
    let mut found_suffix = false;
    while i < tokens.len() {
        let token = tokens[i];
        if token == "," {
            break;
        }
        if let Some(&suffix) = SUFFIX.get(token.to_ascii_lowercase().as_str()) {
            pieces.push(suffix.to_string());
            i += 1;
            found_suffix = true;
            break;
        }
        if let Some(ord) = ordinal::parse(token) {
            pieces.push(ord);
        } else {
            pieces.push(token.to_string());
        }
        i += 1;
    }
    if !found_suffix {
        return None;
    }

    // After the suffix only comma-introduced city / state / ZIP may follow.
    if i < tokens.len() {
        if tokens[i] != "," {
            return None;
        }
        while i < tokens.len() {
            let token = tokens[i];
            if token == "," {
                pieces.push(",".to_string());
            } else if let Some(&state) = STATE.get(token) {
                pieces.push(state.to_string());
            } else if token.len() == 5 && token.chars().all(|c| c.is_ascii_digit()) {
                pieces.push(spell_digits(token));
            } else {
                pieces.push(token.to_string());
            }
            i += 1;
        }
    }

    Some(assemble(&pieces))
}

/// A house number reads year-style, but a middle "oh" is spoken "zero"
/// ("708" → "seven zero eight", "2788" → "twenty seven eighty eight").
fn read_house_number(s: &str) -> Option<String> {
    let n: u32 = s.parse().ok()?;
    let words = verbalize_year(n)?;
    Some(
        words
            .split(' ')
            .map(|w| if w == "oh" { "zero" } else { w })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Join pieces with spaces, attaching commas to the preceding word.
fn assemble(pieces: &[String]) -> String {
    let mut out = String::new();
    for piece in pieces {
        if piece == "," {
            out.push(',');
        } else {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(piece);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_basic() {
        assert_eq!(
            parse("1428 Elm St"),
            Some("fourteen twenty eight Elm Street".to_string())
        );
        assert_eq!(
            parse("1211 E Arques Ave"),
            Some("twelve eleven East Arques Avenue".to_string())
        );
        assert_eq!(
            parse("12 S 1st st"),
            Some("twelve South first Street".to_string())
        );
    }

    #[test]
    fn test_with_city_state_zip() {
        assert_eq!(
            parse("2788 San Tomas Expy, Santa Clara, CA 95051"),
            Some(
                "twenty seven eighty eight San Tomas Expressway, Santa Clara, California nine five zero five one"
                    .to_string()
            )
        );
        assert_eq!(
            parse("123 Smth St, City, NY"),
            Some("one twenty three Smth Street, City, New York".to_string())
        );
    }

    #[test]
    fn test_not_address() {
        assert_eq!(parse("Main St"), None); // no house number
        assert_eq!(parse("1428 Elm St."), None); // trailing period
        assert_eq!(parse("hello world"), None);
    }
}
