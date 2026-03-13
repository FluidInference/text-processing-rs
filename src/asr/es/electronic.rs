//! Electronic tagger for Spanish.
//!
//! Converts spoken Spanish email/URL tokens to written form:
//! - "a b c arroba g mail punto com" → "abc@gmail.com"
//! - "hache te te pe ese dos puntos barra barra ..." → "https://..."

use super::cardinal;

/// Parse spoken Spanish electronic address to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    if !input_trim.contains("arroba")
        && !input_trim.contains("punto")
        && !input_trim.contains("barra")
    {
        return None;
    }

    let tokens: Vec<&str> = input_trim.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    let mut result = String::new();
    let mut i = 0;

    while i < tokens.len() {
        let t = tokens[i];

        // Multi-word tokens
        if t == "doble" && i + 1 < tokens.len() && tokens[i + 1] == "ve" {
            result.push('w');
            i += 2;
            continue;
        }
        if t == "dos" && i + 1 < tokens.len() && tokens[i + 1] == "puntos" {
            result.push(':');
            i += 2;
            continue;
        }
        if t == "signo"
            && i + 2 < tokens.len()
            && tokens[i + 1] == "de"
            && tokens[i + 2] == "interrogación"
        {
            result.push('?');
            i += 3;
            continue;
        }
        if t == "signo" && i + 1 < tokens.len() && tokens[i + 1] == "igual" {
            result.push('=');
            i += 2;
            continue;
        }

        // Single-word special tokens
        match t {
            "arroba" => result.push('@'),
            "punto" => result.push('.'),
            "barra" => result.push('/'),
            "guion" | "guión" => result.push('-'),
            "hache" => result.push('h'),
            "te" => result.push('t'),
            "pe" => result.push('p'),
            "ese" => result.push('s'),
            "efe" => result.push('f'),
            "ene" => result.push('n'),
            "eme" => result.push('m'),
            "ele" => result.push('l'),
            "ere" => result.push('r'),
            "ce" => result.push('c'),
            "de" => result.push('d'),
            "ge" => result.push('g'),
            "jota" => result.push('j'),
            "ka" => result.push('k'),
            "cu" => result.push('q'),
            "equis" => result.push('x'),
            "ye" | "i griega" => result.push('y'),
            "zeta" => result.push('z'),
            _ => {
                // Single letter (a-z)
                if t.len() == 1 && t.chars().all(|c| c.is_ascii_alphabetic()) {
                    result.push_str(t);
                }
                // Digit word
                else if let Some(digit) = cardinal::word_to_digit(t) {
                    result.push_str(&digit.to_string());
                }
                // Multi-char word that's not a special token → append as-is
                else if t.len() > 1 {
                    // Could be a domain part like "gmail", "nvidia", "com", "edu", "gob"
                    result.push_str(t);
                }
            }
        }

        i += 1;
    }

    if result.is_empty() || result == input_trim {
        return None;
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert_eq!(
            parse("a b c arroba g mail punto com"),
            Some("abc@gmail.com".to_string())
        );
    }

    #[test]
    fn test_url() {
        assert_eq!(
            parse("doble ve doble ve doble ve punto n vidia punto com"),
            Some("www.nvidia.com".to_string())
        );
    }
}
