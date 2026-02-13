//! Common test utilities for parsing NeMo-format test files.

use std::fs;
use std::path::Path;

/// Parse NeMo-format test file (input~expected per line).
///
/// Lines starting with # are comments.
/// Empty lines are skipped.
pub fn parse_test_file(path: &Path) -> Vec<(String, String)> {
    let content = fs::read_to_string(path).expect(&format!("Failed to read test file: {:?}", path));
    content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, '~').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Run all test cases from a file and return (passed, failed, total).
pub fn run_test_file<F>(path: &Path, normalize_fn: F) -> TestResults
where
    F: Fn(&str) -> String,
{
    let cases = parse_test_file(path);
    let mut results = TestResults::new(cases.len());

    for (input, expected) in &cases {
        let result = normalize_fn(input);
        if result == *expected {
            results.passed += 1;
        } else {
            results.failures.push(TestFailure {
                input: input.clone(),
                expected: expected.clone(),
                got: result,
            });
        }
    }

    results
}

/// Assert all tests pass, panic with details if any fail.
pub fn assert_test_file<F>(path: &Path, normalize_fn: F)
where
    F: Fn(&str) -> String,
{
    let results = run_test_file(path, normalize_fn);

    if !results.failures.is_empty() {
        eprintln!("\n{} failures:\n", results.failures.len());
        for failure in &results.failures {
            eprintln!(
                "  {:?}\n    expected: {:?}\n    got:      {:?}\n",
                failure.input, failure.expected, failure.got
            );
        }
        panic!(
            "{}/{} tests failed in {:?}",
            results.failures.len(),
            results.total,
            path
        );
    }
}

#[derive(Debug)]
pub struct TestResults {
    pub total: usize,
    pub passed: usize,
    pub failures: Vec<TestFailure>,
}

impl TestResults {
    fn new(total: usize) -> Self {
        Self {
            total,
            passed: 0,
            failures: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TestFailure {
    pub input: String,
    pub expected: String,
    pub got: String,
}
