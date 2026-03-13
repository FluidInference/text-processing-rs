//! Electronic tagger for French.
//!
//! Converts spoken French electronic addresses to written form:
//! - "test arobase gmail point com" → "test@gmail.com"
//! - Handles email addresses and URLs

/// Parse spoken French electronic address to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Try email pattern
    if let Some(result) = parse_email(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse email address pattern
fn parse_email(input: &str) -> Option<String> {
    // Look for "arobase" (at) as the key indicator
    if !input.contains("arobase") {
        return None;
    }

    let parts: Vec<&str> = input.split("arobase").collect();
    if parts.len() != 2 {
        return None;
    }

    let local_part = convert_email_part(parts[0].trim());
    let domain_part = convert_email_part(parts[1].trim());

    if local_part.is_empty() || domain_part.is_empty() {
        return None;
    }

    Some(format!("{}@{}", local_part, domain_part))
}

/// Convert email part (replace "point" with ".", keep other words)
fn convert_email_part(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            if word == "point" {
                "."
            } else if word == "tiret" {
                "-"
            } else if word == "tiret du bas" || word == "sous-tiret" {
                "_"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_email() {
        assert_eq!(
            parse("test arobase gmail point com"),
            Some("test@gmail.com".to_string())
        );
    }

    #[test]
    fn test_email_with_dash() {
        assert_eq!(
            parse("jean tiret luc arobase example point com"),
            Some("jean-luc@example.com".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("test at gmail dot com"), None); // English, not French
        assert_eq!(parse("arobase"), None); // Missing parts
    }
}
