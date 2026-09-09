//! `plotplot receipt seal`, `verify` and `show`, driven as the binary a post-commit hook runs.
//!
//! The fixture is a real git repository with three commits, an origin remote and one
//! session's draft, because everything these three faces do is a statement about what git
//! holds. Three commits, so a range can hold a gap. The
//! assertions re-derive the commit and tree digests with git itself and the statement's
//! digest with sha2, so nothing here is checked against the same code that wrote it.

use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// The ref a receipt lives on. Spelled here rather than imported, so a change to the ref has
/// to be made in both places on purpose.
const RECEIPTS_REF: &str = "refs/notes/plotplot/receipts";

/// The session the fixture's draft belongs to.
const SESSION: &str = "6f3c1b2a-8d47-4f0e-9b31-2a5c7e91d044";

// ------------------------------------------------------------------------ the fixture

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the parent directory");
    }
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git runs on the build machine");
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// What git says a revision's commit and tree hashes are, asked of git and not of the stem.
fn digests(root: &Path, rev: &str) -> (String, String) {
    (
        git(root, &["rev-parse", rev]),
        git(root, &["rev-parse", &format!("{rev}^{{tree}}")]),
    )
}

/// One session's draft, in the shape `receipt draft` writes.
fn draft_json() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "harness": {"name": "claude-code", "version": "2.1.265"},
        "models": ["claude-opus-5"],
        "principal": "cli",
        "sessions": [SESSION],
        "cost": {
            "inputTokens": 41_000,
            "outputTokens": 2_100,
            "wallSeconds": null,
            "toolCalls": {"Bash": 41, "Edit": 12}
        },
        "friction": {"summary": {"sha256": "9c1d0f0e6a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f6071829304a5"}}
    }))
    .expect("a serializable draft")
        + "\n"
}

/// A repository with three commits, an origin on GitHub, and one draft waiting to be sealed.
fn fixture() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temp directory");
    let root = tmp.path().join("repo");
    std::fs::create_dir_all(&root).expect("the repository directory");

    git(tmp.path(), &["init", "-q", "repo"]);
    git(&root, &["config", "user.email", "fixture@plotplot.invalid"]);
    git(&root, &["config", "user.name", "plotplot fixture"]);
    git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/jahala/fixture.git",
        ],
    );

    write(&root.join(".gitignore"), ".plotplot/\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "the first commit"]);

    write(&root.join("README.md"), "# fixture\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "the second commit"]);

    write(&root.join("src/lib.rs"), "pub fn one() -> u8 {\n    1\n}\n");
    write(
        &root.join("docs/tend2/rules-block.tend2.html"),
        &loop_file(false),
    );
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "the third commit"]);

    write(
        &root
            .join(".plotplot/receipts/drafts")
            .join(format!("{SESSION}.json")),
        &draft_json(),
    );

    (tmp, root)
}

/// A loop file with three checks, the third of them stamped when `stamped` is true. The
/// stamp is what `seal` reads out of a diff.
fn loop_file(stamped: bool) -> String {
    let third = if stamped {
        "- [x] (code) the third check · scripts/fit/rules.sh three @9853abb · by cli\n"
    } else {
        "- [ ] (code) the third check · scripts/fit/rules.sh three\n"
    };
    format!(
        "<script type=\"text/markdown\" id=\"loop\">\n\
         # rules-block\n\
         \n\
         ## Tests\n\
         - [ ] (code) the first check · scripts/fit/rules.sh one\n\
         - [ ] (code) the second check · scripts/fit/rules.sh two\n\
         {third}</script>\n"
    )
}

// ------------------------------------------------------------------------ the faces

fn plotplot(root: &Path, args: &[&str]) -> Output {
    let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
    command.current_dir(root);
    command.args(args);
    command.output().expect("the binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn seal(root: &Path, args: &[&str]) -> Output {
    let mut all = vec!["receipt", "seal"];
    all.extend_from_slice(args);
    let output = plotplot(root, &all);
    assert_eq!(
        code(&output),
        0,
        "receipt seal {}: {}{}",
        args.join(" "),
        stdout(&output),
        stderr(&output)
    );
    output
}

/// The note git holds for a commit, read with git and not with the stem.
fn note(root: &Path, rev: &str) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["notes", "--ref", RECEIPTS_REF, "show", rev])
        .output()
        .expect("git runs on the build machine");
    assert!(
        output.status.success(),
        "no note on {rev}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// A note split the way its own last line says it should be: the statement's bytes, and the
/// digest those bytes are claimed to have.
fn split(note: &str) -> (String, String) {
    let (statement, last) = note
        .trim_end_matches('\n')
        .rsplit_once('\n')
        .expect("a note carries a statement and a digest line");
    let digest = last
        .strip_prefix("sha256: ")
        .expect("the last line is the statement's sha256");
    (format!("{statement}\n"), digest.to_owned())
}

fn statement(root: &Path, rev: &str) -> Value {
    let note = note(root, rev);
    let (bytes, _) = split(&note);
    serde_json::from_str(&bytes).expect("the note's statement is JSON")
}

/// The lines of one object inside a pretty-printed JSON document, without its braces.
///
/// serde_json's `Value` is a sorted map, so a document parsed and asked for its keys says
/// nothing about the order they were written in — and the order is what §3 fixes. These two
/// helpers read the bytes.
fn object(text: &str, key: &str) -> String {
    let opening = format!("\"{key}\": {{");
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.trim_start().starts_with(&opening))
        .unwrap_or_else(|| panic!("no object named {key} in\n{text}"));
    let indent = lines[at].len() - lines[at].trim_start().len();
    let closing = format!("{}}}", " ".repeat(indent));
    let end = lines[at + 1..]
        .iter()
        .position(|line| line.trim_end_matches(',') == closing)
        .expect("the object closes")
        + at
        + 1;
    lines[at + 1..end].join("\n")
}

/// The keys of an object's body, in the order they are written.
fn keys(body: &str) -> Vec<String> {
    let indent = body
        .lines()
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or_default();
    let prefix = " ".repeat(indent);
    body.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix(&prefix)?.strip_prefix('"')?;
            if rest.starts_with(' ') {
                return None;
            }
            Some(rest.split_once("\":")?.0.to_owned())
        })
        .collect()
}

/// The statement's own body: the whole document without its outer braces.
fn body(text: &str) -> String {
    let lines: Vec<&str> = text.trim_end().lines().collect();
    lines[1..lines.len() - 1].join("\n")
}

// ------------------------------------------------------------------------ seal

#[test]
fn the_statement_carries_the_plans_keys_in_the_plans_order() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let (text, _) = split(&note(&root, "HEAD"));
    assert_eq!(
        keys(&body(&text)),
        ["_type", "subject", "predicateType", "predicate"]
    );
    let statement = statement(&root, "HEAD");
    assert_eq!(statement["_type"], "https://in-toto.io/Statement/v1");
    assert_eq!(statement["predicateType"], "https://plotplot.ai/receipt/v1");

    let predicate = object(&text, "predicate");
    assert_eq!(
        keys(&predicate),
        [
            "producer",
            "createdAt",
            "harness",
            "models",
            "principal",
            "sessions",
            "conductor",
            "changed",
            "verification",
            "cost",
            "friction"
        ]
    );
    assert_eq!(
        keys(&object(&predicate, "verification")),
        ["weeder", "tend2", "pleach", "commands"]
    );
    assert_eq!(
        keys(&object(&predicate, "cost")),
        ["inputTokens", "outputTokens", "wallSeconds", "toolCalls"]
    );
    assert_eq!(keys(&object(&text, "digest")), ["gitCommit", "gitTree"]);
}

#[test]
fn the_subjects_digests_are_the_ones_git_recomputes() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let (commit, tree) = digests(&root, "HEAD");
    let statement = statement(&root, "HEAD");
    let subject = &statement["subject"][0];
    assert_eq!(subject["name"], "github.com/jahala/fixture");
    assert_eq!(subject["digest"]["gitCommit"], commit);
    assert_eq!(subject["digest"]["gitTree"], tree);
}

#[test]
fn the_notes_last_line_is_the_sha256_of_the_statement_above_it() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let note = note(&root, "HEAD");
    let (bytes, claimed) = split(&note);
    assert_eq!(hex::encode(Sha256::digest(bytes.as_bytes())), claimed);
}

#[test]
fn the_predicate_says_what_the_producer_is_and_when_it_sealed() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let predicate = statement(&root, "HEAD")["predicate"].clone();
    assert_eq!(predicate["producer"]["name"], "plotplot");
    assert_eq!(predicate["producer"]["version"], env!("CARGO_PKG_VERSION"));
    let created = predicate["createdAt"]
        .as_str()
        .expect("createdAt is a string");
    assert!(created.ends_with('Z'), "{created}");
    assert!(created.starts_with("20"), "{created}");
}

#[test]
fn the_draft_is_folded_in_and_then_put_away_so_it_is_never_folded_twice() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let predicate = statement(&root, "HEAD")["predicate"].clone();
    assert_eq!(predicate["harness"]["name"], "claude-code");
    assert_eq!(predicate["harness"]["version"], "2.1.265");
    assert_eq!(predicate["models"], serde_json::json!(["claude-opus-5"]));
    assert_eq!(predicate["principal"], "cli");
    assert_eq!(predicate["sessions"], serde_json::json!([SESSION]));
    assert_eq!(predicate["cost"]["inputTokens"], 41_000);
    assert_eq!(predicate["cost"]["toolCalls"]["Bash"], 41);
    assert!(predicate["friction"]["summary"]["sha256"].is_string());

    let commit = git(&root, &["rev-parse", "HEAD"]);
    let drafts = root.join(".plotplot/receipts/drafts");
    let sealed = root.join(".plotplot/receipts/sealed").join(&commit);
    assert!(
        !drafts.join(format!("{SESSION}.json")).exists(),
        "the folded draft is still under drafts/"
    );
    assert!(
        sealed.join(format!("{SESSION}.json")).exists(),
        "the folded draft was not put away under sealed/{commit}/"
    );
}

#[test]
fn a_commit_with_no_draft_says_so_with_nulls_and_no_sessions() {
    let (_tmp, root) = fixture();
    std::fs::remove_dir_all(root.join(".plotplot/receipts/drafts")).expect("the drafts go away");
    seal(&root, &[]);

    let predicate = statement(&root, "HEAD")["predicate"].clone();
    assert!(predicate["harness"].is_null());
    assert!(predicate["models"].is_null());
    assert!(predicate["principal"].is_null());
    assert_eq!(predicate["sessions"], serde_json::json!([]));
    assert!(predicate["friction"]["summary"].is_null());
    assert!(predicate["cost"]["inputTokens"].is_null());
    assert_eq!(predicate["cost"]["toolCalls"], serde_json::json!({}));
}

#[test]
fn what_this_slice_cannot_know_is_null_and_never_guessed() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let predicate = statement(&root, "HEAD")["predicate"].clone();
    assert!(predicate["conductor"].is_null());
    assert!(predicate["verification"]["pleach"].is_null());
    // No judge is placed in this fixture, so there is no SARIF to digest.
    assert!(predicate["verification"]["weeder"].is_null());
    assert!(predicate["cost"]["wallSeconds"].is_null());
}

#[test]
fn changed_is_what_the_commit_touched_and_nothing_else() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    assert_eq!(
        statement(&root, "HEAD")["predicate"]["changed"],
        serde_json::json!(["docs/tend2/rules-block.tend2.html", "src/lib.rs"])
    );
}

#[test]
fn a_stamp_the_diff_adds_to_a_loop_is_carried_with_its_check_and_its_sha() {
    let (_tmp, root) = fixture();
    write(
        &root.join("docs/tend2/rules-block.tend2.html"),
        &loop_file(true),
    );
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "the verifier stamps c3"]);
    seal(&root, &[]);

    assert_eq!(
        statement(&root, "HEAD")["predicate"]["verification"]["tend2"],
        serde_json::json!([{"check": "rules-block:c3", "sha": "9853abb"}])
    );
}

#[test]
fn a_commit_that_stamps_nothing_carries_no_stamps() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);
    assert_eq!(
        statement(&root, "HEAD")["predicate"]["verification"]["tend2"],
        serde_json::json!([])
    );
}

#[test]
fn seal_takes_the_commit_it_is_given() {
    let (_tmp, root) = fixture();
    seal(&root, &["--commit", "HEAD~1"]);

    let (commit, tree) = digests(&root, "HEAD~1");
    let statement = statement(&root, "HEAD~1");
    assert_eq!(statement["subject"][0]["digest"]["gitCommit"], commit);
    assert_eq!(statement["subject"][0]["digest"]["gitTree"], tree);
}

#[test]
fn a_second_seal_of_the_same_commit_changes_nothing_and_says_so() {
    let (_tmp, root) = fixture();
    let first = seal(&root, &[]);
    assert!(stdout(&first).contains("sealed"), "{}", stdout(&first));
    let before = note(&root, "HEAD");

    let second = seal(&root, &[]);
    assert!(stdout(&second).contains("unchanged"), "{}", stdout(&second));
    assert_eq!(note(&root, "HEAD"), before);
}

#[test]
fn a_seal_that_would_differ_replaces_the_note_and_says_so() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);
    let before = note(&root, "HEAD");

    // A second session's draft is new information about the same commit.
    write(
        &root.join(".plotplot/receipts/drafts/2b7d4e10.json"),
        &draft_json().replace(SESSION, "2b7d4e10"),
    );
    let again = seal(&root, &[]);
    assert!(stdout(&again).contains("replaced"), "{}", stdout(&again));
    assert_ne!(note(&root, "HEAD"), before);
    assert_eq!(
        statement(&root, "HEAD")["predicate"]["sessions"],
        serde_json::json!(["2b7d4e10", SESSION])
    );
}

// ------------------------------------------------------------------------ verify

#[test]
fn verify_passes_a_commit_that_carries_a_valid_receipt() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let output = plotplot(&root, &["receipt", "verify", "HEAD"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let short = git(&root, &["rev-parse", "--short", "HEAD"]);
    assert_eq!(stdout(&output), format!("{short} receipt ok\n"));
}

#[test]
fn verify_refuses_a_commit_with_no_receipt() {
    let (_tmp, root) = fixture();

    let output = plotplot(&root, &["receipt", "verify", "HEAD"]);
    assert_eq!(code(&output), 3, "{}", stdout(&output));
    assert!(
        stdout(&output).contains("no receipt"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn verify_refuses_a_note_whose_predicate_was_edited() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let tampered = note(&root, "HEAD").replace("\"cli\"", "\"CLI\"");
    let path = root.join("tampered.txt");
    write(&path, &tampered);
    git(
        &root,
        &[
            "notes",
            "--ref",
            RECEIPTS_REF,
            "add",
            "-f",
            "-F",
            &path.to_string_lossy(),
            "HEAD",
        ],
    );

    let output = plotplot(&root, &["receipt", "verify", "HEAD"]);
    assert_eq!(code(&output), 3, "{}", stdout(&output));
    assert!(
        stdout(&output).contains("digest mismatch"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn verify_refuses_a_note_whose_subject_is_not_the_commit_it_hangs_on() {
    let (_tmp, root) = fixture();
    seal(&root, &["--commit", "HEAD~1"]);

    // The receipt of the first commit, re-attached to the second: its digest line is still
    // right, and it is still not this commit's receipt.
    let stolen = note(&root, "HEAD~1");
    let path = root.join("stolen.txt");
    write(&path, &stolen);
    git(
        &root,
        &[
            "notes",
            "--ref",
            RECEIPTS_REF,
            "add",
            "-f",
            "-F",
            &path.to_string_lossy(),
            "HEAD",
        ],
    );

    let output = plotplot(&root, &["receipt", "verify", "HEAD"]);
    assert_eq!(code(&output), 3, "{}", stdout(&output));
    assert!(stdout(&output).contains("gitCommit"), "{}", stdout(&output));
}

#[test]
fn a_range_with_a_gap_is_refused_and_names_the_commit_that_has_none() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let first = git(&root, &["rev-parse", "--short", "HEAD~1"]);
    let output = plotplot(&root, &["receipt", "verify", "--range", "HEAD~2..HEAD"]);
    assert_eq!(code(&output), 3, "{}", stdout(&output));
    assert!(
        stdout(&output).contains(&format!("{first} no receipt")),
        "{}",
        stdout(&output)
    );
}

#[test]
fn a_range_every_commit_of_which_carries_a_receipt_passes() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);
    seal(&root, &["--commit", "HEAD~1"]);

    let output = plotplot(&root, &["receipt", "verify", "--range", "HEAD~2..HEAD"]);
    assert_eq!(code(&output), 0, "{}{}", stdout(&output), stderr(&output));
    assert_eq!(stdout(&output).lines().count(), 2, "{}", stdout(&output));
    for line in stdout(&output).lines() {
        assert!(line.ends_with(" receipt ok"), "{line}");
    }
}

#[test]
fn verify_refuses_every_v0_note_when_it_is_asked_for_a_signature() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let output = plotplot(&root, &["receipt", "verify", "HEAD", "--require-signed"]);
    assert_eq!(code(&output), 3, "{}", stdout(&output));
    assert!(
        stdout(&output).contains("unsigned (v0)"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn verify_answers_one_when_git_cannot_say_what_the_revision_is() {
    let (_tmp, root) = fixture();
    let output = plotplot(&root, &["receipt", "verify", "no-such-revision"]);
    assert_eq!(code(&output), 1, "{}{}", stdout(&output), stderr(&output));
    assert!(!stderr(&output).is_empty());
}

// ------------------------------------------------------------------------ show

#[test]
fn show_prints_the_predicate_and_nothing_around_it() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let output = plotplot(&root, &["receipt", "show", "HEAD"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let printed = stdout(&output);
    let predicate: Value =
        serde_json::from_str(&printed).expect("show prints the predicate as JSON");
    assert!(predicate["producer"]["name"] == "plotplot", "{printed}");
    assert_eq!(
        keys(&body(&printed)),
        [
            "producer",
            "createdAt",
            "harness",
            "models",
            "principal",
            "sessions",
            "conductor",
            "changed",
            "verification",
            "cost",
            "friction"
        ]
    );
    assert!(
        printed.contains("\n  \"producer\""),
        "the predicate is printed for a person to read"
    );
}

#[test]
fn show_refuses_a_commit_that_carries_no_receipt() {
    let (_tmp, root) = fixture();
    let output = plotplot(&root, &["receipt", "show", "HEAD"]);
    assert_eq!(code(&output), 3, "{}", stdout(&output));
    assert!(
        stderr(&output).contains("no receipt"),
        "{}",
        stderr(&output)
    );
}

// ------------------------------------------------------------------------ weeder

/// The log the fixture judge prints: one block and two warnings, the smallest thing the
/// SARIF 2.1.0 schema accepts that carries counts worth reading.
const WEEDER_LOG: &str = concat!(
    "{\"$schema\":\"https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/",
    "sarif-schema-2.1.0.json\",\"version\":\"2.1.0\",\"runs\":[{\"tool\":{\"driver\":",
    "{\"name\":\"weeder\",\"version\":\"0.1.0\"}},\"results\":[",
    "{\"level\":\"error\",\"message\":{\"text\":\"a stub reached production code\"}},",
    "{\"level\":\"warning\",\"message\":{\"text\":\"an error was swallowed\"}},",
    "{\"level\":\"warning\",\"message\":{\"text\":\"a debug leftover reached production code\"}}",
    "]}]}\n"
);

/// A judge under `.plotplot/bin/`, which is the only place the stem calls one from. It
/// records the arguments it was called with, so the check command itself is asserted.
fn place_weeder(root: &Path, prints: &str) {
    let judge = root.join(".plotplot/bin/weeder");
    write(
        &judge,
        &format!(
            "#!/bin/sh\nprintf '%s' \"$*\" > .plotplot/weeder-args\ncat <<'LOG'\n{prints}LOG\nexit 2\n"
        ),
    );
    let mut mode = std::fs::metadata(&judge)
        .expect("the judge was written")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
    std::fs::set_permissions(&judge, mode).expect("the judge can be made executable");
}

#[test]
fn a_pinned_weeder_is_asked_about_the_commit_and_its_log_is_digested_and_counted() {
    let (_tmp, root) = fixture();
    place_weeder(&root, WEEDER_LOG);
    seal(&root, &[]);

    let commit = git(&root, &["rev-parse", "HEAD"]);
    assert_eq!(
        std::fs::read_to_string(root.join(".plotplot/weeder-args")).expect("the judge ran"),
        format!("check --format sarif --base {commit}^")
    );

    let weeder = statement(&root, "HEAD")["predicate"]["verification"]["weeder"].clone();
    assert_eq!(weeder["block"], 1);
    assert_eq!(weeder["warn"], 2);
    assert_eq!(
        weeder["sarif"]["sha256"],
        hex::encode(Sha256::digest(WEEDER_LOG.as_bytes()))
    );
}

#[test]
fn a_judge_that_answers_with_something_other_than_sarif_leaves_null_and_says_why() {
    let (_tmp, root) = fixture();
    place_weeder(&root, "not sarif at all\n");
    let output = seal(&root, &[]);

    assert!(
        statement(&root, "HEAD")["predicate"]["verification"]["weeder"].is_null(),
        "a judge that did not answer is not a count"
    );
    assert!(
        stderr(&output).contains("weeder"),
        "the seal said nothing about the judge it could not read: {}",
        stderr(&output)
    );
}

#[test]
fn verify_refuses_a_note_that_is_not_text_rather_than_failing_to_judge() {
    let (_tmp, root) = fixture();
    seal(&root, &[]);

    let path = root.join("bytes.bin");
    std::fs::write(&path, [0x7b, 0xff, 0xfe, 0x0a]).expect("bytes that are not UTF-8");
    git(
        &root,
        &[
            "notes",
            "--ref",
            RECEIPTS_REF,
            "add",
            "-f",
            "-F",
            &path.to_string_lossy(),
            "HEAD",
        ],
    );

    let output = plotplot(&root, &["receipt", "verify", "HEAD"]);
    assert_eq!(code(&output), 3, "{}{}", stdout(&output), stderr(&output));
    assert!(stdout(&output).contains("UTF-8"), "{}", stdout(&output));
}
