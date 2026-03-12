//! Ordinal number tagger for French.
//!
//! Converts spoken French ordinal words to written form with Unicode superscripts:
//! - "premier" → "1ᵉʳ"
//! - "première" → "1ʳᵉ"
//! - "deuxième" → "2ᵉ"
//! - "troisièmes" → "3ᵉˢ"
//! - "second" → "2ᵈ"

use super::cardinal::words_to_number;

/// Parse spoken French ordinal number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Special case: "Xième siècle" → Roman numerals
    if input_trim.ends_with(" siècle") {
        return parse_century(input_trim);
    }

    // Try to extract ordinal suffix and detect plural
    if let Some((number_str, suffix)) = extract_ordinal_parts(input_trim) {
        // Parse the number part
        let number = if number_str.is_empty() || number_str == "premier" || number_str == "première" {
            1
        } else if number_str == "second" || number_str == "seconde" {
            2
        } else {
            words_to_number(&number_str)? as i64
        };

        // Format with appropriate Unicode superscripts
        return Some(format_ordinal(number, &suffix));
    }

    None
}

/// Parse century pattern "Xième siècle"
fn parse_century(input: &str) -> Option<String> {
    let without_siecle = input.strip_suffix(" siècle")?;

    // Extract the ordinal number before "ième"
    if let Some(num_part) = without_siecle.strip_suffix("ième") {
        let num_part = num_part.trim_end_matches('-').trim();
        let number = if num_part.is_empty() {
            return None;
        } else {
            words_to_number(num_part)? as i64
        };

        // Convert to Roman numerals
        return Some(format!("{}ᵉ siècle", int_to_roman(number)));
    }

    None
}

/// Convert integer to Roman numerals (for centuries)
fn int_to_roman(mut num: i64) -> String {
    let values = [
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

    let mut result = String::new();
    for (value, numeral) in &values {
        while num >= *value {
            result.push_str(numeral);
            num -= value;
        }
    }
    result
}

/// Reconstruct cardinal form from ordinal stem
/// E.g., "quatr" → "quatre", "onz" → "onze", "mill" → "mille"
fn reconstruct_cardinal(stem: &str) -> Option<String> {
    // Direct mapping for common ordinal stems that need reconstruction
    let mappings = [
        ("quatr", "quatre"),
        ("cinqu", "cinq"),
        ("neuv", "neuf"),
        ("dix", "dix"),  // stays same
        ("onz", "onze"),
        ("douz", "douze"),
        ("treiz", "treize"),
        ("quatorz", "quatorze"),
        ("quinz", "quinze"),
        ("seiz", "seize"),
        ("vingt", "vingt"),  // stays same
        ("trent", "trente"),
        ("quarant", "quarante"),
        ("cinquant", "cinquante"),
        ("soixant", "soixante"),
        ("cent", "cent"),  // stays same
        ("mill", "mille"),
        ("million", "million"),  // stays same
        ("milliard", "milliard"),  // stays same
    ];

    for (ord_stem, cardinal) in &mappings {
        if stem == *ord_stem || stem.starts_with(*ord_stem) {
            // For compound ordinals like "vingt-et-un", keep the full stem
            if stem.contains('-') || stem.contains(' ') {
                return Some(stem.to_string());
            }
            return Some(cardinal.to_string());
        }
    }

    // If no mapping found, assume stem is already in cardinal form or compound
    if stem.contains('-') || stem.contains(' ') || !stem.is_empty() {
        Some(stem.to_string())
    } else {
        None
    }
}

/// Extract number and ordinal suffix from input
fn extract_ordinal_parts(input: &str) -> Option<(String, OrdinalSuffix)> {
    // Check if the whole word is "premier", "première", "second", "seconde" FIRST
    // before checking ends_with, otherwise they'll match themselves
    if input == "premier" {
        return Some(("premier".to_string(), OrdinalSuffix::PremierM));
    }
    if input == "première" {
        return Some(("première".to_string(), OrdinalSuffix::PremiereF));
    }
    if input == "premiers" {
        return Some(("premier".to_string(), OrdinalSuffix::PremiersM));
    }
    if input == "premières" {
        return Some(("première".to_string(), OrdinalSuffix::PremieresF));
    }
    if input == "second" {
        return Some(("second".to_string(), OrdinalSuffix::SecondM));
    }
    if input == "seconde" {
        return Some(("seconde".to_string(), OrdinalSuffix::SecondeF));
    }
    if input == "seconds" {
        return Some(("second".to_string(), OrdinalSuffix::SecondsM));
    }
    if input == "secondes" {
        return Some(("seconde".to_string(), OrdinalSuffix::SecondesF));
    }

    // Check for specific ordinal endings
    if input.ends_with("premiers") {
        let num_part = input.strip_suffix("premiers")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::PremiersM));
    }
    if input.ends_with("premier") {
        let num_part = input.strip_suffix("premier")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::PremierM));
    }
    if input.ends_with("premières") {
        let num_part = input.strip_suffix("premières")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::PremieresF));
    }
    if input.ends_with("première") {
        let num_part = input.strip_suffix("première")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::PremiereF));
    }
    if input.ends_with("seconds") {
        let num_part = input.strip_suffix("seconds")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::SecondsM));
    }
    if input.ends_with("second") {
        let num_part = input.strip_suffix("second")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::SecondM));
    }
    if input.ends_with("secondes") {
        let num_part = input.strip_suffix("secondes")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::SecondesF));
    }
    if input.ends_with("seconde") {
        let num_part = input.strip_suffix("seconde")?.trim_end_matches('-').trim();
        return Some((num_part.to_string(), OrdinalSuffix::SecondeF));
    }

    // Regular ordinals: ième/ièmes
    if input.ends_with("ièmes") {
        let stem = input.strip_suffix("ièmes")?.trim_end_matches('-').trim();
        let num_part = reconstruct_cardinal(stem)?;
        return Some((num_part, OrdinalSuffix::IemesPlural));
    }
    if input.ends_with("ième") {
        let stem = input.strip_suffix("ième")?.trim_end_matches('-').trim();
        let num_part = reconstruct_cardinal(stem)?;
        return Some((num_part, OrdinalSuffix::Ieme));
    }

    None
}

#[derive(Debug)]
enum OrdinalSuffix {
    PremierM,      // premier → Nᵉʳ
    PremiersM,     // premiers → Nᵉʳˢ
    PremiereF,     // première → Nʳᵉ
    PremieresF,    // premières → Nʳᵉˢ
    SecondM,       // second → Nᵈ
    SecondsM,      // seconds → Nᵈˢ
    SecondeF,      // seconde → Nᵈᵉ
    SecondesF,     // secondes → Nᵈᵉˢ
    Ieme,          // deuxième → Nᵉ
    IemesPlural,   // deuxièmes → Nᵉˢ
}

/// Format number with appropriate Unicode superscript suffix
fn format_ordinal(number: i64, suffix: &OrdinalSuffix) -> String {
    match suffix {
        OrdinalSuffix::PremierM => format!("{}ᵉʳ", number),
        OrdinalSuffix::PremiersM => format!("{}ᵉʳˢ", number),
        OrdinalSuffix::PremiereF => format!("{}ʳᵉ", number),
        OrdinalSuffix::PremieresF => format!("{}ʳᵉˢ", number),
        OrdinalSuffix::SecondM => format!("{}ᵈ", number),
        OrdinalSuffix::SecondsM => format!("{}ᵈˢ", number),
        OrdinalSuffix::SecondeF => format!("{}ᵈᵉ", number),
        OrdinalSuffix::SecondesF => format!("{}ᵈᵉˢ", number),
        OrdinalSuffix::Ieme => format!("{}ᵉ", number),
        OrdinalSuffix::IemesPlural => format!("{}ᵉˢ", number),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ordinals() {
        assert_eq!(parse("premier"), Some("1ᵉʳ".to_string()));
        assert_eq!(parse("première"), Some("1ʳᵉ".to_string()));
        assert_eq!(parse("deuxième"), Some("2ᵉ".to_string()));
        assert_eq!(parse("troisième"), Some("3ᵉ".to_string()));
    }

    #[test]
    fn test_compound_ordinals() {
        assert_eq!(parse("vingt et unième"), Some("21ᵉ".to_string()));
        assert_eq!(parse("cent onzième"), Some("111ᵉ".to_string()));
    }

    #[test]
    fn test_large_ordinals() {
        assert_eq!(parse("millième"), Some("1000ᵉ".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("vingt"), None);
    }
}
