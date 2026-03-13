//! Date tagger for Spanish.
//!
//! Converts spoken Spanish date expressions to written form:
//! - "primero de enero" → "1 de enero"
//! - "siglo diecinueve" → "siglo xix"
//! - "doscientos tres antes de cristo" → "203 a. c."

use super::cardinal;

const MONTHS: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];

const DAYS_OF_WEEK: [&str; 7] = [
    "lunes",
    "martes",
    "miércoles",
    "jueves",
    "viernes",
    "sábado",
    "domingo",
];

/// Parse spoken Spanish date expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Try "siglo X" → "siglo xix"
    if let Some(result) = parse_siglo(input_trim) {
        return Some(result);
    }

    // Try "X antes de cristo" → "X a. c."
    if let Some(result) = parse_antes_de_cristo(input_trim) {
        return Some(result);
    }

    // Try full date: "DAY de MONTH de YEAR"
    if let Some(result) = parse_full_date(input_trim) {
        return Some(result);
    }

    // Try day+month with optional prefix: "[el/DOW] DAY de MONTH"
    if let Some(result) = parse_day_month(input_trim) {
        return Some(result);
    }

    None
}

/// Parse "siglo X" → "siglo xix"
fn parse_siglo(input: &str) -> Option<String> {
    if !input.starts_with("siglo ") {
        return None;
    }
    let rest = &input[6..];
    let num = cardinal::words_to_number(rest)?;
    let roman = to_roman(num as i64)?;
    Some(format!("siglo {}", roman.to_lowercase()))
}

/// Parse "X antes de cristo" → "X a. c."
fn parse_antes_de_cristo(input: &str) -> Option<String> {
    if !input.ends_with(" antes de cristo") {
        return None;
    }
    let before = input[..input.len() - 16].trim();
    let num = cardinal::words_to_number(before)?;
    Some(format!("{} a. c.", num))
}

/// Parse full date: "treinta y uno de diciembre de mil novecientos noventa y dos"
fn parse_full_date(input: &str) -> Option<String> {
    for &month in &MONTHS {
        let de_month_de = format!(" de {} de ", month);
        if let Some(pos) = input.find(&de_month_de) {
            let day_part = &input[..pos];
            let year_part = &input[pos + de_month_de.len()..];

            let day = parse_day(day_part)?;
            let year = cardinal::words_to_number(year_part)?;

            return Some(format!("{} de {} de {}", day, month, year));
        }
    }
    None
}

/// Parse day+month: "[prefix] DAY de MONTH"
fn parse_day_month(input: &str) -> Option<String> {
    for &month in &MONTHS {
        let de_month = format!(" de {}", month);
        if input.ends_with(&de_month) || input.contains(&format!("{} ", &de_month[1..])) {
            // Check if ends with " de MONTH"
            if input.ends_with(&de_month) {
                let before = &input[..input.len() - de_month.len()];

                // Extract prefix (el, day of week)
                let (prefix, day_part) = extract_prefix(before);
                let day = parse_day(day_part)?;

                if let Some(p) = prefix {
                    return Some(format!("{} {} de {}", p, day, month));
                } else {
                    return Some(format!("{} de {}", day, month));
                }
            }
        }
    }
    None
}

/// Extract prefix like "el" or day of week
fn extract_prefix(input: &str) -> (Option<&str>, &str) {
    let trimmed = input.trim();

    // Check for "el"
    if trimmed.starts_with("el ") {
        return (Some("el"), trimmed[3..].trim());
    }

    // Check for day of week
    for &dow in &DAYS_OF_WEEK {
        if trimmed.starts_with(dow) {
            let rest = trimmed[dow.len()..].trim();
            return (Some(dow), rest);
        }
    }

    (None, trimmed)
}

/// Parse day number (handles "primero" → 1, "uno" → 1, number words → number)
fn parse_day(input: &str) -> Option<i128> {
    let trimmed = input.trim();
    match trimmed {
        "primero" | "primer" => Some(1),
        "uno" | "una" | "un" => Some(1),
        _ => cardinal::words_to_number(trimmed),
    }
}

/// Convert number to Roman numeral (lowercase)
fn to_roman(num: i64) -> Option<String> {
    if num <= 0 || num > 3999 {
        return None;
    }
    let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    let symbols = [
        "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
    ];

    let mut result = String::new();
    let mut remaining = num;
    for (i, &val) in values.iter().enumerate() {
        while remaining >= val {
            result.push_str(symbols[i]);
            remaining -= val;
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_month() {
        assert_eq!(parse("primero de enero"), Some("1 de enero".to_string()));
        assert_eq!(parse("uno de enero"), Some("1 de enero".to_string()));
    }

    #[test]
    fn test_with_article() {
        assert_eq!(
            parse("el uno de diciembre"),
            Some("el 1 de diciembre".to_string())
        );
    }

    #[test]
    fn test_siglo() {
        assert_eq!(parse("siglo diecinueve"), Some("siglo xix".to_string()));
    }

    #[test]
    fn test_antes_de_cristo() {
        assert_eq!(
            parse("doscientos tres antes de cristo"),
            Some("203 a. c.".to_string())
        );
    }
}
