//! NeMo parity reporter / drift guard.
//!
//! Runs the port against NVIDIA/NeMo-text-processing's *full* upstream test
//! files (not the curated `tests/data/**` subsets) and reports the true
//! pass-rate per language / direction / class.
//!
//! Usage:
//!   NEMO_DIR=/path/to/NeMo-text-processing cargo run --release --example parity
//!   NEMO_DIR=... cargo run --release --example parity -- --write-baseline
//!
//! `NEMO_DIR` must contain `tests/nemo_text_processing/<lang>/`. In check mode
//! (the default) the tool compares against `tests/parity_baseline.tsv` and
//! exits non-zero if any class *regressed* (fewer passing cases than the
//! baseline) — improvements are fine and just prompt a baseline refresh.

use std::collections::BTreeMap;
use std::fs;
use std::panic;
use std::path::Path;
use std::process::exit;

use text_processing_rs::{normalize_with_lang, tn_normalize_lang};

const LANGS: &[&str] = &["en", "de", "es", "fr", "hi", "ja", "zh"];
const BASELINE_PATH: &str = "tests/parity_baseline.tsv";

/// (pass, total, panics) for one class file.
#[derive(Clone, Copy, Default)]
struct Stat {
    pass: usize,
    total: usize,
    panics: usize,
}

fn main() {
    // Keep panic output quiet — we catch and count them per case.
    panic::set_hook(Box::new(|_| {}));

    let write_baseline = std::env::args().any(|a| a == "--write-baseline");
    let nemo_dir = std::env::var("NEMO_DIR").unwrap_or_else(|_| {
        eprintln!("NEMO_DIR is not set (path to a NeMo-text-processing checkout)");
        exit(2);
    });

    // key: "lang\tdir\tclass" -> Stat
    let mut stats: BTreeMap<String, Stat> = BTreeMap::new();

    for &lang in LANGS {
        for (dir, subdir, is_tn) in [
            ("itn", "data_inverse_text_normalization", false),
            ("tn", "data_text_normalization", true),
        ] {
            let base = format!(
                "{}/tests/nemo_text_processing/{}/{}",
                nemo_dir, lang, subdir
            );
            let entries = match fs::read_dir(&base) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|x| x == "txt").unwrap_or(false))
                .collect();
            paths.sort();

            for path in paths {
                let class = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.trim_start_matches("test_cases_").to_string())
                    .unwrap_or_default();
                let stat = score_file(&path, lang, is_tn);
                if stat.total > 0 {
                    stats.insert(format!("{}\t{}\t{}", lang, dir, class), stat);
                }
            }
        }
    }

    if write_baseline {
        write_baseline_file(&stats);
        return;
    }

    print_report(&stats);
    let regressions = check_against_baseline(&stats);
    if !regressions.is_empty() {
        eprintln!(
            "\n❌ NeMo parity REGRESSED in {} class(es):",
            regressions.len()
        );
        for (key, base_pass, cur_pass) in &regressions {
            eprintln!(
                "   {}  {} -> {}",
                key.replace('\t', " "),
                base_pass,
                cur_pass
            );
        }
        exit(1);
    }
    println!("\n✅ No parity regressions vs baseline.");
}

/// Run every `input~expected` line through the port, counting matches. Each
/// call is isolated with `catch_unwind` so a panicking input is counted as a
/// failure instead of aborting the run.
fn score_file(path: &Path, lang: &str, is_tn: bool) -> Stat {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut stat = Stat::default();
    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '~');
        let (Some(input), Some(expected)) = (parts.next(), parts.next()) else {
            continue;
        };
        stat.total += 1;
        let input = input.to_string();
        let lang = lang.to_string();
        let got = panic::catch_unwind(|| {
            if is_tn {
                tn_normalize_lang(&input, &lang)
            } else {
                normalize_with_lang(&input, &lang)
            }
        });
        match got {
            Ok(g) if g == expected => stat.pass += 1,
            Ok(_) => {}
            Err(_) => stat.panics += 1,
        }
    }
    stat
}

fn print_report(stats: &BTreeMap<String, Stat>) {
    println!("## NeMo parity (port vs full upstream test files)\n");
    println!("| lang | dir | class | pass/total | panics |");
    println!("|------|-----|-------|-----------|--------|");
    let mut totals: BTreeMap<String, Stat> = BTreeMap::new();
    for (key, s) in stats {
        let mut it = key.split('\t');
        let (lang, dir, class) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
        let panic_note = if s.panics > 0 {
            format!("{}", s.panics)
        } else {
            String::new()
        };
        println!(
            "| {} | {} | {} | {}/{} | {} |",
            lang, dir, class, s.pass, s.total, panic_note
        );
        let t = totals.entry(format!("{}\t{}", lang, dir)).or_default();
        t.pass += s.pass;
        t.total += s.total;
        t.panics += s.panics;
    }
    println!("\n### Totals by language / direction\n");
    println!("| lang | dir | pass/total | % | panics |");
    println!("|------|-----|-----------|---|--------|");
    for (key, t) in &totals {
        let pct = if t.total > 0 {
            100.0 * t.pass as f64 / t.total as f64
        } else {
            0.0
        };
        println!(
            "| {} | {} | {}/{} | {:.0}% | {} |",
            key.split('\t').next().unwrap(),
            key.split('\t').nth(1).unwrap(),
            t.pass,
            t.total,
            pct,
            if t.panics > 0 {
                t.panics.to_string()
            } else {
                String::new()
            }
        );
    }
}

fn write_baseline_file(stats: &BTreeMap<String, Stat>) {
    let mut out = String::from("# NeMo parity baseline: lang\\tdir\\tclass\\tpass\\ttotal\n");
    out.push_str("# Regenerate with: NEMO_DIR=... cargo run --release --example parity -- --write-baseline\n");
    for (key, s) in stats {
        out.push_str(&format!("{}\t{}\t{}\n", key, s.pass, s.total));
    }
    fs::write(BASELINE_PATH, out).expect("write baseline");
    println!("Wrote {} ({} classes).", BASELINE_PATH, stats.len());
}

/// Returns (key, baseline_pass, current_pass) for every class whose passing
/// count dropped below the committed baseline.
fn check_against_baseline(stats: &BTreeMap<String, Stat>) -> Vec<(String, usize, usize)> {
    let content = match fs::read_to_string(BASELINE_PATH) {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "(no baseline at {} — skipping regression check)",
                BASELINE_PATH
            );
            return Vec::new();
        }
    };
    let mut regressions = Vec::new();
    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 5 {
            continue;
        }
        let key = format!("{}\t{}\t{}", cols[0], cols[1], cols[2]);
        let base_pass: usize = cols[3].parse().unwrap_or(0);
        let cur_pass = stats.get(&key).map(|s| s.pass).unwrap_or(0);
        if cur_pass < base_pass {
            regressions.push((key, base_pass, cur_pass));
        }
    }
    regressions
}
