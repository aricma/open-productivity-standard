//! Generates `ops/tests/acceptance/**` — one `#[test]` per case under
//! `specs/<version>/tests/` — so the suite is committed, reviewable, and
//! mirrors the corpus layout.
//!
//! The corpus is grouped by format, and each format by kind: `valid/`,
//! `invalid/`, `roundtrip/`. `transform/` is the one kind that crosses formats.
//! The test name is the case directory name.
//!
//! Formats the reference implementation does not claim become `#[ignore]`d
//! tests, so skips stay visible and counted.
//!
//! ```text
//! cargo run -p gen-corpus             regenerate, removing stale files
//! cargo run -p gen-corpus -- --check  fail if the committed files differ
//! ```
//!
//! The generator owns `ops/tests/acceptance/`: it writes every file it plans
//! and removes any other `.rs` in that tree.
//!
//! - [`case`] is what a case is, and what it asserts.
//! - [`constants`] is the tool's fixed knowledge about the repository and the
//!   corpus interface.
//! - [`corpus`] reads the corpus from disk, and refuses to read it wrongly.
//! - [`emission`] turns a case into Rust source.
//! - [`plan`] is the whole tree the output should contain, and where it lives.
//! - [`output`] writes that tree, or verifies it is current.
//! - [`formats`] is the one place that knows both format vocabularies.

mod case;
mod constants;
mod corpus;
mod emission;
mod formats;
mod output;
mod plan;

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut verify_only = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--check" => verify_only = true,
            other => {
                eprintln!("usage: gen-corpus [--check] (unknown argument: {other})");
                return ExitCode::from(2);
            }
        }
    }

    let repository = plan::repository_root();
    let out = plan::acceptance_dir(&repository);
    let files = plan::generated_files(&repository, &out);
    if verify_only {
        output::verify_tree(&out, &files)
    } else {
        output::write_tree(&out, &files)
    }
}
