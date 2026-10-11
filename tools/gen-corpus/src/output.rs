//! The generated tree on disk: writing it, or verifying it is current.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Write every planned file, and remove any `.rs` in the tree that is no longer
/// planned, so the tree on disk is exactly the plan and nothing else.
pub(crate) fn write_tree(out: &Path, files: &[(String, String)]) -> ExitCode {
    let mut planned = HashSet::new();
    for (relative, contents) in files {
        let path = out.join(relative);
        if let Some(parent) = path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            eprintln!("mkdir {}: {e}", parent.display());
            return ExitCode::FAILURE;
        }
        if let Err(e) = fs::write(&path, contents) {
            eprintln!("write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        planned.insert(path);
    }
    for path in rust_files(out) {
        if !planned.contains(&path) {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("remove {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
            println!("removed stale {}", relative_to(out, &path));
        }
    }
    println!("wrote {} generated files", files.len());
    ExitCode::SUCCESS
}

/// Report every planned file that is missing or out of date, and every `.rs` in
/// the tree that is no longer planned. Writes nothing.
pub(crate) fn verify_tree(out: &Path, files: &[(String, String)]) -> ExitCode {
    let mut problems = Vec::new();
    let mut planned = HashSet::new();
    for (relative, contents) in files {
        let path = out.join(relative);
        planned.insert(path.clone());
        match fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == *contents => {}
            Ok(_) => problems.push(format!("out of date: {relative}")),
            Err(_) => problems.push(format!("missing:     {relative}")),
        }
    }
    for path in rust_files(out) {
        if !planned.contains(&path) {
            problems.push(format!("stale:       {}", relative_to(out, &path)));
        }
    }
    if problems.is_empty() {
        println!("generated tests are up to date");
        return ExitCode::SUCCESS;
    }
    for problem in &problems {
        eprintln!("{problem}");
    }
    eprintln!("run `cargo run -p gen-corpus` to regenerate");
    ExitCode::FAILURE
}

/// Every `.rs` under `dir`, recursively.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(rust_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

/// A path as the reader sees it, relative to the output root.
fn relative_to(out: &Path, path: &Path) -> String {
    path.strip_prefix(out).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// A scratch output tree, emptied of any earlier run.
    fn scratch(name: &str) -> PathBuf {
        let out = std::env::temp_dir().join(format!("gen-corpus-out-{name}"));
        let _ = fs::remove_dir_all(&out);
        fs::create_dir_all(&out).unwrap();
        out
    }

    fn files() -> Vec<(String, String)> {
        vec![
            ("main.rs".to_string(), "//! main\n".to_string()),
            ("v0/mod.rs".to_string(), "//! v0\n".to_string()),
        ]
    }

    /// Writing the plan twice is writing the same tree, and both times it
    /// reports that the tree is current.
    #[test]
    fn writing_then_verifying_is_current() {
        let out = scratch("write-then-verify");
        assert_eq!(write_tree(&out, &files()), ExitCode::SUCCESS);
        assert_eq!(verify_tree(&out, &files()), ExitCode::SUCCESS);
    }

    /// A file left over from an older layout is removed, not kept.
    #[test]
    fn writing_removes_a_stale_file() {
        let out = scratch("stale");
        fs::create_dir_all(out.join("old")).unwrap();
        fs::write(out.join("old/left.rs"), "//! left over\n").unwrap();

        assert_eq!(write_tree(&out, &files()), ExitCode::SUCCESS);
        assert!(!out.join("old/left.rs").exists());
        // And a tree that still has one is no longer current.
        fs::write(out.join("main.rs"), "//! edited by hand\n").unwrap();
        assert_eq!(verify_tree(&out, &files()), ExitCode::FAILURE);
    }

    /// A plan the tree does not match yet is reported, and nothing is written.
    #[test]
    fn verifying_an_empty_tree_fails_without_writing() {
        let out = scratch("verify-empty");
        assert_eq!(verify_tree(&out, &files()), ExitCode::FAILURE);
        assert!(!out.join("main.rs").exists());
    }
}
