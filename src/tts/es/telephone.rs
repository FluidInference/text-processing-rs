//! Telephone TN tagger for Spanish.
//!
//! Converts written phone numbers to spoken Spanish form:
//! - "123-456-7890" → "uno dos tres, cuatro cinco seis, siete ocho nueve cero"
//! - "+34-91-123-4567" → "mas tres cuatro, nueve uno, uno dos tres, cuatro cinco seis siete"
//! - "(555) 123-4567" → "cinco cinco cinco, uno dos tres, cuatro cinco seis siete"

/// Parse a written phone number to spoken Spanish form.
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
        let mut first = String::from("mas ");
        first.push_str(&spell_digit_group(&groups[0]));
        parts.push(first);
        for g in &groups[1..] {
            parts.push(spell_digit_group(g));
        }
    } else {
        for g in &groups {
            parts.push(spell_digit_group(g));
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

/// Spell each digit in a group using Spanish words.
fn spell_digit_group(group: &str) -> String {
    group
        .chars()
        .filter_map(|c| {
            let word = match c {
                '0' => "cero",
                '1' => "uno",
                '2' => "dos",
                '3' => "tres",
                '4' => "cuatro",
                '5' => "cinco",
                '6' => "seis",
                '7' => "siete",
                '8' => "ocho",
                '9' => "nueve",
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
            Some("uno dos tres, cuatro cinco seis, siete ocho nueve cero".to_string())
        );
    }

    #[test]
    fn test_with_country_code() {
        assert_eq!(
            parse("+34-91-123-4567"),
            Some(
                "mas tres cuatro, nueve uno, uno dos tres, cuatro cinco seis siete".to_string()
            )
        );
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(
            parse("(555) 123-4567"),
            Some("cinco cinco cinco, uno dos tres, cuatro cinco seis siete".to_string())
        );
    }

    #[test]
    fn test_dots() {
        assert_eq!(
            parse("555.123.4567"),
            Some("cinco cinco cinco, uno dos tres, cuatro cinco seis siete".to_string())
        );
    }

    #[test]
    fn test_non_phone() {
        assert_eq!(parse("hola"), None);
        assert_eq!(parse("123"), None); // too few digits
    }
}
