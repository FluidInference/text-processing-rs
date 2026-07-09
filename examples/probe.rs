use std::fs;
use text_processing_rs::tn_normalize_sentence_lang;
fn main() {
    let path="/tmp/nemo-parity/tests/nemo_text_processing/en/data_text_normalization/test_cases_punctuation.txt";
    for line in fs::read_to_string(path).unwrap().lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let Some((i, e)) = l.split_once('~') else {
            continue;
        };
        let g = tn_normalize_sentence_lang(i, "en");
        if g != e {
            println!("[{}]\n  got  [{}]\n  want [{}]", i, g, e);
        }
    }
}
