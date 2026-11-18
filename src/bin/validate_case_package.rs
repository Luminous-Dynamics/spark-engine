use std::env;
use std::fs;
use std::process::ExitCode;

use spark_engine::case_package::{
    validate_case_manifest_str, CASE_PACKAGE_SEMANTIC_VALIDATOR_REVISION,
};

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(path) = args.next() else {
        eprintln!(
            "usage: {} <case-manifest.json>",
            std::path::Path::new(&program)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("spark-case-validate")
        );
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("error: exactly one manifest path is required");
        return ExitCode::from(2);
    }
    let path = std::path::PathBuf::from(path);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", path.display());
            return ExitCode::from(2);
        }
    };
    match validate_case_manifest_str(&contents) {
        Ok(summary) => {
            println!(
                "semantic validation passed: case={} family={} models={} boundaries={} flows={} evidence={} accounting_checks={}",
                summary.case_id, summary.model_family, summary.model_count,
                summary.boundary_count, summary.flow_count, summary.evidence_count,
                summary.accounting_check_count
            );
            println!("validator_revision={CASE_PACKAGE_SEMANTIC_VALIDATOR_REVISION}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            eprintln!("semantic validation failed ({} issue(s)):", errors.len());
            for error in errors { eprintln!(" - {error}"); }
            ExitCode::from(1)
        }
    }
}
