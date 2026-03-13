//! Money tagger for Spanish.
//!
//! Converts spoken Spanish currency expressions to written form:
//! - "doce dólares y cinco centavos" → "$12,05"
//! - "veinticinco céntimos" → "€0,25"
//! - "diez pesetas" → "₧10"

use super::cardinal;

struct Currency {
    names: &'static [&'static str],
    symbol: &'static str,
    cent_names: &'static [&'static str],
}

const CURRENCIES: &[Currency] = &[
    Currency {
        names: &["dólares estadounidenses", "dólares americanos"],
        symbol: "US$",
        cent_names: &["centavos", "centavo"],
    },
    Currency {
        names: &["pesos mexicanos", "peso mexicano"],
        symbol: "Mex$",
        cent_names: &["centavos", "centavo"],
    },
    Currency {
        names: &["dólar", "dólares"],
        symbol: "$",
        cent_names: &["centavos", "centavo", "céntimos", "céntimo"],
    },
    Currency {
        names: &["euro", "euros"],
        symbol: "€",
        cent_names: &["centavos", "centavo", "céntimos", "céntimo"],
    },
    Currency {
        names: &["peso", "pesos"],
        symbol: "$",
        cent_names: &["centavos", "centavo"],
    },
    Currency {
        names: &["yen", "yenes"],
        symbol: "¥",
        cent_names: &["centavos", "centavo"],
    },
    Currency {
        names: &["peseta", "pesetas"],
        symbol: "₧",
        cent_names: &[],
    },
    Currency {
        names: &["colón", "colones"],
        symbol: "₡",
        cent_names: &[],
    },
    Currency {
        names: &["won", "wones"],
        symbol: "₩",
        cent_names: &["chon", "chones"],
    },
    Currency {
        names: &["quetzal", "quetzales"],
        symbol: "q",
        cent_names: &[],
    },
];

/// Parse spoken Spanish money expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    if !input_trim.contains(' ') {
        return None;
    }

    // Try "dos dólares y sesenta y tres dólares" → "$2 y $63" (two amounts)
    if let Some(result) = parse_two_amounts(input_trim) {
        return Some(result);
    }

    // Try scale money: "nueve punto cinco millones de pesos" → "$9.5 millones"
    if let Some(result) = parse_scale_money(input_trim) {
        return Some(result);
    }

    // Try full scale: "catorce millones quinientos mil pesos mexicanos" → "Mex$14500000"
    if let Some(result) = parse_full_scale_money(input_trim) {
        return Some(result);
    }

    // Try "X CURRENCY y/con Y centavos"
    if let Some(result) = parse_with_subcurrency(input_trim) {
        return Some(result);
    }

    // Try "X CURRENCY Y [centavos]" (implied or explicit cents)
    if let Some(result) = parse_implied_cents(input_trim) {
        return Some(result);
    }

    // Try "X CURRENCY con Y"
    if let Some(result) = parse_con_amount(input_trim) {
        return Some(result);
    }

    // Try simple: "un dólar" → "$1"
    if let Some(result) = parse_simple(input_trim) {
        return Some(result);
    }

    // Try cent-only: "veinticinco centavos" → "$0,25"
    if let Some(result) = parse_cents_only(input_trim) {
        return Some(result);
    }

    // Try chon: "un chon" → "₩0,01"
    if let Some(result) = parse_subunit_only(input_trim) {
        return Some(result);
    }

    None
}

/// Parse two separate amounts: "dos dólares y sesenta y tres dólares"
fn parse_two_amounts(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &name in cur.names {
            // Look for "X NAME y ... NAME" pattern
            let pattern = format!(" {} y ", name);
            if let Some(pos) = input.find(&pattern) {
                let first_part = &input[..pos];
                let second_part = &input[pos + pattern.len()..];

                // Second part should end with same currency
                if second_part.ends_with(name) {
                    let second_num = second_part[..second_part.len() - name.len()].trim();
                    let first_val = cardinal::words_to_number(first_part)?;
                    let second_val = cardinal::words_to_number(second_num)?;
                    return Some(format!(
                        "{}{} y {}{}",
                        cur.symbol, first_val, cur.symbol, second_val
                    ));
                }
            }
        }
    }
    None
}

/// Parse scale money: "nueve punto cinco millones de pesos" → "$9.5 millones"
fn parse_scale_money(input: &str) -> Option<String> {
    let scale_words = ["millones", "millón", "billones", "billón"];

    for cur in CURRENCIES {
        for &name in cur.names {
            // Check for "de CURRENCY" at end
            let de_pattern = format!("de {}", name);
            if input.ends_with(&de_pattern) {
                let before = input[..input.len() - de_pattern.len()].trim();
                // Check for scale word
                for &sw in &scale_words {
                    if before.ends_with(sw) {
                        let num_part = before[..before.len() - sw.len()].trim();
                        // Try "punto" decimal
                        if num_part.contains(" punto ") {
                            let parts: Vec<&str> = num_part.splitn(2, " punto ").collect();
                            let int_val = cardinal::words_to_number(parts[0].trim())?;
                            let dec_digits = parse_decimal_digits(parts[1].trim())?;
                            return Some(format!(
                                "{}{}.{} {}",
                                cur.symbol, int_val, dec_digits, sw
                            ));
                        }
                        let num = cardinal::words_to_number(num_part)?;
                        return Some(format!("{}{} {}", cur.symbol, num, sw));
                    }
                }
            }
        }
    }
    None
}

/// Parse full-scale money: "catorce millones quinientos mil pesos mexicanos" → "Mex$14500000"
fn parse_full_scale_money(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &name in cur.names {
            if input.ends_with(name) {
                let before = input[..input.len() - name.len()].trim();
                if before.is_empty() {
                    continue;
                }
                // Must contain a scale word to be full-scale
                let has_scale = ["millones", "millón", "mil", "billones", "billón"]
                    .iter()
                    .any(|&sw| before.contains(sw));
                if !has_scale {
                    continue;
                }
                let num = cardinal::words_to_number(before)?;
                if num >= 1000 {
                    return Some(format!("{}{}", cur.symbol, num));
                }
            }
        }
    }
    None
}

/// Parse with subcurrency: "doce dólares y cinco centavos" → "$12,05"
fn parse_with_subcurrency(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cent_name in cur.cent_names {
            if !input.ends_with(cent_name) {
                continue;
            }
            let before_cent = input[..input.len() - cent_name.len()].trim();

            // Try "X CURRENCY y Y"
            for &cur_name in cur.names {
                // "y" separator
                let y_pattern = format!("{} y ", cur_name);
                if let Some(pos) = before_cent.find(&y_pattern) {
                    let main_part = &before_cent[..pos];
                    let cent_part = &before_cent[pos + y_pattern.len()..];

                    let main_val = cardinal::words_to_number(main_part)?;
                    let cent_val = cardinal::words_to_number(cent_part.trim())?;

                    return Some(format!("{}{},{:02}", cur.symbol, main_val, cent_val));
                }

                // "con" separator
                let con_pattern = format!("{} con ", cur_name);
                if let Some(pos) = before_cent.find(&con_pattern) {
                    let main_part = &before_cent[..pos];
                    let cent_part = &before_cent[pos + con_pattern.len()..];

                    let main_val = cardinal::words_to_number(main_part)?;
                    let cent_val = cardinal::words_to_number(cent_part.trim())?;

                    return Some(format!("{}{},{:02}", cur.symbol, main_val, cent_val));
                }

                // No separator: "veintinueve dólares cincuenta centavos"
                let space_pattern = format!("{} ", cur_name);
                if let Some(pos) = before_cent.find(&space_pattern) {
                    let main_part = &before_cent[..pos];
                    let cent_part = &before_cent[pos + space_pattern.len()..];

                    let main_val = cardinal::words_to_number(main_part)?;
                    let cent_val = cardinal::words_to_number(cent_part.trim())?;

                    return Some(format!("{}{},{:02}", cur.symbol, main_val, cent_val));
                }
            }
        }
    }
    None
}

/// Parse implied cents: "setenta y cinco dólares sesenta y tres" → "$75,63"
fn parse_implied_cents(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cur_name in cur.names {
            let pattern = format!(" {} ", cur_name);
            if let Some(pos) = input.find(&pattern) {
                let main_part = &input[..pos];
                let cent_part = &input[pos + pattern.len()..];

                // cent_part should not end with a currency name
                let is_subcurrency = cur.cent_names.iter().any(|&c| cent_part.ends_with(c));
                if is_subcurrency {
                    continue;
                }

                let main_val = cardinal::words_to_number(main_part)?;
                let cent_val = cardinal::words_to_number(cent_part)?;

                return Some(format!("{}{},{:02}", cur.symbol, main_val, cent_val));
            }
        }
    }
    None
}

/// Parse "X CURRENCY con Y"
fn parse_con_amount(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cur_name in cur.names {
            let pattern = format!(" {} con ", cur_name);
            if let Some(pos) = input.find(&pattern) {
                let main_part = &input[..pos];
                let cent_part = &input[pos + pattern.len()..];

                let main_val = cardinal::words_to_number(main_part)?;
                let cent_val = cardinal::words_to_number(cent_part)?;

                return Some(format!("{}{},{:02}", cur.symbol, main_val, cent_val));
            }
        }
    }
    None
}

/// Parse simple: "un dólar" → "$1"
fn parse_simple(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cur_name in cur.names {
            if input.ends_with(cur_name) {
                let before = input[..input.len() - cur_name.len()].trim();
                if before.is_empty() {
                    continue;
                }
                let num = cardinal::words_to_number(before)?;
                return Some(format!("{}{}", cur.symbol, num));
            }
        }
    }
    None
}

/// Parse cents-only: "veinticinco centavos" → "$0,25"
fn parse_cents_only(input: &str) -> Option<String> {
    // "centavos" defaults to dollar
    for &cent_name in &["centavos", "centavo"] {
        if input.ends_with(cent_name) {
            let before = input[..input.len() - cent_name.len()].trim();
            if before.is_empty() {
                continue;
            }
            let num = cardinal::words_to_number(before)?;
            return Some(format!("$0,{:02}", num));
        }
    }
    // "céntimos" defaults to euro
    for &cent_name in &["céntimos", "céntimo"] {
        if input.ends_with(cent_name) {
            let before = input[..input.len() - cent_name.len()].trim();
            if before.is_empty() {
                continue;
            }
            let num = cardinal::words_to_number(before)?;
            return Some(format!("€0,{:02}", num));
        }
    }
    None
}

/// Parse subunit-only: "un chon" → "₩0,01"
fn parse_subunit_only(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cent_name in cur.cent_names {
            if input.ends_with(cent_name) {
                let before = input[..input.len() - cent_name.len()].trim();
                if before.is_empty() {
                    continue;
                }
                let num = cardinal::words_to_number(before)?;
                return Some(format!("{}0,{:02}", cur.symbol, num));
            }
        }
    }
    None
}

/// Parse decimal digits
fn parse_decimal_digits(input: &str) -> Option<String> {
    let digit_map = [
        ("cero", "0"),
        ("uno", "1"),
        ("un", "1"),
        ("dos", "2"),
        ("tres", "3"),
        ("cuatro", "4"),
        ("cinco", "5"),
        ("seis", "6"),
        ("siete", "7"),
        ("ocho", "8"),
        ("nueve", "9"),
    ];

    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();
    for token in &tokens {
        let mut found = false;
        for &(word, digit) in &digit_map {
            if token == &word {
                result.push_str(digit);
                found = true;
                break;
            }
        }
        if !found {
            // Try as a compound number
            if let Some(num) = cardinal::words_to_number(token) {
                result.push_str(&num.to_string());
            } else {
                return None;
            }
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(parse("un dólar"), Some("$1".to_string()));
    }

    #[test]
    fn test_with_cents() {
        assert_eq!(
            parse("doce dólares y cinco centavos"),
            Some("$12,05".to_string())
        );
    }

    #[test]
    fn test_centimos() {
        assert_eq!(parse("veinticinco céntimos"), Some("€0,25".to_string()));
    }

    #[test]
    fn test_pesetas() {
        assert_eq!(parse("diez pesetas"), Some("₧10".to_string()));
    }
}
