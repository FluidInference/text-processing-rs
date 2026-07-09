use std::fs;
use text_processing_rs::tn_normalize_sentence_lang;
fn main() {
    let path="/tmp/nemo-parity/tests/nemo_text_processing/en/data_text_normalization/test_cases_address.txt";
    let (mut p, mut f) = (0, 0);
    for line in fs::read_to_string(path).unwrap().lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let Some((i, e)) = l.split_once('~') else {
            continue;
        };
        if tn_normalize_sentence_lang(i, "en") == e {
            p += 1;
        } else {
            f += 1;
            println!(
                "F [{}]\n  got  [{}]\n  want [{}]",
                i,
                tn_normalize_sentence_lang(i, "en"),
                e
            );
        }
    }
    eprintln!("address: {}/{}", p, p + f);
}
