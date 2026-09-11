//! What `init` will write: the garden block, the git hooks and the git configuration.
//!
//! The pure shapes are unit-tested beside their code. This file checks them against ground
//! truth a unit test cannot reach: real git repositories, this repository's own `AGENTS.md`,
//! and the system's `sh` reading each rendered hook.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use plotplot::bed::{Bed, GitHook};
use plotplot::error::Error;
use plotplot::plant::{garden_block, gitconfig, githooks};

/// A bed carrying only the fields these renderers read.
fn bed(name: &str, version: &str, binary: Option<&str>, git_hooks: &[GitHook]) -> Bed {
    Bed {
        name: name.to_owned(),
        version: version.to_owned(),
        binary: binary.map(str::to_owned),
        skill: None,
        hooks: std::collections::BTreeMap::new(),
        git_hooks: git_hooks.to_vec(),
        mcp: None,
        check: None,
    }
}

fn planted_beds() -> Vec<Bed> {
    vec![
        bed(
            "tend2",
            "1.0.0",
            Some("tend2"),
            &[GitHook::PreCommit, GitHook::PostCommit],
        ),
        bed("tilth", "0.10.1", Some("tilth"), &[]),
        bed(
            "weeder",
            "0.1.0",
            Some("weeder"),
            &[GitHook::PreCommit, GitHook::PrePush, GitHook::PreRebase],
        ),
    ]
}

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git runs on the build machine");
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Every value git holds for `key` in the repository's own configuration.
fn values(repo: &Path, key: &str) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "--local", "--get-all", key])
        .output()
        .expect("git runs on the build machine");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// A repository with a remote named origin and no refspecs of its own, which is what the
/// three keys look like before `init` has run.
fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    git(dir.path(), &["init", "--quiet"]);
    git(
        dir.path(),
        &[
            "config",
            "remote.origin.url",
            "https://example.invalid/garden.git",
        ],
    );
    dir
}

/// tend2's block, copied byte for byte from this repository's AGENTS.md, which is also that
/// file as it was before the stem planted it. The stem never renders it: it is what tend2's
/// init writes, and the input these tests plant beside.
fn tend2_block_fixture() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/AGENTS.tend2.md");
    std::fs::read_to_string(&path).expect("the tend2 block fixture")
}

/// What `sed -n '/tend2:begin/,/tend2:end/p'` prints: the whole lines from the first that
/// carries tend2's begin marker through the next that carries its end marker.
fn tend2_region(agents_md: &str) -> String {
    agents_md
        .split_inclusive('\n')
        .skip_while(|line| !line.contains("<!-- tend2:begin -->"))
        .scan(false, |ended, line| {
            if *ended {
                return None;
            }
            *ended = line.contains("<!-- tend2:end -->");
            Some(line)
        })
        .collect()
}

/// The bytes before the garden block's begin marker, and the bytes after its end marker.
fn outside_the_garden_block(agents_md: &str) -> (&[u8], &[u8]) {
    let begin = agents_md
        .find(garden_block::BEGIN)
        .expect("the begin marker");
    let end = agents_md.find(garden_block::END).expect("the end marker") + garden_block::END.len();
    let bytes = agents_md.as_bytes();
    (&bytes[..begin], &bytes[end..])
}

// ---------------------------------------------------------------- garden_block

#[test]
fn the_block_is_marked_short_and_says_what_a_planted_repository_needs() {
    let block = garden_block::render("2026.09", &planted_beds());

    let (before, rest) = block
        .split_once(garden_block::BEGIN)
        .expect("the begin marker");
    let (body, after) = rest.split_once(garden_block::END).expect("the end marker");
    assert_eq!(before, "");
    assert_eq!(after, "");
    let body = body.trim_matches('\n');
    assert!(
        body.lines().count() <= 10,
        "more than ten lines between the markers:\n{body}"
    );

    for bed in planted_beds() {
        assert!(
            block.contains(&format!("{} {}", bed.name, bed.version)),
            "{} is not named with its version:\n{block}",
            bed.name
        );
    }
    assert!(block.contains("2026.09"), "{block}");
    assert!(block.contains("tend2 next docs/tend2"), "{block}");
    assert!(block.contains("plotplot check"), "{block}");
    assert!(block.contains("plotplot doctor"), "{block}");
    assert!(block.contains("--no-verify"), "{block}");
    assert!(block.contains("garden.lock"), "{block}");
    assert!(block.contains(".githooks"), "{block}");
    assert!(!body.contains('!'), "{block}");
    assert!(!body.contains('\u{2014}'), "{block}");
}

#[test]
fn the_block_lands_after_an_unplanted_agents_md_and_leaves_it_alone() {
    let agents = tend2_block_fixture();
    let block = garden_block::render("2026.09", &planted_beds());

    let planted = garden_block::replace_in(&agents, &block).expect("a file with no markers");
    assert!(planted.starts_with(&agents), "the tend2 block moved");
    assert!(planted.contains(&block));
    assert!(
        planted[agents.len()..].starts_with('\n'),
        "the block did not follow one blank line"
    );
}

#[test]
fn planting_the_same_block_twice_changes_nothing_the_second_time() {
    let agents = tend2_block_fixture();
    let block = garden_block::render("2026.09", &planted_beds());

    let once = garden_block::replace_in(&agents, &block).expect("the first planting");
    let twice = garden_block::replace_in(&once, &block).expect("the second planting");
    assert_eq!(once, twice);
}

#[test]
fn a_new_block_replaces_only_the_marked_region() {
    let agents = tend2_block_fixture();
    let old = garden_block::render("2026.09", &planted_beds());
    let new = garden_block::render("2026.10", &planted_beds()[..1]);

    let planted = garden_block::replace_in(&agents, &old).expect("the first planting");
    let replanted = garden_block::replace_in(&planted, &new).expect("the second planting");

    assert!(replanted.starts_with(&agents), "the tend2 block moved");
    assert!(replanted.contains(&new));
    assert!(!replanted.contains("2026.09"), "{replanted}");
    assert_eq!(
        planted.len() - old.len(),
        replanted.len() - new.len(),
        "bytes outside the markers changed"
    );
}

#[test]
fn a_begin_marker_without_an_end_is_refused() {
    let agents = format!(
        "{}\n{}\nplanted\n",
        tend2_block_fixture(),
        garden_block::BEGIN
    );
    let block = garden_block::render("2026.09", &planted_beds());

    match garden_block::replace_in(&agents, &block) {
        Err(Error::Agents { problem }) => {
            assert!(problem.contains(garden_block::END), "{problem}");
        }
        other => panic!("expected Error::Agents, got {other:?}"),
    }
}

// ------------------------------------------- garden_block beside tend2's block

// tend2's init writes its own block between `<!-- tend2:begin -->` and `<!-- tend2:end -->`,
// beside the garden block, and each init leaves the other's bytes alone (jahala/plotplot
// issue 28, ruled 2026-09-10 on the contracts loop). Every comparison here is of bytes.

const PROSE: &str = "This repository keeps notes of its own here.\n";
const MORE_PROSE: &str = "## Written by hand\n\nNobody else's to move.\n";

#[test]
fn planting_beside_tend2s_block_alone_keeps_it_first_and_the_garden_block_one_blank_line_below() {
    let tend2 = tend2_block_fixture();
    let block = garden_block::render("2026.09", &planted_beds());

    let planted =
        garden_block::replace_in(&tend2, &block).expect("a file with no plotplot markers");

    let (before, after) = outside_the_garden_block(&planted);
    assert_eq!(
        before,
        format!("{tend2}\n").as_bytes(),
        "the bytes before the garden block are not tend2's block and one blank line:\n{planted}"
    );
    assert_eq!(after, b"\n", "{planted}");
    assert_eq!(tend2_region(&planted), tend2);
}

#[test]
fn replanting_between_tend2s_block_and_prose_moves_no_byte_outside_the_markers() {
    let tend2 = tend2_block_fixture();
    let older = garden_block::render("2026.09", &[bed("weeder", "0.1.0", Some("weeder"), &[])]);
    let newer = garden_block::render("2026.09", &[bed("weeder", "0.2.2", Some("weeder"), &[])]);
    let agents = format!("{tend2}\n{PROSE}\n{older}\n\n{MORE_PROSE}");

    let planted = garden_block::replace_in(&agents, &newer).expect("one marked region");

    let (was_before, was_after) = outside_the_garden_block(&agents);
    let (before, after) = outside_the_garden_block(&planted);
    assert_eq!(before, was_before, "bytes before the garden block changed");
    assert_eq!(after, was_after, "bytes after the garden block changed");
    assert_eq!(
        &planted.as_bytes()[before.len()..planted.len() - after.len()],
        newer.as_bytes()
    );
    assert!(planted.as_bytes().starts_with(tend2.as_bytes()));
    assert_eq!(tend2_region(&planted), tend2);
}

#[test]
fn planting_above_tend2s_block_leaves_it_byte_for_byte_below_the_garden_block() {
    let tend2 = tend2_block_fixture();
    let older = garden_block::render("2026.09", &[bed("weeder", "0.1.0", Some("weeder"), &[])]);
    let newer = garden_block::render("2026.09", &planted_beds());
    let agents = format!("{older}\n\n{tend2}");

    let planted = garden_block::replace_in(&agents, &newer).expect("one marked region");

    let (before, after) = outside_the_garden_block(&planted);
    assert_eq!(before, b"", "{planted}");
    assert_eq!(
        after,
        format!("\n\n{tend2}").as_bytes(),
        "tend2's block is not where it was below the garden block:\n{planted}"
    );
    assert_eq!(tend2_region(&planted), tend2);
}

#[test]
fn another_seasons_block_changes_only_the_marked_region_in_every_layout() {
    let tend2 = tend2_block_fixture();
    let this_season = garden_block::render("2026.09", &planted_beds());
    let next_season = garden_block::render("2026.10", &planted_beds());
    let layouts = [
        ("tend2's block alone", tend2.clone()),
        (
            "tend2's block, prose, the garden block, prose",
            format!("{tend2}\n{PROSE}\n{this_season}\n\n{MORE_PROSE}"),
        ),
        (
            "the garden block above tend2's block",
            format!("{this_season}\n\n{tend2}"),
        ),
    ];

    for (layout, agents) in layouts {
        let planted = garden_block::replace_in(&agents, &this_season).expect("this season");
        let replanted = garden_block::replace_in(&planted, &next_season).expect("the next season");

        let (was_before, was_after) = outside_the_garden_block(&planted);
        let (before, after) = outside_the_garden_block(&replanted);
        assert_eq!(
            before, was_before,
            "{layout}: bytes before the block changed"
        );
        assert_eq!(after, was_after, "{layout}: bytes after the block changed");
        assert_eq!(
            &replanted.as_bytes()[before.len()..replanted.len() - after.len()],
            next_season.as_bytes(),
            "{layout}: the marked region is not the next season's block"
        );
        assert_eq!(tend2_region(&replanted), tend2, "{layout}");
    }
}

#[test]
fn the_tend2_fixture_is_this_repositorys_tend2_block_byte_for_byte() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("AGENTS.md");
    let live = std::fs::read_to_string(&path).expect("this repository's AGENTS.md");

    let region = tend2_region(&live);
    assert!(
        region.starts_with("<!-- tend2:begin -->\n") && region.ends_with("<!-- tend2:end -->\n"),
        "AGENTS.md carries no whole tend2 block:\n{region}"
    );
    assert_eq!(tend2_block_fixture(), region);
}

// ------------------------------------------------------------------- githooks

/// `sh -n` parses the script without running it.
fn sh_accepts(script: &str) -> Result<(), String> {
    use std::io::Write;

    let mut child = Command::new("sh")
        .arg("-n")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sh runs on the build machine");
    child
        .stdin
        .as_mut()
        .expect("a pipe to sh")
        .write_all(script.as_bytes())
        .expect("sh reads the script");
    let output = child.wait_with_output().expect("sh finishes");
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[test]
fn every_hook_is_a_shell_script_sh_accepts_and_no_two_are_alike() {
    let beds = planted_beds();
    let mut rendered = Vec::new();
    for hook in GitHook::ALL {
        let script = githooks::render(hook, &beds);
        assert!(
            script.starts_with("#!/bin/sh\n"),
            "{}: {script}",
            hook.file_name()
        );
        assert!(
            script
                .lines()
                .nth(1)
                .is_some_and(|line| line.starts_with('#')
                    && line.contains("plotplot")
                    && line.contains("Do not edit")),
            "{}: the first comment does not claim the file:\n{script}",
            hook.file_name()
        );
        assert!(script.contains("\nset -eu\n"), "{}", hook.file_name());
        assert!(
            script.contains("git rev-parse --show-toplevel"),
            "{}",
            hook.file_name()
        );
        if let Err(problem) = sh_accepts(&script) {
            panic!("sh refused {}: {problem}\n{script}", hook.file_name());
        }
        rendered.push(script);
    }
    for (index, first) in rendered.iter().enumerate() {
        for second in &rendered[index + 1..] {
            assert_ne!(first, second, "two hooks rendered the same script");
        }
    }
}

#[test]
fn each_hook_calls_exactly_the_beds_that_declared_it() {
    let beds = planted_beds();

    let pre_commit = githooks::render(GitHook::PreCommit, &beds);
    assert!(
        pre_commit.contains(".plotplot/bin/tend2\" guard pre-commit"),
        "{pre_commit}"
    );
    assert!(
        pre_commit.contains(".plotplot/bin/weeder\" guard pre-commit"),
        "{pre_commit}"
    );
    assert!(!pre_commit.contains("tilth"), "{pre_commit}");

    let pre_push = githooks::render(GitHook::PrePush, &beds);
    assert!(
        pre_push.contains(".plotplot/bin/weeder\" guard pre-push"),
        "{pre_push}"
    );
    assert!(!pre_push.contains("tend2"), "{pre_push}");

    let pre_rebase = githooks::render(GitHook::PreRebase, &beds);
    assert!(
        pre_rebase.contains(".plotplot/bin/weeder\" guard pre-rebase"),
        "{pre_rebase}"
    );

    let post_commit = githooks::render(GitHook::PostCommit, &beds);
    assert!(
        post_commit.contains(".plotplot/bin/plotplot\" receipt seal"),
        "{post_commit}"
    );
    assert!(
        post_commit.contains(".plotplot/bin/tend2\" guard post-commit"),
        "{post_commit}"
    );
    let seal = post_commit
        .find("receipt seal")
        .expect("the receipt seal call");
    let tend2 = post_commit
        .find("guard post-commit")
        .expect("the tend2 guard");
    assert!(seal < tend2, "the beds run before the seal:\n{post_commit}");
}

#[test]
fn a_hook_no_bed_declared_is_still_a_script_that_exits_zero() {
    let beds = vec![bed("tilth", "0.10.1", Some("tilth"), &[])];
    for hook in [GitHook::PreCommit, GitHook::PrePush, GitHook::PreRebase] {
        let script = githooks::render(hook, &beds);
        assert!(sh_accepts(&script).is_ok(), "{script}");
        assert!(!script.contains(" guard "), "{script}");
        assert!(script.trim_end().ends_with("exit 0"), "{script}");
    }
}

/// A repository holding executable stand-ins for `beds`, each recording the arguments and
/// the stdin it was called with and exiting with `code`.
fn repository_with_judges(binaries: &[(&str, i32)]) -> tempfile::TempDir {
    let repo = repository();
    let bin = repo.path().join(".plotplot/bin");
    std::fs::create_dir_all(&bin).expect("the judges' directory");
    for (binary, code) in binaries {
        let path = bin.join(binary);
        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf '%s' \"$*\" > \"$(git rev-parse --show-toplevel)/{binary}.args\"\ncat > \"$(git rev-parse --show-toplevel)/{binary}.stdin\"\nexit {code}\n"
            ),
        )
        .expect("a stand-in judge");
        make_executable(&path);
    }
    repo
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("the file was written")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("the file can be made executable");
}

/// Run a rendered hook in `repo` as git would, and hand back its status and what it read.
fn run_hook(repo: &Path, hook: GitHook, beds: &[Bed], args: &[&str], stdin: &str) -> i32 {
    use std::io::Write;

    let hooks = repo.join(".githooks");
    std::fs::create_dir_all(&hooks).expect("the hooks directory");
    let path = hooks.join(hook.file_name());
    std::fs::write(&path, githooks::render(hook, beds)).expect("the hook is written");
    make_executable(&path);

    let mut child = spawn_written(&path, repo, args);
    child
        .stdin
        .as_mut()
        .expect("a pipe to the hook")
        .write_all(stdin.as_bytes())
        .expect("the hook reads stdin");
    let output = child.wait_with_output().expect("the hook finishes");
    output.status.code().unwrap_or(-1)
}

/// Execute a script this process just wrote, the way git would.
///
/// These tests run on threads, and each writes a script and then executes it. On Linux a
/// fork on another thread inherits every open descriptor, this thread's write handle to the
/// script included, and holds it until that child's exec closes it (the handles are
/// close-on-exec, but a fork is not an exec). An exec of the script inside that window is
/// refused with `ETXTBSY`, "Text file busy": a race between two tests, not a fault in the
/// script. The window is microseconds, so the exec is tried again for up to a second on that
/// one error and no other, and the script's own refusal is still its own exit status.
fn spawn_written(path: &Path, repo: &Path, args: &[&str]) -> std::process::Child {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    loop {
        let attempt = Command::new(path)
            .current_dir(repo)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        match attempt {
            Ok(child) => return child,
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(error) => panic!("the hook runs: {error}"),
        }
    }
}

/// The race above, made deterministic: a shell holds the script open for writing while it
/// is executed, which is exactly what another test's forked child does by accident. Linux
/// refuses the exec with `ETXTBSY` until the writer lets go; the runner waits it out and the
/// script's own status comes back. macOS does not enforce this rule, so there the test
/// proves only that the runner still runs a script.
#[test]
fn a_script_held_open_for_writing_by_another_process_still_runs_once_the_writer_lets_go() {
    let repo = repository();
    let hooks = repo.path().join(".githooks");
    std::fs::create_dir_all(&hooks).expect("the hooks directory");
    let path = hooks.join("pre-commit");
    std::fs::write(&path, "#!/bin/sh\nexit 7\n").expect("the script is written");
    make_executable(&path);

    // A writer that holds the script open for a third of a second, then exits.
    let mut writer = Command::new("sh")
        .arg("-c")
        .arg(format!("exec 3>>\"{}\"; sleep 0.3", path.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("a shell that holds the script open");
    // Give the shell time to open the descriptor before the exec races it.
    std::thread::sleep(std::time::Duration::from_millis(50));

    let child = spawn_written(&path, repo.path(), &[]);
    let output = child.wait_with_output().expect("the script finishes");
    assert_eq!(output.status.code(), Some(7));
    writer.wait().expect("the writer exits");
}

fn recorded(repo: &Path, name: &str) -> Option<String> {
    std::fs::read_to_string(repo.join(name)).ok()
}

#[test]
fn pre_push_hands_every_guard_the_same_refs_and_the_same_arguments() {
    let repo = repository_with_judges(&[("weeder", 0), ("tend2", 0)]);
    let beds = vec![
        bed("weeder", "0.1.0", Some("weeder"), &[GitHook::PrePush]),
        bed("tend2", "1.0.0", Some("tend2"), &[GitHook::PrePush]),
    ];
    let refs = "refs/heads/master abc123 refs/heads/master def456\n";

    let code = run_hook(
        repo.path(),
        GitHook::PrePush,
        &beds,
        &["origin", "https://example.invalid/garden.git"],
        refs,
    );

    assert_eq!(code, 0);
    for binary in ["weeder", "tend2"] {
        assert_eq!(
            recorded(repo.path(), &format!("{binary}.stdin")).as_deref(),
            Some(refs),
            "{binary} did not read the refs"
        );
        assert_eq!(
            recorded(repo.path(), &format!("{binary}.args")).as_deref(),
            Some("guard pre-push origin https://example.invalid/garden.git"),
            "{binary} did not get the arguments"
        );
    }
}

#[test]
fn the_first_guard_that_refuses_ends_the_hook_with_its_own_status() {
    let repo = repository_with_judges(&[("weeder", 7), ("tend2", 0)]);
    let beds = vec![
        bed("weeder", "0.1.0", Some("weeder"), &[GitHook::PreCommit]),
        bed("tend2", "1.0.0", Some("tend2"), &[GitHook::PreCommit]),
    ];

    let code = run_hook(repo.path(), GitHook::PreCommit, &beds, &[], "");

    assert_eq!(code, 7);
    assert!(recorded(repo.path(), "weeder.args").is_some());
    assert!(
        recorded(repo.path(), "tend2.args").is_none(),
        "the second guard ran after the first refused"
    );
}

#[test]
fn post_commit_seals_when_the_stem_is_fetched_and_passes_by_when_it_is_not() {
    let beds = vec![bed("tend2", "1.0.0", Some("tend2"), &[GitHook::PostCommit])];

    let without = repository_with_judges(&[("tend2", 0)]);
    assert_eq!(
        run_hook(without.path(), GitHook::PostCommit, &beds, &[], ""),
        0
    );
    assert!(recorded(without.path(), "plotplot.args").is_none());
    assert!(recorded(without.path(), "tend2.args").is_some());

    let with = repository_with_judges(&[("plotplot", 0), ("tend2", 0)]);
    assert_eq!(
        run_hook(with.path(), GitHook::PostCommit, &beds, &[], ""),
        0
    );
    assert_eq!(
        recorded(with.path(), "plotplot.args").as_deref(),
        Some("receipt seal")
    );
    assert!(recorded(with.path(), "tend2.args").is_some());
}

#[test]
fn a_hook_whose_judge_was_never_fetched_refuses_rather_than_passing() {
    let repo = repository();
    let beds = vec![bed(
        "weeder",
        "0.1.0",
        Some("weeder"),
        &[GitHook::PreCommit],
    )];

    let code = run_hook(repo.path(), GitHook::PreCommit, &beds, &[], "");

    assert_ne!(code, 0, "a missing judge let the commit through");
}

// ------------------------------------------------------------------ gitconfig

#[test]
fn a_fresh_repository_holds_none_of_the_keys() {
    let repo = repository();
    let current = gitconfig::read(repo.path()).expect("a repository git can read");
    assert!(current.is_empty(), "{current:?}");
    assert_eq!(
        gitconfig::missing(&gitconfig::desired(repo.path()), &current).len(),
        1
    );
}

#[test]
fn applying_the_desired_configuration_writes_all_three_and_repeats_without_duplicating() {
    let repo = repository();
    let desired = gitconfig::desired(repo.path());

    gitconfig::apply(repo.path(), &desired).expect("git accepts the configuration");
    let current = gitconfig::read(repo.path()).expect("a repository git can read");
    for entry in &desired {
        assert!(current.contains(entry), "{entry:?} is not set: {current:?}");
    }
    assert!(gitconfig::missing(&desired, &current).is_empty());
    assert_eq!(values(repo.path(), "core.hooksPath"), [".githooks"]);

    gitconfig::apply(repo.path(), &desired).expect("a second planting");
    for (key, _) in &desired {
        assert_eq!(values(repo.path(), key).len(), 1, "{key} was duplicated");
    }
    let after = gitconfig::read(repo.path()).expect("a repository git can read");
    assert!(gitconfig::missing(&desired, &after).is_empty());
}

#[test]
fn applying_the_desired_entries_leaves_the_remotes_own_refspec_alone_and_adds_none() {
    let repo = tempfile::tempdir().expect("a temporary directory");
    git(repo.path(), &["init", "--quiet"]);
    git(
        repo.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/garden.git",
        ],
    );
    let heads = values(repo.path(), "remote.origin.fetch");
    assert_eq!(heads.len(), 1, "git wrote no default refspec");

    gitconfig::apply(repo.path(), &gitconfig::desired(repo.path())).expect("git accepts it");

    // No refspec for the receipts ref, in either direction: a fetch refspec for a ref the
    // remote lacks fails every fetch, and a push refspec makes git push send that ref alone.
    let fetch = values(repo.path(), "remote.origin.fetch");
    assert_eq!(fetch, heads, "{fetch:?}");
    assert!(values(repo.path(), "remote.origin.push").is_empty());
    assert_eq!(
        values(repo.path(), "core.hooksPath"),
        vec![".githooks".to_owned()]
    );
}

#[test]
fn reading_a_directory_that_is_not_a_repository_is_a_git_error() {
    let outside = tempfile::tempdir().expect("a temporary directory");
    match gitconfig::read(outside.path()) {
        Err(Error::Git { command, stderr }) => {
            assert!(command.starts_with("git "), "{command}");
            assert!(!stderr.is_empty(), "git said nothing about the failure");
        }
        other => panic!("expected Error::Git, got {other:?}"),
    }
}
