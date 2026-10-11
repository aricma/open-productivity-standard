//! Rendering the generated Rust source.
//!
//! A case becomes one `#[test]`, a kind becomes one module, and a format
//! becomes a directory of modules. Rendering is pure: given a case and the
//! source paths it needs, it returns text.

use crate::case::{Case, Group, test_name};
use crate::constants;
use crate::formats::{self, Unsupported};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

/// The `#[test]` blocks for one module's cases, ending with a trailing newline.
///
/// A case whose format this implementation does not claim becomes an
/// `#[ignore]`d test, so a skip stays visible and counted instead of silently
/// absent.
pub(crate) fn render_tests(root: &Path, out: &Path, file: &str, cases: &[&Case]) -> String {
    let reason = constants::UNSUPPORTED_REASON;
    let mut names: HashMap<String, usize> = HashMap::new();
    let mut tests = String::new();
    let mut ignored = Vec::new();

    for case in cases {
        let base = test_name(&case.name);
        let seen = names.entry(base.clone()).or_insert(0);
        *seen += 1;
        let name = if *seen == 1 {
            base
        } else {
            format!("{base}_{seen}")
        };
        match case_body(root, out, file, case) {
            Ok(body) => {
                writeln!(
                    tests,
                    "#[rustfmt::skip]\n#[test]\nfn {name}() {{\n{body}}}\n"
                )
                .unwrap();
            }
            Err(Unsupported(format)) => ignored.push((name, format)),
        }
    }
    for (name, format) in ignored {
        writeln!(
            tests,
            "#[rustfmt::skip]\n#[test]\n#[ignore = \"{reason} {format}\"]\nfn {name}() {{}}\n"
        )
        .unwrap();
    }
    if !tests.is_empty() {
        tests.pop();
    }
    tests
}

/// One case's test body, or the format this implementation does not claim.
///
/// The support check lives here, and the rendering below it, so that every
/// shape is exercised by a test — including the writer-reject one that no
/// claimed format reaches today.
pub(crate) fn case_body(
    root: &Path,
    out: &Path,
    file: &str,
    case: &Case,
) -> Result<String, Unsupported> {
    if let Some(missing) = case
        .format_names()
        .into_iter()
        .find(|name| formats::variant_name(&case.version, name).is_none())
    {
        return Err(Unsupported(missing.to_string()));
    }
    let (reads, writes) = case.format_paths();
    let reads = reads.expect("checked above");
    let api = format!("ops_lib::{}", case.version);
    Ok(render_body(
        case,
        &prefix_to_root(root, out, file),
        &api,
        &reads,
        writes.as_deref(),
    ))
}

/// One test body, from the parts it needs. Pure, so it can be rendered for any
/// shape without a corpus on disk.
fn render_body(case: &Case, prefix: &str, api: &str, reads: &str, writes: Option<&str>) -> String {
    let given = include_str_expr(prefix, &case.given);
    let where_it_lives = case.corpus_path();
    let claim = &case.claim;
    match case.group {
        Group::Valid => format!(
            r#"    let given = {given};
    {api}::read({reads}, given)
        .unwrap_or_else(|e| panic!("{where_it_lives}: must be accepted — {claim}: {{e}}"));
"#
        ),
        Group::Invalid => format!(
            r#"    let given = {given};
    assert!(
        {api}::read({reads}, given).is_err(),
        "{where_it_lives}: must be rejected — {claim}"
    );
"#
        ),
        // A conversion that cannot be done: the model is valid, so the read
        // has to succeed, and only the write is expected to fail.
        Group::Transform if case.must_fail() => {
            let writes = writes.expect("a conversion names the format it converts to");
            format!(
                r#"    let given = {given};
    let tasks = {api}::read({reads}, given)
        .unwrap_or_else(|e| panic!("{where_it_lives}: given rejected: {{e}}"));
    assert!(
        {api}::write({writes}, &tasks).is_err(),
        "{where_it_lives}: must be rejected — {claim}"
    );
"#
            )
        }
        Group::Roundtrip | Group::Transform => {
            let writes = writes.expect("a round-trip or conversion writes a format");
            let (_, expectation) = case
                .expectation
                .as_ref()
                .expect("checked when the case was read");
            let expected = include_str_expr(prefix, expectation);
            format!(
                r#"    let given = {given};
    let expected = {expected};
    let tasks = {api}::read({reads}, given)
        .unwrap_or_else(|e| panic!("{where_it_lives}: given rejected: {{e}}"));
    let out = {api}::write({writes}, &tasks)
        .unwrap_or_else(|e| panic!("{where_it_lives}: write failed: {{e}}"));
    let produced = {api}::read({writes}, &out)
        .unwrap_or_else(|e| panic!("{where_it_lives}: output rejected: {{e}}"));
    let wanted = {api}::read({writes}, expected)
        .unwrap_or_else(|e| panic!("{where_it_lives}: expectation rejected: {{e}}"));
    assert_eq!(
        produced, wanted,
        "{where_it_lives}: {claim} (the exported model differs from the expectation)"
    );
"#
            )
        }
    }
}

/// The `../..` that gets from a generated file back to the repository root,
/// where the corpus paths are relative to.
fn prefix_to_root(root: &Path, out: &Path, file: &str) -> String {
    let path = out.join(file);
    let directory = path.parent().expect("a generated file has a parent");
    assert!(
        directory.starts_with(root),
        "{} is not under {}",
        directory.display(),
        root.display()
    );
    let depth = directory
        .strip_prefix(root)
        .expect("checked above")
        .components()
        .count();
    "../".repeat(depth)
}

/// A corpus file, as an `include_str!` expression.
fn include_str_expr(prefix: &str, path: &str) -> String {
    format!("include_str!(\"{prefix}{path}\")")
}

/// The doc comment at the top of a generated file.
pub(crate) fn file_header(summary: &str) -> String {
    format!("//! {summary} Generated by `gen-corpus`; do not edit by hand.\n\n")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn case(
        group: Group,
        format: Option<&str>,
        given: &str,
        target: Option<&str>,
        expectation: Option<&str>,
    ) -> Case {
        Case {
            version: "v0".to_string(),
            format: format.map(str::to_string),
            group,
            name: "case".to_string(),
            claim: "the claim".to_string(),
            given_ext: given.to_string(),
            given: format!("specs/v0/tests/place/given.{given}"),
            target_ext: target.map(str::to_string),
            expectation: expectation.map(|ext| {
                (
                    ext.to_string(),
                    format!("specs/v0/tests/place/expectation.{ext}"),
                )
            }),
        }
    }

    /// Rendered from the paths directly, so every shape is covered — including
    /// the conversion that cannot be done, which no corpus case reaches while
    /// the format it needs is unclaimed.
    fn body(case: &Case, writes: Option<&str>) -> String {
        let reads = formats::variant_path(&case.version, &case.given_ext)
            .expect("these test cases are in a claimed format");
        render_body(case, "../", "ops_lib::v0", &reads, writes)
    }

    #[test]
    fn each_shape_generates_its_assertion() {
        let valid = body(&case(Group::Valid, Some("json"), "json", None, None), None);
        assert!(
            valid.contains("::read(ops_lib::v0::Format::Json"),
            "{valid}"
        );
        assert!(valid.contains("must be accepted"), "{valid}");

        // An `invalid` case is a document the enclosing format's reader rejects.
        let invalid = body(
            &case(Group::Invalid, Some("json"), "json", None, None),
            None,
        );
        assert!(invalid.contains(".is_err()"), "{invalid}");
        assert!(
            invalid.contains("::read(ops_lib::v0::Format::Json"),
            "{invalid}"
        );

        // A conversion that cannot be done reads the model and refuses to write.
        let impossible = body(
            &case(Group::Transform, None, "json", Some("jsonl"), None),
            Some("ops_lib::v0::Format::Jsonl"),
        );
        assert!(
            impossible.contains("::read(ops_lib::v0::Format::Json"),
            "{impossible}"
        );
        assert!(
            impossible.contains("::write(ops_lib::v0::Format::Jsonl"),
            "{impossible}"
        );
        assert!(impossible.contains(".is_err()"), "{impossible}");

        let roundtrip = body(
            &case(
                Group::Roundtrip,
                Some("json"),
                "json",
                Some("json"),
                Some("json"),
            ),
            Some("ops_lib::v0::Format::Json"),
        );
        assert!(
            roundtrip.contains("::write(ops_lib::v0::Format::Json"),
            "{roundtrip}"
        );

        let transform = body(
            &case(Group::Transform, None, "json", Some("yaml"), Some("yaml")),
            Some("ops_lib::v0::Format::Yaml"),
        );
        assert!(
            transform.contains("::write(ops_lib::v0::Format::Yaml"),
            "{transform}"
        );
    }

    /// Both documents are read back, and the comparison is the model.
    #[test]
    fn a_round_trip_compares_the_model() {
        let roundtrip = body(
            &case(
                Group::Roundtrip,
                Some("json"),
                "json",
                Some("json"),
                Some("json"),
            ),
            Some("ops_lib::v0::Format::Json"),
        );
        assert!(roundtrip.contains("expectation.json"), "{roundtrip}");
        assert!(roundtrip.contains("assert_eq!("), "{roundtrip}");
    }

    /// A format the implementation does not claim makes a case ignored instead
    /// of generating a body. The name in the reason is whichever the case
    /// mentions first.
    #[test]
    fn an_unsupported_format_is_ignored() {
        let root = Path::new("/repo");
        let out = root.join(constants::ACCEPTANCE_DIR);
        let markdown = case(Group::Valid, Some("markdown"), "md", None, None);
        assert_eq!(
            case_body(root, &out, "v0/markdown/valid.rs", &markdown)
                .unwrap_err()
                .0,
            "md"
        );
        let impossible = case(Group::Transform, None, "json", Some("markdown"), None);
        assert_eq!(
            case_body(root, &out, "v0/transform.rs", &impossible)
                .unwrap_err()
                .0,
            "markdown"
        );
    }

    /// Two cases with the same directory name get distinct test names.
    #[test]
    fn duplicate_case_names_are_numbered() {
        let one = case(Group::Valid, Some("json"), "json", None, None);
        let two = case(Group::Valid, Some("json"), "json", None, None);
        let tests = render_tests(
            Path::new("/repo"),
            Path::new("/repo"),
            "f.rs",
            &[&one, &two],
        );
        assert_eq!(tests.matches("fn case()").count(), 1, "{tests}");
        assert_eq!(tests.matches("fn case_2()").count(), 1, "{tests}");
    }
}
