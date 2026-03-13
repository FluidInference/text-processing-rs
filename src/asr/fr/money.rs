//! Money tagger for French.
//!
//! Converts spoken French currency expressions to written form:
//! - "cinq euros" → "5 €"
//! - "cinq euros et cinquante centimes" → "5,50 €"
//! - "cinquante centimes" → "0,50 €"
//! - "un euro" → "1 €"

use super::cardinal::words_to_number;

/// Parse spoken French money expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Try euros and centimes pattern
    if let Some(result) = parse_euros_and_centimes(&input_lower) {
        return Some(result);
    }

    // Try euros only
    if let Some(result) = parse_euros(&input_lower) {
        return Some(result);
    }

    // Try centimes only
    if let Some(result) = parse_centimes(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse "X euros et Y centimes" pattern
fn parse_euros_and_centimes(input: &str) -> Option<String> {
    // Pattern: "X euros et Y centimes"
    if let Some((euros_part, rest)) = input.split_once(" euros et ") {
        if rest.ends_with(" centimes") {
            let centimes_words = rest.trim_end_matches(" centimes");
            let euros = if euros_part == "zero" {
                0
            } else {
                words_to_number(euros_part)? as i64
            };
            let centimes = if centimes_words == "zero" {
                0
            } else {
                words_to_number(centimes_words)? as i64
            };
            return Some(format!("{},{:02} €", euros, centimes));
        }
    }

    // Pattern: "X euro et Y centimes" (singular)
    if let Some((euros_part, rest)) = input.split_once(" euro et ") {
        if rest.ends_with(" centimes") {
            let centimes_words = rest.trim_end_matches(" centimes");
            let euros = if euros_part == "zero" {
                0
            } else {
                words_to_number(euros_part)? as i64
            };
            let centimes = if centimes_words == "zero" {
                0
            } else {
                words_to_number(centimes_words)? as i64
            };
            return Some(format!("{},{:02} €", euros, centimes));
        }
    }

    None
}

/// Parse "X euros" pattern
fn parse_euros(input: &str) -> Option<String> {
    if input.ends_with(" euros") {
        let euros_words = input.trim_end_matches(" euros");
        let euros = if euros_words == "zero" {
            0
        } else {
            words_to_number(euros_words)? as i64
        };
        return Some(format!("{} €", euros));
    }

    if input.ends_with(" euro") {
        let euros_words = input.trim_end_matches(" euro");
        let euros = if euros_words == "zero" {
            0
        } else {
            words_to_number(euros_words)? as i64
        };
        return Some(format!("{} €", euros));
    }

    None
}

/// Parse "X centimes" pattern
fn parse_centimes(input: &str) -> Option<String> {
    // Only match "centimes", not "cents" (which is plural of "cent" = hundred)
    if input.ends_with(" centimes") {
        let centimes_words = input.trim_end_matches(" centimes");
        let centimes = if centimes_words == "zero" {
            0
        } else {
            words_to_number(centimes_words)? as i64
        };
        return Some(format!("0,{:02} €", centimes));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euros() {
        assert_eq!(parse("cinq euros"), Some("5 €".to_string()));
        assert_eq!(parse("un euro"), Some("1 €".to_string()));
        assert_eq!(parse("cent euros"), Some("100 €".to_string()));
        assert_eq!(parse("mille euros"), Some("1000 €".to_string()));
    }

    #[test]
    fn test_euros_and_centimes() {
        assert_eq!(
            parse("cinq euros et cinquante centimes"),
            Some("5,50 €".to_string())
        );
        assert_eq!(
            parse("un euro et vingt centimes"),
            Some("1,20 €".to_string())
        );
        assert_eq!(
            parse("dix euros et un centimes"),
            Some("10,01 €".to_string())
        );
    }

    #[test]
    fn test_centimes_only() {
        assert_eq!(parse("cinquante centimes"), Some("0,50 €".to_string()));
        assert_eq!(parse("un centimes"), Some("0,01 €".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("cinq"), None);
    }
}
