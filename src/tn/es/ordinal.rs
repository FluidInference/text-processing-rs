//! Ordinal TN tagger for Spanish.
//!
//! Converts written ordinal numbers to spoken Spanish:
//! - "1.o" → "primero"
//! - "2.o" → "segundo"
//! - "3.o" → "tercero"
//! - "1.a" → "primera"
//! - "10.o" → "decimo"

use super::number_to_words;

/// Ordinal words for 1-10 (masculine).
const ORDINALS_M: [&str; 11] = [
    "", "primero", "segundo", "tercero", "cuarto", "quinto", "sexto", "septimo", "octavo",
    "noveno", "decimo",
];

/// Ordinal words for 1-10 (feminine).
const ORDINALS_F: [&str; 11] = [
    "", "primera", "segunda", "tercera", "cuarta", "quinta", "sexta", "septima", "octava",
    "novena", "decima",
];

/// Higher ordinals 11-20 (masculine).
const ORDINALS_HIGH_M: [&str; 10] = [
    "undecimo",
    "duodecimo",
    "decimotercero",
    "decimocuarto",
    "decimoquinto",
    "decimosexto",
    "decimoseptimo",
    "decimoctavo",
    "decimonoveno",
    "vigesimo",
];

/// Higher ordinals 11-20 (feminine).
const ORDINALS_HIGH_F: [&str; 10] = [
    "undecima",
    "duodecima",
    "decimotercera",
    "decimocuarta",
    "decimoquinta",
    "decimosexta",
    "decimoseptima",
    "decimoctava",
    "decimonovena",
    "vigesima",
];

/// Parse a written ordinal to spoken Spanish words.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Detect Spanish ordinal suffixes: .o, .a, .er, .os, .as
    // Also handle without dot: 1o, 1a, 2o, etc.
    let (num_str, feminine) = if let Some(s) = trimmed.strip_suffix(".a") {
        (s, true)
    } else if let Some(s) = trimmed.strip_suffix(".as") {
        (s, true)
    } else if let Some(s) = trimmed.strip_suffix(".o") {
        (s, false)
    } else if let Some(s) = trimmed.strip_suffix(".os") {
        (s, false)
    } else if let Some(s) = trimmed.strip_suffix(".er") {
        (s, false)
    } else if trimmed.len() >= 2 {
        // Try without dot: "1o", "2a", "3er"
        if let Some(s) = trimmed.strip_suffix("er") {
            if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
                (s, false)
            } else {
                return None;
            }
        } else if let Some(s) = trimmed.strip_suffix("os") {
            if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
                (s, false)
            } else {
                return None;
            }
        } else if let Some(s) = trimmed.strip_suffix("as") {
            if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
                (s, true)
            } else {
                return None;
            }
        } else {
            let last = trimmed.chars().last()?;
            let rest = &trimmed[..trimmed.len() - 1];
            if last == 'a' && rest.chars().all(|c| c.is_ascii_digit()) && !rest.is_empty() {
                (rest, true)
            } else if last == 'o' && rest.chars().all(|c| c.is_ascii_digit()) && !rest.is_empty() {
                (rest, false)
            } else {
                return None;
            }
        }
    } else {
        return None;
    };

    if num_str.is_empty() || !num_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: i64 = num_str.parse().ok()?;
    if n <= 0 {
        return None;
    }

    // Ordinals 1-10: use dedicated words
    if n <= 10 {
        let ordinal = if feminine {
            ORDINALS_F[n as usize]
        } else {
            ORDINALS_M[n as usize]
        };
        return Some(ordinal.to_string());
    }

    // Ordinals 11-20: use dedicated higher ordinal words
    if n <= 20 {
        let idx = (n - 11) as usize;
        let ordinal = if feminine {
            ORDINALS_HIGH_F[idx]
        } else {
            ORDINALS_HIGH_M[idx]
        };
        return Some(ordinal.to_string());
    }

    // For 21+, fall back to cardinal form (common in spoken Spanish)
    Some(number_to_words(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_masculine() {
        assert_eq!(parse("1.o"), Some("primero".to_string()));
        assert_eq!(parse("2.o"), Some("segundo".to_string()));
        assert_eq!(parse("3.o"), Some("tercero".to_string()));
        assert_eq!(parse("5.o"), Some("quinto".to_string()));
        assert_eq!(parse("10.o"), Some("decimo".to_string()));
    }

    #[test]
    fn test_basic_feminine() {
        assert_eq!(parse("1.a"), Some("primera".to_string()));
        assert_eq!(parse("2.a"), Some("segunda".to_string()));
        assert_eq!(parse("3.a"), Some("tercera".to_string()));
    }

    #[test]
    fn test_higher_ordinals() {
        assert_eq!(parse("11.o"), Some("undecimo".to_string()));
        assert_eq!(parse("12.o"), Some("duodecimo".to_string()));
        assert_eq!(parse("20.o"), Some("vigesimo".to_string()));
    }

    #[test]
    fn test_without_dot() {
        assert_eq!(parse("1o"), Some("primero".to_string()));
        assert_eq!(parse("1a"), Some("primera".to_string()));
        assert_eq!(parse("3er"), Some("tercero".to_string()));
    }

    #[test]
    fn test_fallback_cardinal() {
        assert_eq!(parse("21.o"), Some("veintiuno".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("0.o"), None);
    }
}
