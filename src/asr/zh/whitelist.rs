//! Whitelist tagger for Chinese ITN.
//!
//! Maps Chinese terms to their abbreviation/acronym forms:
//! - "人力资源" → "HR"
//! - "自动取款机" → "ATM"

/// Whitelist entries: (Chinese term, abbreviation)
const WHITELIST: &[(&str, &str)] = &[
    ("人力资源", "HR"),
    ("自动取款机", "ATM"),
    ("首席执行官", "CEO"),
    ("美国研究生入学考试", "GRE"),
    ("研究生管理专业入学考试", "GMAT"),
    ("全球定位系统", "GPS"),
    ("刷卡机", "POS机"),
    ("数位多功能光碟", "DVD"),
    ("镭射唱片", "CD"),
    ("通用串行总线", "USB"),
    ("统一资源定位符", "URL"),
    ("虚拟专用网络", "VPN"),
    ("网络互联协议", "IP"),
    ("脱氧核糖核酸", "DNA"),
    ("核糖核酸", "RNA"),
    ("平均学分绩点", "GPA"),
    ("发光二极管", "LED"),
    ("可移植文档格式", "PDF"),
    ("社会性网络服务", "SNS"),
    ("博士", "PhD"),
];

/// Process whitelist replacements in the input string.
pub fn process(input: &str) -> String {
    let mut result = input.to_string();
    // Apply longest matches first to avoid partial matches
    let mut sorted: Vec<&(&str, &str)> = WHITELIST.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for &(term, abbr) in &sorted {
        result = result.replace(term, abbr);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("人力资源"), "HR");
        assert_eq!(process("自动取款机"), "ATM");
        assert_eq!(process("博士"), "PhD");
    }
}
