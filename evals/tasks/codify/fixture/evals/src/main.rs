mod lint;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("lint") {
        eprintln!("usage: evals lint [repo-root]");
        return ExitCode::from(2);
    }
    let root = args.next().map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."), PathBuf::from);
    let errors = lint::lint(&root);
    for e in &errors {
        println!("{e}");
    }
    if errors.is_empty() {
        println!("lint: ok");
        ExitCode::SUCCESS
    } else {
        println!("lint: {} error(s)", errors.len());
        ExitCode::FAILURE
    }
}
