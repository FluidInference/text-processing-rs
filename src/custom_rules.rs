//! Custom user-defined normalization rules.
//!
//! Allows callers to register spoken→written mappings at runtime.
//! These rules are checked with the highest priority in sentence mode,
//! before any built-in taggers.
//!
//! Example: ("linux", "Linux"), ("gee pee tee", "GPT")

use std::sync::RwLock;

use lazy_static::lazy_static;

lazy_static! {
    /// Global custom rules store. Entries are (lowercase_spoken, written).
    static ref CUSTOM_RULES: RwLock<Vec<(String, String)>> = RwLock::new(Vec::new());
}

/// Add a custom spoken→written mapping.
///
/// The spoken form is stored lowercased for case-insensitive matching.
/// If the same spoken form already exists, it is replaced.
pub fn add_rule(spoken: &str, written: &str) {
    let spoken_lower = spoken.to_lowercase();
    let mut rules = CUSTOM_RULES.write().unwrap();
    // Replace if exists
    if let Some(entry) = rules.iter_mut().find(|(s, _)| *s == spoken_lower) {
        entry.1 = written.to_string();
    } else {
        rules.push((spoken_lower, written.to_string()));
    }
}

/// Remove a custom rule by its spoken form.
///
/// Returns true if the rule was found and removed.
pub fn remove_rule(spoken: &str) -> bool {
    let spoken_lower = spoken.to_lowercase();
    let mut rules = CUSTOM_RULES.write().unwrap();
    let len_before = rules.len();
    rules.retain(|(s, _)| *s != spoken_lower);
    rules.len() < len_before
}

/// Clear all custom rules.
pub fn clear_rules() {
    let mut rules = CUSTOM_RULES.write().unwrap();
    rules.clear();
}

/// Try to match input against custom rules (exact match, case-insensitive).
///
/// Returns `Some(written_form)` if a rule matches, `None` otherwise.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    let rules = CUSTOM_RULES.read().unwrap();
    for (spoken, written) in rules.iter() {
        if input_trimmed == spoken {
            return Some(written.clone());
        }
    }

    None
}

/// First private-use code point of [`protect`]'s placeholders; one per match.
const PLACEHOLDER_BASE: u32 = 0xE000;
const PLACEHOLDER_LAST: u32 = 0xF8FF;

/// Mark every registered spoken form in `input` for NeMo's grammars: each match
/// becomes a private-use placeholder the grammars pass through, and the written
/// forms come back from [`restore`] afterwards, so NeMo can't re-read them
/// (`"two dollars"` stays `"two dollars"`). Words match case-insensitively and
/// exactly, longest rule first; only punctuation around the whole phrase may differ
/// (`"gee pee tee."` matches, `"gee, pee tee"` doesn't), and it is kept.
pub(crate) fn protect(input: &str) -> (std::borrow::Cow<'_, str>, Vec<String>) {
    let rules = CUSTOM_RULES.read().unwrap();
    if rules.is_empty() {
        return (std::borrow::Cow::Borrowed(input), Vec::new());
    }
    let mut rules: Vec<(Vec<&str>, &str)> = rules
        .iter()
        .map(|(spoken, written)| {
            (
                spoken.split_whitespace().collect::<Vec<_>>(),
                written.as_str(),
            )
        })
        .filter(|(spoken, _)| !spoken.is_empty())
        .collect();
    rules.sort_by_key(|(spoken, _)| std::cmp::Reverse(spoken.len()));

    let tokens: Vec<String> = input.split_whitespace().map(str::to_lowercase).collect();
    let originals: Vec<&str> = input.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut written_forms = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let matched = rules.iter().find_map(|(spoken, written)| {
            let n = spoken.len();
            if i + n > tokens.len()
                || PLACEHOLDER_BASE + written_forms.len() as u32 > PLACEHOLDER_LAST
            {
                return None;
            }
            let (lead, trail) = phrase_affixes(&tokens[i..i + n], spoken)?;
            Some((n, lead, trail, *written))
        });
        let Some((n, lead, trail, written)) = matched else {
            out.push(originals[i].to_string());
            i += 1;
            continue;
        };
        let placeholder = char::from_u32(PLACEHOLDER_BASE + written_forms.len() as u32).unwrap();
        out.push(format!("{lead}{placeholder}{trail}"));
        written_forms.push(written.to_string());
        i += n;
    }
    if written_forms.is_empty() {
        (std::borrow::Cow::Borrowed(input), written_forms)
    } else {
        (std::borrow::Cow::Owned(out.join(" ")), written_forms)
    }
}

/// Put [`protect`]'s written forms back in place of their placeholders.
pub(crate) fn restore(output: &str, written_forms: &[String]) -> String {
    if written_forms.is_empty() {
        return output.to_string();
    }
    let mut restored = String::with_capacity(output.len());
    for c in output.chars() {
        let index = (c as u32).wrapping_sub(PLACEHOLDER_BASE) as usize;
        match written_forms.get(index) {
            Some(written) if (c as u32) >= PLACEHOLDER_BASE => restored.push_str(written),
            _ => restored.push(c),
        }
    }
    restored
}

/// The punctuation before and after `tokens` (lowercased input words) when they
/// spell `spoken`: the words must match exactly, and only the first token may
/// carry extra leading punctuation and the last extra trailing punctuation.
fn phrase_affixes<'a>(tokens: &'a [String], spoken: &[&str]) -> Option<(&'a str, &'a str)> {
    let is_mark = |c: char| !c.is_alphanumeric() && c != '\'';
    let last = spoken.len() - 1;
    let mut lead = "";
    let mut trail = "";
    for (k, (token, word)) in tokens.iter().zip(spoken).enumerate() {
        let token = token.as_str();
        // The extra leading marks the first token may have, then the word, then the
        // extra trailing marks the last token may have.
        let start = if k == 0 {
            (0..=token.len())
                .filter(|&at| token.is_char_boundary(at) && token[at..].starts_with(word))
                .find(|&at| token[..at].chars().all(is_mark))?
        } else if token.starts_with(word) {
            0
        } else {
            return None;
        };
        let end = start + word.len();
        let rest = &token[end..];
        if k == last {
            if !rest.chars().all(is_mark) {
                return None;
            }
            trail = rest;
        } else if !rest.is_empty() {
            return None;
        }
        if k == 0 {
            lead = &token[..start];
        }
    }
    Some((lead, trail))
}

/// Get the number of custom rules currently registered.
pub fn rule_count() -> usize {
    let rules = CUSTOM_RULES.read().unwrap();
    rules.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Single test to avoid parallel test races on shared global state.
    #[test]
    fn test_custom_rules() {
        clear_rules();

        // Add and parse
        add_rule("gee pee tee", "GPT");
        assert_eq!(parse("gee pee tee"), Some("GPT".to_string()));
        assert_eq!(parse("Gee Pee Tee"), Some("GPT".to_string()));
        assert_eq!(parse("unknown"), None);

        // Replace existing
        add_rule("gee pee tee", "GPT-4");
        assert_eq!(parse("gee pee tee"), Some("GPT-4".to_string()));
        assert_eq!(rule_count(), 1);

        // Remove
        assert!(remove_rule("gee pee tee"));
        assert_eq!(parse("gee pee tee"), None);
        assert!(!remove_rule("gee pee tee"));

        // Multiple rules + clear
        add_rule("alpha", "A");
        add_rule("bravo", "B");
        assert_eq!(rule_count(), 2);
        assert_eq!(parse("alpha"), Some("A".to_string()));
        assert_eq!(parse("bravo"), Some("B".to_string()));
        clear_rules();
        assert_eq!(rule_count(), 0);
        assert_eq!(parse("alpha"), None);
    }
}
