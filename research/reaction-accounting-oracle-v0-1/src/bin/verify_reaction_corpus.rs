//! Independent structural check of the frozen oracle corpus.
//! This binary never calls account_channel or account_dd_total.

use std::collections::BTreeSet;
use std::process::ExitCode;

const CORPUS: &str = include_str!("../../fixtures/reaction-accounting-cases-v1.tsv");
const REQUIRED: [&str; 24] = [
    "dd-neutron-known-answer",
    "dd-proton-known-answer",
    "dd-branch-total",
    "dt-unlike-reactants",
    "dhe3-unlike-reactants",
    "pb11-unlike-reactants",
    "he3he3-identical-reactants",
    "unlike-density-linear-scaling",
    "identical-density-quadratic-scaling",
    "branch-specific-q-not-average",
    "branch-sum-not-double-fractioned",
    "universal-quarter-factor-control",
    "negative-density",
    "zero-density",
    "nan-density",
    "infinite-density",
    "zero-reactivity",
    "negative-reactivity",
    "nan-reactivity",
    "zero-q-value",
    "negative-q-value",
    "nan-q-value",
    "wrong-reactant-species",
    "wrong-density-shape",
];

fn main() -> ExitCode {
    match audit() {
        Ok((accept, reject)) => {
            println!("PASS: {} rows; ACCEPT={accept}, REJECT={reject}", REQUIRED.len());
            println!("SCOPE: corpus inventory only; oracle behavior and physical correctness are not evaluated");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("FAIL: {error}");
            ExitCode::FAILURE
        }
    }
}

fn audit() -> Result<(usize, usize), String> {
    let mut lines = CORPUS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    if lines.next() != Some("case_id|expected|mutation") {
        return Err("missing or incorrect header".to_string());
    }
    let mut ids = BTreeSet::new();
    let mut accept = 0;
    let mut reject = 0;
    for line in lines {
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() != 3 {
            return Err(format!("expected three fields: {line}"));
        }
        let [id, expected, mutation] = [fields[0], fields[1], fields[2]];
        if id.is_empty() || mutation.is_empty() || !ids.insert(id) {
            return Err(format!("empty/duplicate identity or mutation: {line}"));
        }
        match expected {
            "ACCEPT" => accept += 1,
            "REJECT" => reject += 1,
            other => return Err(format!("invalid disposition {other} for {id}")),
        }
    }
    if ids.len() != REQUIRED.len() {
        return Err(format!("expected {} rows, found {}", REQUIRED.len(), ids.len()));
    }
    for id in REQUIRED {
        if !ids.contains(id) {
            return Err(format!("required fixture absent: {id}"));
        }
    }
    if accept != 12 || reject != 12 {
        return Err(format!("unexpected disposition counts: ACCEPT={accept}, REJECT={reject}"));
    }
    Ok((accept, reject))
}
