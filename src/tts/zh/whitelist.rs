//! Whitelist TN tagger for Mandarin Chinese (pinyin output).
//!
//! Lookup table for common abbreviations with pinyin spoken forms:
//! - "Dr." -> "boshi"
//! - "Mr." -> "xiansheng"
//! - "Mrs." -> "taitai"
//! - "etc." -> "deng deng"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("Dr.", "boshi");
        m.insert("Dr", "boshi");
        m.insert("Mr.", "xiansheng");
        m.insert("Mr", "xiansheng");
        m.insert("Mrs.", "taitai");
        m.insert("Mrs", "taitai");
        m.insert("Ms.", "nvshi");
        m.insert("Ms", "nvshi");
        m.insert("Prof.", "jiaoshou");
        m.insert("St.", "sheng");
        m.insert("Jr.", "xiao");
        m.insert("Sr.", "lao");

        // Common abbreviations
        m.insert("etc.", "deng deng");
        m.insert("vs.", "dui");
        m.insert("vs", "dui");
        m.insert("No.", "hao");

        // Business terms
        m.insert("Inc.", "gongsi");
        m.insert("Ltd.", "youxian gongsi");
        m.insert("Co.", "gongsi");

        m
    };
}

/// Parse a whitelist abbreviation to its spoken form in Mandarin Chinese pinyin.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Direct lookup (case-sensitive)
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
        assert_eq!(parse("Dr."), Some("boshi".to_string()));
        assert_eq!(parse("Mr."), Some("xiansheng".to_string()));
        assert_eq!(parse("Mrs."), Some("taitai".to_string()));
        assert_eq!(parse("Ms."), Some("nvshi".to_string()));
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse("etc."), Some("deng deng".to_string()));
        assert_eq!(parse("vs."), Some("dui".to_string()));
        assert_eq!(parse("No."), Some("hao".to_string()));
    }

    #[test]
    fn test_business() {
        assert_eq!(parse("Inc."), Some("gongsi".to_string()));
        assert_eq!(parse("Ltd."), Some("youxian gongsi".to_string()));
        assert_eq!(parse("Co."), Some("gongsi".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("world"), None);
    }
}
