//! Telephone TN tagger for Hindi (romanized).
//!
//! Converts written phone numbers to spoken form with Hindi romanized digit words:
//! - "123-456-7890" -> "ek do teen, chaar paanch chhah, saat aath nau shunya"
//! - "+91-98765-43210" -> "plus nau ek, nau aath saat chhah paanch, chaar teen do ek shunya"
//! - "(011) 2345-6789" -> "shunya ek ek, do teen chaar paanch, chhah saat aath nau"

/// Parse a written phone number to spoken form (Hindi romanized).
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Phone numbers contain digits and separators (-, ., space, parens)
    // Must have mostly digits
    let digit_count = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
    let non_digit_non_sep = trimmed
        .chars()
        .filter(|c| {
            !c.is_ascii_digit()
                && *c != '-'
                && *c != '.'
                && *c != ' '
                && *c != '('
                && *c != ')'
                && *c != '+'
        })
        .count();

    // Must have at least 7 digits and no unexpected characters
    if digit_count < 7 || non_digit_non_sep > 0 {
        return None;
    }

    // Must contain at least one separator (-, ., space, parens) to distinguish
    // from plain numbers like "1000000"
    let has_separator = trimmed
        .chars()
        .any(|c| c == '-' || c == '.' || c == ' ' || c == '(' || c == ')');
    if !has_separator {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    let mut has_plus = false;

    // Handle leading +
    let rest = if let Some(r) = trimmed.strip_prefix('+') {
        has_plus = true;
        r.trim_start()
    } else {
        trimmed
    };

    // Split by common separators
    let groups = split_phone_groups(rest);

    if has_plus && !groups.is_empty() {
        // The first group after + is the country code
        let mut first = String::from("plus ");
        first.push_str(&spell_digit_group_hi(&groups[0]));
        parts.push(first);
        for g in &groups[1..] {
            parts.push(spell_digit_group_hi(g));
        }
    } else {
        for g in &groups {
            parts.push(spell_digit_group_hi(g));
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(", "))
}

/// Split phone number into groups by separators.
fn split_phone_groups(input: &str) -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    let mut current = String::new();

    for c in input.chars() {
        match c {
            '0'..='9' => current.push(c),
            '-' | '.' | ' ' | '(' | ')' => {
                if !current.is_empty() {
                    groups.push(current.clone());
                    current.clear();
                }
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        groups.push(current);
    }

    groups
}

/// Spell each digit in a group using Hindi romanized words.
fn spell_digit_group_hi(group: &str) -> String {
    group
        .chars()
        .filter_map(|c| {
            let word = match c {
                '0' => "shunya",
                '1' => "ek",
                '2' => "do",
                '3' => "teen",
                '4' => "chaar",
                '5' => "paanch",
                '6' => "chhah",
                '7' => "saat",
                '8' => "aath",
                '9' => "nau",
                _ => return None,
            };
            Some(word)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_phone() {
        assert_eq!(
            parse("123-456-7890"),
            Some("ek do teen, chaar paanch chhah, saat aath nau shunya".to_string())
        );
    }

    #[test]
    fn test_with_country_code() {
        assert_eq!(
            parse("+91-98765-43210"),
            Some("plus nau ek, nau aath saat chhah paanch, chaar teen do ek shunya".to_string())
        );
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(
            parse("(011) 2345-6789"),
            Some("shunya ek ek, do teen chaar paanch, chhah saat aath nau".to_string())
        );
    }

    #[test]
    fn test_dots() {
        assert_eq!(
            parse("555.123.4567"),
            Some("paanch paanch paanch, ek do teen, chaar paanch chhah saat".to_string())
        );
    }

    #[test]
    fn test_non_phone() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
