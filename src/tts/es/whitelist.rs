//! Whitelist TN tagger for Spanish.
//!
//! Lookup table for common Spanish abbreviations and special terms:
//! - "Dr." → "doctor"
//! - "Sr." → "senor"
//! - "Ud." → "usted"
//! - "p.ej." → "por ejemplo"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("Dr.", "doctor");
        m.insert("Dr", "doctor");
        m.insert("Dra.", "doctora");
        m.insert("Dra", "doctora");
        m.insert("Sr.", "senor");
        m.insert("Sr", "senor");
        m.insert("Sra.", "senora");
        m.insert("Sra", "senora");
        m.insert("Srta.", "senorita");
        m.insert("Srta", "senorita");
        m.insert("Prof.", "profesor");
        m.insert("Prof", "profesor");
        m.insert("Profa.", "profesora");
        m.insert("Profa", "profesora");

        // Formal pronouns
        m.insert("Ud.", "usted");
        m.insert("Ud", "usted");
        m.insert("Uds.", "ustedes");
        m.insert("Uds", "ustedes");

        // Common abbreviations
        m.insert("etc.", "etcetera");
        m.insert("p.ej.", "por ejemplo");

        // Address abbreviations
        m.insert("Av.", "avenida");
        m.insert("Av", "avenida");
        m.insert("Blvd.", "bulevar");
        m.insert("Blvd", "bulevar");
        m.insert("Col.", "colonia");
        m.insert("Col", "colonia");

        // Organizational
        m.insert("Dept.", "departamento");
        m.insert("Dept", "departamento");
        m.insert("No.", "numero");
        m.insert("No", "numero");
        m.insert("Cia.", "compania");
        m.insert("Cia", "compania");
        m.insert("Ltda.", "limitada");
        m.insert("Ltda", "limitada");
        m.insert("S.A.", "sociedad anonima");

        // Military / professional titles
        m.insert("Gral.", "general");
        m.insert("Gral", "general");
        m.insert("Ing.", "ingeniero");
        m.insert("Ing", "ingeniero");
        m.insert("Lic.", "licenciado");
        m.insert("Lic", "licenciado");
        m.insert("Arq.", "arquitecto");
        m.insert("Arq", "arquitecto");

        m
    };
}

/// Parse a whitelist abbreviation to its spoken Spanish form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Direct lookup (case-sensitive first)
    if let Some(&spoken) = WHITELIST.get(trimmed) {
        return Some(spoken.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titles() {
        assert_eq!(parse("Dr."), Some("doctor".to_string()));
        assert_eq!(parse("Dra."), Some("doctora".to_string()));
        assert_eq!(parse("Sr."), Some("senor".to_string()));
        assert_eq!(parse("Sra."), Some("senora".to_string()));
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse("etc."), Some("etcetera".to_string()));
        assert_eq!(parse("p.ej."), Some("por ejemplo".to_string()));
        assert_eq!(parse("Ud."), Some("usted".to_string()));
        assert_eq!(parse("S.A."), Some("sociedad anonima".to_string()));
    }

    #[test]
    fn test_professional_titles() {
        assert_eq!(parse("Ing."), Some("ingeniero".to_string()));
        assert_eq!(parse("Lic."), Some("licenciado".to_string()));
        assert_eq!(parse("Arq."), Some("arquitecto".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hola"), None);
        assert_eq!(parse("mundo"), None);
    }
}
