//! Date tagger for German.
//!
//! Converts spoken German date expressions to written form:
//! - "vierundzwanzigster juli zwei tausend dreizehn" → "24. Jul. 2013"
//! - "neunzehn achtzig" → "1980"
//! - "januar zweitausendneun" → "Jan. 2009"
//! - "vierzehnter januar" → "14. Jan."

use super::cardinal;

const MONTHS: [(&str, &str); 12] = [
    ("januar", "Jan."),
    ("februar", "Feb."),
    ("märz", "Mär."),
    ("april", "Apr."),
    ("mai", "Mai"),
    ("juni", "Jun."),
    ("juli", "Jul."),
    ("august", "Aug."),
    ("september", "Sep."),
    ("oktober", "Okt."),
    ("november", "Nov."),
    ("dezember", "Dez."),
];

/// Parse spoken German date expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Try full date: "vierundzwanzigster juli zwei tausend dreizehn"
    if let Some(result) = parse_full_date(input_trim) {
        return Some(result);
    }

    // Try day + month: "vierzehnter januar"
    if let Some(result) = parse_day_month(input_trim) {
        return Some(result);
    }

    // Try month + year: "januar zweitausendneun"
    if let Some(result) = parse_month_year(input_trim) {
        return Some(result);
    }

    // Try year patterns: "neunzehn achtzig" → 1980
    if let Some(result) = parse_year_pattern(input_trim) {
        return Some(result);
    }

    None
}

/// Parse full date: "Nter MONAT JAHR"
fn parse_full_date(input: &str) -> Option<String> {
    for &(month_name, month_abbr) in &MONTHS {
        if let Some(pos) = input.find(month_name) {
            let before = input[..pos].trim();
            let after = input[pos + month_name.len()..].trim();

            // Parse day (ordinal) before month
            let day = parse_ordinal_day(before)?;
            if day < 1 || day > 31 {
                return None;
            }

            // Parse year after month
            if after.is_empty() {
                return None; // This is day+month, handled by parse_day_month
            }

            let year = parse_year(after)?;

            return Some(format!("{}. {} {}", day, month_abbr, year));
        }
    }
    None
}

/// Parse day + month: "vierzehnter januar" → "14. Jan."
fn parse_day_month(input: &str) -> Option<String> {
    for &(month_name, month_abbr) in &MONTHS {
        if input.ends_with(month_name) {
            let before = input[..input.len() - month_name.len()].trim();
            let day = parse_ordinal_day(before)?;
            if day < 1 || day > 31 {
                return None;
            }
            return Some(format!("{}. {}", day, month_abbr));
        }
    }
    None
}

/// Parse month + year: "januar zweitausendneun" → "Jan. 2009"
fn parse_month_year(input: &str) -> Option<String> {
    for &(month_name, month_abbr) in &MONTHS {
        if input.starts_with(month_name) {
            let after = input[month_name.len()..].trim();
            if after.is_empty() {
                continue;
            }
            // Reject compound: "januarzweitausendneun" (no space)
            if !input.contains(' ') {
                return None;
            }
            let year = parse_year(after)?;
            return Some(format!("{} {}", month_abbr, year));
        }
    }
    None
}

/// Parse year patterns:
/// - "neunzehn achtzig" → 1980
/// - "neunzehnhundertachtzig" → 1980
/// - "zwei tausend zwanzig" → 2020
/// - "zwanzig zwanzig" → 2020
fn parse_year_pattern(input: &str) -> Option<String> {
    // Reject if contains "achtziger" etc. (decade reference, not year)
    if input.ends_with("iger") || input.ends_with("er") {
        // Check if it ends with a decade suffix
        let decade_suffixes = [
            "achtziger",
            "siebziger",
            "sechziger",
            "fünfziger",
            "vierziger",
            "dreißiger",
            "zwanziger",
            "neunziger",
        ];
        for &suffix in &decade_suffixes {
            if input.ends_with(suffix) {
                // This is "neunzehn achtziger" → "19 achtziger"
                let before = input[..input.len() - suffix.len()].trim();
                if !before.is_empty() {
                    let num = cardinal::words_to_number(before)?;
                    return Some(format!("{} {}", num, suffix));
                }
                return None;
            }
        }
    }

    let year = parse_year(input)?;
    Some(year.to_string())
}

/// Parse a year value from German words
fn parse_year(input: &str) -> Option<i128> {
    // Try direct cardinal parsing first
    if let Some(num) = cardinal::words_to_number(input) {
        if num >= 1000 && num <= 9999 {
            return Some(num);
        }
    }

    // Try "CENTURY DECADE" pattern: "neunzehn achtzig" → 1980
    // Also handles compound form: "neunzehnachtzig" → decompose → "neunzehn achtzig"
    // And spaced compound decades: "neunzehn vierundneunzig" → 1994

    // First try with original whitespace-split tokens
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() == 2 {
        if let Some(century) = cardinal::words_to_number(tokens[0]) {
            if let Some(decade) = cardinal::words_to_number(tokens[1]) {
                if century >= 10 && century <= 99 && decade >= 0 && decade <= 99 {
                    let year = century * 100 + decade;
                    if year >= 1000 && year <= 9999 {
                        return Some(year);
                    }
                }
            }
        }
    }

    // Try compound form: "neunzehnachtzig" → decompose → "neunzehn achtzig"
    if tokens.len() == 1 {
        let decomposed = cardinal::decompose_compound_public(input);
        let dtokens: Vec<&str> = decomposed.split_whitespace().collect();
        if dtokens.len() == 2 {
            if let Some(century) = cardinal::words_to_number(dtokens[0]) {
                if let Some(decade) = cardinal::words_to_number(dtokens[1]) {
                    if century >= 10 && century <= 99 && decade >= 0 && decade <= 99 {
                        let year = century * 100 + decade;
                        if year >= 1000 && year <= 9999 {
                            return Some(year);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Parse ordinal day number from German ordinal word.
/// "erster" → 1, "vierundzwanzigster" → 24, "dreißigster" → 30
fn parse_ordinal_day(input: &str) -> Option<i128> {
    // Strip ordinal suffix
    let ordinal_suffixes = [
        "ster", "sten", "stem", "stes", "ste", "ter", "ten", "tem", "tes", "te",
    ];

    for &suffix in &ordinal_suffixes {
        if input.ends_with(suffix) {
            let stem = &input[..input.len() - suffix.len()];
            // Reconstruct the cardinal form
            let cardinal_form = reconstruct_cardinal_from_ordinal(stem);
            return cardinal::words_to_number(&cardinal_form);
        }
    }

    None
}

/// Reconstruct cardinal form from ordinal stem.
/// "er" → "eins" (from "erster"), "vierundzwanzig" stays, etc.
fn reconstruct_cardinal_from_ordinal(stem: &str) -> String {
    match stem {
        "er" | "ers" => "eins".to_string(),
        "zwei" => "zwei".to_string(),
        "drit" => "drei".to_string(),
        "vier" => "vier".to_string(),
        "fünf" => "fünf".to_string(),
        "sechs" => "sechs".to_string(),
        "sieb" => "sieben".to_string(),
        "ach" => "acht".to_string(),
        "neun" => "neun".to_string(),
        "zehn" => "zehn".to_string(),
        "elf" => "elf".to_string(),
        "zwölf" => "zwölf".to_string(),
        _ => {
            // For compound ordinals, the stem is already the cardinal form
            // e.g., "vierundzwanzig" from "vierundzwanzigster"
            // But we need to handle "hundert" → "hundert" (from "hundertste")
            stem.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_date() {
        assert_eq!(
            parse("vierundzwanzigster juli zwei tausend dreizehn"),
            Some("24. Jul. 2013".to_string())
        );
    }

    #[test]
    fn test_day_month() {
        assert_eq!(parse("vierzehnter januar"), Some("14. Jan.".to_string()));
        assert_eq!(parse("erster januar"), Some("1. Jan.".to_string()));
    }

    #[test]
    fn test_year() {
        assert_eq!(parse("neunzehn achtzig"), Some("1980".to_string()));
        assert_eq!(parse("zwei tausend zwanzig"), Some("2020".to_string()));
    }

    #[test]
    fn test_month_year() {
        assert_eq!(
            parse("januar zweitausendneun"),
            Some("Jan. 2009".to_string())
        );
    }
}
