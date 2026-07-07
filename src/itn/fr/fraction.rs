//! Fraction tagger for French.
//!
//! Converts spoken French fractions to written form:
//! - "un tiers" → "1/3"
//! - "deux neuvièmes" → "2/9"
//! - "un et demi" → "1 1/2" (mixed)
//! - "quatre et deux quatrièmes" → "4 2/4" (mixed)

use super::cardinal;

/// Parse a spoken French fraction to written form.
pub fn parse(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let trimmed = lower.trim();

    // Mixed fraction: "WHOLE et FRACTION" → "WHOLE N/D"
    // ("un et demi" → "1 1/2", "quatre et deux quatrièmes" → "4 2/4").
    if let Some(pos) = trimmed.find(" et ") {
        let whole_part = &trimmed[..pos];
        let frac_part = &trimmed[pos + " et ".len()..];
        if let (Some(whole), Some(frac)) =
            (word_to_int(whole_part), parse_simple_fraction(frac_part))
        {
            return Some(format!("{} {}", whole, frac));
        }
    }

    parse_simple_fraction(trimmed)
}

/// Parse a simple fraction: "deux neuvièmes" → "2/9", bare "demi" → "1/2".
fn parse_simple_fraction(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Bare "demi" carries an implicit numerator of one.
    if trimmed == "demi" {
        return Some("1/2".to_string());
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    let denom = parse_denominator(tokens.last()?)?;
    let numer = word_to_int(&tokens[..tokens.len() - 1].join(" "))?;
    Some(format!("{}/{}", numer, denom))
}

/// French number word → integer, treating bare `un`/`une` as one (the
/// cardinal tagger expects a fuller phrase).
fn word_to_int(input: &str) -> Option<i128> {
    let trimmed = input.trim();
    if trimmed == "un" || trimmed == "une" {
        return Some(1);
    }
    cardinal::words_to_number(trimmed)
}

/// French ordinal (or `demi`/`tiers`/`quart`) denominator word → integer.
/// Accepts the plural `-s` form (`neuvièmes`).
fn parse_denominator(word: &str) -> Option<i128> {
    match word {
        "demi" | "demis" => return Some(2),
        "tiers" => return Some(3),
        "quart" | "quarts" => return Some(4),
        _ => {}
    }

    // Ordinal `-ième(s)` forms; drop the plural `s` first.
    let singular = word.strip_suffix('s').unwrap_or(word);
    let value = match singular {
        "quatrième" => 4,
        "cinquième" => 5,
        "sixième" => 6,
        "septième" => 7,
        "huitième" => 8,
        "neuvième" => 9,
        "dixième" => 10,
        "onzième" => 11,
        "douzième" => 12,
        "treizième" => 13,
        "quatorzième" => 14,
        "quinzième" => 15,
        "seizième" => 16,
        "dix-septième" => 17,
        "dix-huitième" => 18,
        "dix-neuvième" => 19,
        "vingtième" => 20,
        "trentième" => 30,
        "quarantième" => 40,
        "cinquantième" => 50,
        "soixantième" => 60,
        "soixante-dixième" => 70,
        "quatre-vingtième" => 80,
        "quatre-vingt-dixième" => 90,
        "centième" => 100,
        _ => return None,
    };
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(parse("un tiers"), Some("1/3".to_string()));
        assert_eq!(parse("un quart"), Some("1/4".to_string()));
        assert_eq!(parse("un cinquième"), Some("1/5".to_string()));
        assert_eq!(parse("un quatrième"), Some("1/4".to_string()));
        assert_eq!(parse("un centième"), Some("1/100".to_string()));
    }

    #[test]
    fn test_bare_demi() {
        assert_eq!(parse("demi"), Some("1/2".to_string()));
    }

    #[test]
    fn test_plural_denominator() {
        assert_eq!(parse("deux neuvièmes"), Some("2/9".to_string()));
    }

    #[test]
    fn test_compound_denominator() {
        assert_eq!(parse("un dix-septième"), Some("1/17".to_string()));
        assert_eq!(parse("un soixante-dixième"), Some("1/70".to_string()));
        assert_eq!(parse("un quatre-vingt-dixième"), Some("1/90".to_string()));
    }

    #[test]
    fn test_mixed() {
        assert_eq!(parse("un et demi"), Some("1 1/2".to_string()));
        assert_eq!(
            parse("quatre et deux quatrièmes"),
            Some("4 2/4".to_string())
        );
    }

    #[test]
    fn test_not_a_fraction() {
        assert_eq!(parse("bonjour"), None);
        assert_eq!(parse("cinq"), None);
    }
}
