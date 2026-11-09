//! Roman-numeral TN tagger.
//!
//! Converts a roman numeral that follows a section keyword to a cardinal:
//! - "Chapter IV" → "Chapter four"
//! - "PART XL" → "PART forty"
//!
//! A capitalized name followed by an upper-case numeral reads as an ordinal
//! ("Sam II" → "Sam second"). Numerals must be in canonical form, so ordinary
//! words made of roman letters stay words ("The Mouse did not answer", #101).

use super::number_to_words;

/// Section words that mark a following roman numeral as a cardinal. Matched
/// case-insensitively; the prefix is echoed back with its original casing.
const KEYWORDS: &[&str] = &[
    "chapter",
    "class",
    "part",
    "article",
    "section",
    "paragraph",
];

/// Parse a `"<keyword> <roman>"` pair to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let (prefix, roman) = trimmed.split_once(' ')?;
    let roman = roman.trim();
    let upper = roman.chars().all(|c| c.is_ascii_uppercase());

    // Section keyword → cardinal ("Chapter IV" → "Chapter four"). A lower-case
    // numeral must be a strict `I V X` one, so "part mix" stays words.
    if KEYWORDS.contains(&prefix.to_lowercase().as_str()) {
        let value = if upper {
            canonical_value(roman)?
        } else if roman.chars().all(|c| c.is_ascii_lowercase()) {
            strict_value(roman)?
        } else {
            return None;
        };
        return Some(format!("{} {}", prefix, number_to_words(value)));
    }

    // A capitalized name followed by a multi-character upper-case numeral →
    // ordinal ("Sam II" → "Sam second"). Requiring two or more numeral
    // characters avoids grabbing a stray "I"/"V" after an ordinary capitalized
    // word; requiring upper case and canonical form keeps "It did" a sentence.
    if roman.len() >= 2 && upper && is_name(prefix) {
        let value = canonical_value(roman)?;
        return Some(format!(
            "{} {}",
            prefix,
            super::ordinal::number_to_ordinal_words(value)
        ));
    }

    None
}

/// A plausible personal name: an initial capital followed by lower-case
/// letters ("Sam", "Henry").
fn is_name(word: &str) -> bool {
    let mut chars = word.chars();
    word.len() >= 2
        && matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_lowercase())
}

/// Rewrite roman-numeral list markers — `(ii)`, `ii)`, `ii.` — to spoken
/// cardinals, leaving the surrounding punctuation in place (`(two)`). Port of
/// FluidAudio's `EnglishTextNormalizer` pre-pass (FluidAudio #972); opt-in via
/// [`crate::NormalizeOptions::roman_enumerators`] because NeMo leaves these
/// markers untouched.
///
/// Keyed on the enumerator *form*, never on the letters alone (`mix`, `did`,
/// `civil` and the pronoun `I` are all roman letters):
/// - `(ii)` may appear anywhere, as long as the `(` is not glued to a letter
///   (`f(x)` stays) and the `)` is not glued to a letter or digit.
/// - `ii)` / `ii.` only in enumerator position — line start (indentation
///   allowed) or after `; : , .` + whitespace — and the dot form only when
///   followed by whitespace (`i.e.` stays).
/// - Numerals use `I V X` only, in a single case, strict subtractive form,
///   1–39. Admitting `L C D M` would put `mix`, `cd`, `mm`, `xl` in the trap set.
/// - A marker that is not self-evidently a list item — uppercase, a lone
///   `v`/`x`, or all-`x` (`xx`) — needs a second enumerator somewhere in the
///   text: medical `(IV)`, a checkbox `(x)`, a sign-off `xx.`, the citation
///   `v. Madison` and initials `I. M. Pei` stand alone, while an uppercase
///   outline `I. … II. … III.` converts as a whole.
pub fn spell_enumerators(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let found = find_enumerators(&chars);
    if found.is_empty() {
        return text.to_string();
    }
    let is_list = found.len() >= 2;

    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for e in &found {
        if !(is_list || is_self_evident(&e.numeral)) {
            continue;
        }
        out.extend(&chars[last..e.start]);
        out.push_str(&number_to_words(e.value));
        last = e.end;
    }
    out.extend(&chars[last..]);
    out
}

/// A well-formed roman numeral in an enumerator form. `start..end` covers the
/// numeral only; the surrounding punctuation is re-emitted verbatim.
struct Enumerator {
    start: usize,
    end: usize,
    numeral: String,
    value: i64,
}

fn find_enumerators(chars: &[char]) -> Vec<Enumerator> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_roman_letter(chars[i]) {
            i += 1;
            continue;
        }
        // Maximal run of roman letters starting here.
        let mut end = i;
        while end < chars.len() && is_roman_letter(chars[end]) {
            end += 1;
        }
        let numeral: String = chars[i..end].iter().collect();
        let uniform_case = numeral.chars().all(|c| c.is_ascii_lowercase())
            || numeral.chars().all(|c| c.is_ascii_uppercase());
        if numeral.len() <= 7 && uniform_case && in_enumerator_form(chars, i, end) {
            if let Some(value) = strict_value(&numeral) {
                found.push(Enumerator {
                    start: i,
                    end,
                    numeral,
                    value,
                });
            }
        }
        i = end;
    }
    found
}

fn is_roman_letter(c: char) -> bool {
    matches!(c, 'i' | 'v' | 'x' | 'I' | 'V' | 'X')
}

/// `(ii)`, or `ii)` / `ii.` in enumerator position.
fn in_enumerator_form(chars: &[char], start: usize, end: usize) -> bool {
    let at = |idx: usize| chars.get(idx).copied();
    let before = start.checked_sub(1).and_then(at);
    let after = at(end);

    if before == Some('(') && after == Some(')') {
        let glued_left = start
            .checked_sub(2)
            .and_then(at)
            .is_some_and(|c| c.is_alphabetic());
        let glued_right = at(end + 1).is_some_and(|c| c.is_alphanumeric());
        return !glued_left && !glued_right;
    }

    let closer_ok = match after {
        Some(')') => !at(end + 1).is_some_and(|c| c.is_alphanumeric()),
        Some('.') => at(end + 1).is_some_and(char::is_whitespace),
        _ => false,
    };
    closer_ok && in_enumerator_position(chars, start)
}

/// Line start (after optional indentation) or clause punctuation + whitespace.
fn in_enumerator_position(chars: &[char], start: usize) -> bool {
    let mut j = start;
    while j > 0 && matches!(chars[j - 1], ' ' | '\t') {
        j -= 1;
    }
    if j == 0 || matches!(chars[j - 1], '\n' | '\r') {
        return true;
    }
    start >= 2
        && chars[start - 1].is_whitespace()
        && matches!(chars[start - 2], ';' | ':' | ',' | '.')
}

/// Lowercase `i`, or a lowercase multi-letter numeral that is not all `x`:
/// nothing else reads as a word or an abbreviation in enumerator position.
fn is_self_evident(numeral: &str) -> bool {
    if numeral.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }
    numeral == "i" || (numeral.len() >= 2 && numeral.chars().any(|c| c != 'x'))
}

/// Value of a strict-form `I V X` numeral (1–39), else `None`: tens `X{0,3}`,
/// then units `IX | IV | V?I{0,3}`.
fn strict_value(numeral: &str) -> Option<i64> {
    let upper = numeral.to_ascii_uppercase();
    let tens = upper.chars().take_while(|&c| c == 'X').count();
    if tens > 3 {
        return None;
    }
    let units = match &upper[tens..] {
        "" => 0,
        "I" => 1,
        "II" => 2,
        "III" => 3,
        "IV" => 4,
        "V" => 5,
        "VI" => 6,
        "VII" => 7,
        "VIII" => 8,
        "IX" => 9,
        _ => return None,
    };
    let value = 10 * tens as i64 + units;
    (value > 0).then_some(value)
}

/// Value of a numeral written in canonical form ("XIV", "MCMXCIV"), compared
/// case-insensitively, else `None`. Rejects letter runs that only parse
/// additively, like "DID" (999 is "CMXCIX") or "MIM".
fn canonical_value(numeral: &str) -> Option<i64> {
    let value = roman_to_int(numeral)?;
    (to_roman(value)? == numeral.to_ascii_uppercase()).then_some(value)
}

/// Canonical roman form of 1–3999.
fn to_roman(mut value: i64) -> Option<String> {
    if !(1..=3999).contains(&value) {
        return None;
    }
    const PARTS: [(i64, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (n, s) in PARTS {
        while value >= n {
            out.push_str(s);
            value -= n;
        }
    }
    Some(out)
}

/// Convert a roman numeral to its value, or `None` if it is not a valid
/// numeral (empty, or containing a non-roman letter).
fn roman_to_int(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut total = 0i64;
    let mut prev = 0i64;
    for c in s.chars().rev() {
        let value = match c.to_ascii_uppercase() {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if value < prev {
            total -= value;
        } else {
            total += value;
            prev = value;
        }
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_cardinal() {
        assert_eq!(parse("Chapter IV"), Some("Chapter four".to_string()));
        assert_eq!(parse("PART XL"), Some("PART forty".to_string()));
        assert_eq!(parse("section iii"), Some("section three".to_string()));
        assert_eq!(parse("Article XII"), Some("Article twelve".to_string()));
    }

    #[test]
    fn test_not_roman() {
        assert_eq!(parse("Chapter Five"), None); // not a numeral
        assert_eq!(parse("Sam II"), Some("Sam second".to_string())); // name → ordinal
                                                                     // #101: ordinary words made of roman letters
        assert_eq!(parse("Mouse did"), None);
        assert_eq!(parse("It did"), None);
        assert_eq!(parse("Sam DID"), None); // not canonical (999 is CMXCIX)
        assert_eq!(parse("part mix"), None);
        assert_eq!(parse("Louis XVI"), Some("Louis sixteenth".to_string()));
        assert_eq!(
            parse("Chapter MCMXCIV"),
            Some("Chapter one thousand nine hundred ninety four".to_string())
        );
        assert_eq!(parse("hello world"), None);
        assert_eq!(parse("Chapter"), None);
    }

    #[test]
    fn test_roman_values() {
        assert_eq!(roman_to_int("IV"), Some(4));
        assert_eq!(roman_to_int("XL"), Some(40));
        assert_eq!(roman_to_int("MCMXciv"), Some(1994));
        assert_eq!(roman_to_int("IIII"), Some(4)); // lax: additive form allowed
        assert_eq!(roman_to_int("hi"), None);
    }

    // spell_enumerators — mirrors FluidAudio's EnglishTextNormalizerTests (#972).

    #[test]
    fn enumerators_parenthesized() {
        assert_eq!(
            spell_enumerators("(i) pay rent; (ii) keep the peace; (iii) insure; (iv) vacate."),
            "(one) pay rent; (two) keep the peace; (three) insure; (four) vacate."
        );
        assert_eq!(
            spell_enumerators("(ix) and (xiv) and (xxxix)"),
            "(nine) and (fourteen) and (thirty nine)"
        );
        assert_eq!(
            spell_enumerators("see (IV) and (XII)"),
            "see (four) and (twelve)"
        );
        assert_eq!(
            spell_enumerators("under 2(a)(ii) above"),
            "under 2(a)(two) above"
        );
    }

    #[test]
    fn enumerators_half_paren_and_dot() {
        assert_eq!(
            spell_enumerators("as follows: i) rent; ii) noise; iii) pets"),
            "as follows: one) rent; two) noise; three) pets"
        );
        assert_eq!(
            spell_enumerators("i) first\nii) second\n  iv) fourth"),
            "one) first\ntwo) second\n  four) fourth"
        );
        assert_eq!(
            spell_enumerators("I) first; II) second"),
            "one) first; two) second"
        );
        assert_eq!(
            spell_enumerators("i. Introduction\nii. Methods\niv. Results"),
            "one. Introduction\ntwo. Methods\nfour. Results"
        );
        assert_eq!(
            spell_enumerators("I. Intro\nII. Body\nIII. End"),
            "one. Intro\ntwo. Body\nthree. End"
        );
    }

    #[test]
    fn enumerators_single_lowercase_marker_is_enough() {
        assert_eq!(spell_enumerators("(i) pay rent"), "(one) pay rent");
        assert_eq!(
            spell_enumerators("(iv) vacate on notice"),
            "(four) vacate on notice"
        );
        assert_eq!(spell_enumerators("vi. Appendix"), "six. Appendix");
    }

    #[test]
    fn enumerators_ambiguous_markers_need_list_context() {
        for s in [
            "morphine (IV) fluids",
            "Mark (x) here",
            "(v) to run",
            "(I) think",
            "Thanks. xx. Jane",
            "Marbury\nv. Madison",
            "Solve for the variable, x. Then",
        ] {
            assert_eq!(spell_enumerators(s), s, "{s:?} should be unchanged");
        }
        assert_eq!(
            spell_enumerators("(IV) fluids; (V) rest"),
            "(four) fluids; (five) rest"
        );
        assert_eq!(
            spell_enumerators("(ix) foo; (x) bar"),
            "(nine) foo; (ten) bar"
        );
    }

    #[test]
    fn enumerators_prose_and_non_forms_unchanged() {
        for s in [
            "mix it, did I? civil and mild",
            "(and so did I)",
            "I. M. Pei designed it",
            "I use vi. It rocks",
            "i.e. the rest",
            "the variable x) is free",
            "f(x) and g(i)",
            "café(i) test",
            "(xl) size",
            "(mix) (cd) (mm)",
            "(iiii) (vv) (Iv) (ivi)",
            "",
        ] {
            assert_eq!(spell_enumerators(s), s, "{s:?} should be unchanged");
        }
    }

    #[test]
    fn strict_values() {
        assert_eq!(strict_value("i"), Some(1));
        assert_eq!(strict_value("XXXIX"), Some(39));
        assert_eq!(strict_value("iv"), Some(4));
        assert_eq!(strict_value("IIII"), None);
        assert_eq!(strict_value("XXXX"), None);
        assert_eq!(strict_value("VX"), None);
        assert_eq!(strict_value(""), None);
    }
}
