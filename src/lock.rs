//! `garden.lock`, the pinned judges.
//!
//! A stamp's meaning depends on the judge's version, so the lock is what makes proof
//! reproducible. It is TOML at rest; the contracts' schema describes the JSON a conforming
//! TOML parser produces from it, so the stem parses the TOML to a JSON value, enforces the
//! schema on that value, and only then reads the model.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::bundle::make_executable;
use crate::cli::LockFace;
use crate::error::{Error, Result};
use crate::fetch::{Fetch, Https};
use crate::layout;
use crate::manifest::{self, GitInstall, compile, one_line};

/// The contracts' lock schema, v1.3.0, embedded so the binary carries its own contract.
pub const LOCK_SCHEMA: &str = include_str!("../contracts/lock.schema.json");

/// How a refusal by the contracts opens, as opposed to a TOML syntax failure.
pub const SCHEMA_REFUSED: &str = "contracts/lock.schema.json refused it";

/// The lockfile a planted repository carries.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Lock {
    /// The season identity `contracts/identifiers.md` pins: `yyyy.mm`.
    pub season: String,
    pub judges: BTreeMap<String, Judge>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// One pinned judge, keyed in the lock by its `garden.json` name.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Judge {
    pub version: String,
    /// Present when the judge is resolved through npm rather than a checksummed binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub platforms: BTreeMap<String, Artifact>,
    /// Present when the judge is fetched by cloning at a pinned commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInstall>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// One platform's pinned artifact. The stem refuses to run bytes whose digest differs.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Artifact {
    pub url: String,
    /// Lowercase hex, 64 characters.
    pub sha256: String,
}

/// Read a `garden.lock`.
///
/// # Errors
///
/// [`Error::Toml`] when the text is not TOML, when the contracts' schema refuses the value
/// it produces, or when the stem cannot read the schema-valid value.
pub fn parse_lock(toml: &str) -> Result<Lock> {
    parse_lock_at(Path::new(layout::GARDEN_LOCK), toml)
}

/// Read a `garden.lock` that came from a known path, so a failure can name it.
///
/// # Errors
///
/// As [`parse_lock`], with `path` in the error.
pub fn parse_lock_at(path: &Path, toml: &str) -> Result<Lock> {
    let refuse = |message: String| Error::Toml {
        path: path.to_path_buf(),
        message,
    };

    let value: Value =
        toml::from_str(toml).map_err(|error| refuse(one_line(&error.to_string())))?;

    let validator = lock_validator().as_ref().map_err(|problem| {
        refuse(format!(
            "the embedded lock schema did not compile: {problem}"
        ))
    })?;
    if let Err(error) = validator.validate(&value) {
        let at = error.instance_path().to_string();
        let where_ = if at.is_empty() {
            String::new()
        } else {
            format!(" at {at}")
        };
        return Err(refuse(one_line(&format!(
            "{SCHEMA_REFUSED}{where_}: {error}"
        ))));
    }

    serde_json::from_value(value)
        .map_err(|error| refuse(one_line(&format!("the stem could not read it: {error}"))))
}

/// The lock at `root`, or `None` when the repository carries none.
///
/// # Errors
///
/// [`Error::Io`] when the file exists but cannot be read, and whatever [`parse_lock_at`]
/// refuses.
pub fn read_lock(root: &Path) -> Result<Option<Lock>> {
    let path = layout::garden_lock(root);
    let toml = match std::fs::read_to_string(&path) {
        Ok(toml) => toml,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(Error::Io { path, source }),
    };
    parse_lock_at(&path, &toml).map(Some)
}

/// The Rust target triple this binary was built for, as `build.rs` read it from cargo.
///
/// The lock keys its artifacts by this triple, so the judge that gets fetched is the one
/// built for the machine actually running.
pub fn platform() -> &'static str {
    env!("PLOTPLOT_TARGET")
}

/// The compiled lock schema, built once.
fn lock_validator() -> &'static std::result::Result<jsonschema::Validator, String> {
    static VALIDATOR: OnceLock<std::result::Result<jsonschema::Validator, String>> =
        OnceLock::new();
    VALIDATOR.get_or_init(|| compile(LOCK_SCHEMA))
}

// ------------------------------------------------------------------- verifying the lock

/// What one judge turned out to be when `lock verify` looked at it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeState {
    /// Already on disk, out of the bytes the lock pins. Nothing was fetched.
    Verified,
    /// Fetched now; its digest matched the lock and it is placed. `manifest` says whether
    /// the artifact carried a `garden.json` to cache beside it.
    Fetched { manifest: bool },
    /// The lock pins nothing this machine can resolve, and the reason says what is absent.
    Missing(String),
    /// The bytes are not the bytes the lock pins. Nothing was written, nothing replaced.
    Mismatch { expected: String, actual: String },
}

/// One judge's line of `plotplot lock verify`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeReport {
    pub name: String,
    pub version: String,
    pub state: JudgeState,
}

/// How a line reads when the artifact's digest held but it carried no manifest.
pub const NO_MANIFEST: &str = "no manifest in artifact";

/// Resolve every judge the lock pins for `platform`, fetching what is not already there.
///
/// Judges are answered in name order, which is the order [`Lock::judges`] holds them in, so
/// the output of two runs on the same lock is comparable line by line.
///
/// A judge with `platforms` is resolved through them even when it also names an npm
/// package: the checksummed artifact is the stronger pin, and a platform it does not cover
/// is [`JudgeState::Missing`] rather than a quiet fall back to bytes nobody pinned.
///
/// The lock is the truth and this function never edits it: a digest that does not match
/// yields [`JudgeState::Mismatch`] with nothing fetched, written or replaced, and the caller
/// decides what to do about it.
///
/// # Errors
///
/// [`Error::Fetch`] when a url gave up no bytes, [`Error::Archive`] when an artifact could
/// not be unpacked or is in a format the stem does not unpack, [`Error::Io`] when a file
/// could not be read or written, [`Error::Bed`] when npm refused an install,
/// [`Error::Git`] when a clone or checkout failed, and whatever a `garden.json` inside an
/// artifact refuses.
pub fn verify(
    root: &Path,
    lock: &Lock,
    platform: &str,
    fetch: &dyn Fetch,
) -> Result<Vec<JudgeReport>> {
    let mut reports = Vec::with_capacity(lock.judges.len());
    for (name, judge) in &lock.judges {
        reports.push(JudgeReport {
            name: name.clone(),
            version: judge.version.clone(),
            state: judge_state(root, name, judge, platform, fetch)?,
        });
    }
    Ok(reports)
}

/// One judge's line: `<name> <version> <state>`.
pub fn line(report: &JudgeReport) -> String {
    let state = match &report.state {
        JudgeState::Verified => "verified".to_owned(),
        JudgeState::Fetched { manifest: true } => "fetched".to_owned(),
        JudgeState::Fetched { manifest: false } => format!("fetched, {NO_MANIFEST}"),
        JudgeState::Missing(reason) => format!("missing: {reason}"),
        JudgeState::Mismatch { expected, actual } => {
            format!("mismatch: expected {expected}, got {actual}")
        }
    };
    format!("{} {} {state}", report.name, report.version)
}

/// Whether every judge is one the stem may now run.
pub fn resolved(reports: &[JudgeReport]) -> bool {
    reports.iter().all(|report| {
        matches!(
            report.state,
            JudgeState::Verified | JudgeState::Fetched { .. }
        )
    })
}

/// Which of the three ways a judge is pinned applies, and the result of taking it.
fn judge_state(
    root: &Path,
    name: &str,
    judge: &Judge,
    platform: &str,
    fetch: &dyn Fetch,
) -> Result<JudgeState> {
    if !judge.platforms.is_empty() {
        return from_artifact(root, name, judge, platform, fetch);
    }
    if let Some(package) = &judge.npm {
        return from_npm(root, name, judge, package);
    }
    if let Some(git) = &judge.git {
        return from_git(root, name, git);
    }
    // The contracts refuse such an entry, so this is only reachable from a `Lock` built in
    // memory. It is still answered rather than assumed away.
    Ok(JudgeState::Missing(
        "the lock pins no artifact, npm package or clone for it".to_owned(),
    ))
}

/// A judge pinned as a per-platform artifact with a digest: the reproducible case.
fn from_artifact(
    root: &Path,
    name: &str,
    judge: &Judge,
    platform: &str,
    fetch: &dyn Fetch,
) -> Result<JudgeState> {
    let Some(artifact) = judge.platforms.get(platform) else {
        return Ok(JudgeState::Missing(format!(
            "the lock pins no artifact for {platform}"
        )));
    };

    // What the judge is called on disk is what the cached manifest says, so a second run
    // finds the file the first run placed even when that name is not the lock's key.
    let placed = cached_binary_name(root, name)?;
    if let Some(recorded) = read_if_present(&layout::judge_digest(root, &placed))? {
        let recorded = recorded.trim().to_owned();
        if recorded != artifact.sha256 {
            return Ok(JudgeState::Mismatch {
                expected: artifact.sha256.clone(),
                actual: recorded,
            });
        }
        if layout::judge_binary(root, &placed).is_file() {
            return Ok(JudgeState::Verified);
        }
    }

    let bytes = fetch.fetch(&artifact.url)?;
    let digest = digest_of(&bytes);
    if digest != artifact.sha256 {
        return Ok(JudgeState::Mismatch {
            expected: artifact.sha256.clone(),
            actual: digest,
        });
    }

    let artifact_dir = layout::bed_artifact(root, name);
    unpack(&bytes, &artifact.url, &artifact_dir)?;
    place(root, name, &artifact_dir, &digest)
}

/// Copy the executable and the manifest out of an unpacked artifact into the planted tree.
fn place(root: &Path, name: &str, artifact_dir: &Path, digest: &str) -> Result<JudgeState> {
    let carried_at = artifact_dir.join(layout::GARDEN_JSON);
    let carried = read_if_present(&carried_at)?;
    let binary = match &carried {
        Some(json) => binary_named_by(&carried_at, json, name)?,
        None => name.to_owned(),
    };

    let source = artifact_dir.join(&binary);
    if !source.is_file() {
        return Err(Error::Archive {
            path: artifact_dir.to_path_buf(),
            problem: format!("carries no executable named {binary} at its root"),
        });
    }

    let placed = layout::judge_binary(root, &binary);
    make_parent(&placed)?;
    std::fs::copy(&source, &placed).map_err(|source| Error::Io {
        path: placed.clone(),
        source,
    })?;
    make_executable(&placed)?;
    write_file(&layout::judge_digest(root, &binary), &format!("{digest}\n"))?;

    if let Some(json) = &carried {
        write_file(&layout::bed_manifest(root, name), json)?;
    }

    Ok(JudgeState::Fetched {
        manifest: carried.is_some(),
    })
}

/// A judge resolved through npm: install into a prefix of its own, then link its bin where
/// the dispatcher and the git hooks look for a judge.
fn from_npm(root: &Path, name: &str, judge: &Judge, package: &str) -> Result<JudgeState> {
    let placed = cached_binary_name(root, name)?;
    if layout::judge_binary(root, &placed).exists() {
        return Ok(JudgeState::Verified);
    }

    let prefix = layout::npm_prefix(root, name);
    make_dir(&prefix)?;
    let spec = format!("{package}@{}", judge.version);
    let output = Command::new("npm")
        .args(["install", "--prefix"])
        .arg(&prefix)
        // `--` so a package name that reads as an option is a package name and nothing else.
        .arg("--")
        .arg(&spec)
        .output()
        .map_err(|source| Error::Bed {
            bed: name.to_owned(),
            problem: format!("npm install could not be run: {source}"),
        })?;
    if !output.status.success() {
        return Err(Error::Bed {
            bed: name.to_owned(),
            problem: format!(
                "npm install {spec} exited {}: {}",
                exit_of(&output.status),
                one_line(&String::from_utf8_lossy(&output.stderr))
            ),
        });
    }

    let package_root = prefix.join("node_modules").join(package);
    let carried_at = package_root.join(layout::GARDEN_JSON);
    let carried = read_if_present(&carried_at)?;
    let binary = match &carried {
        Some(json) => binary_named_by(&carried_at, json, name)?,
        None => name.to_owned(),
    };

    let target = npm_bin(&package_root, &binary, name)?;
    let link = layout::judge_binary(root, &binary);
    make_parent(&link)?;
    symlink(&target, &link)?;

    if let Some(json) = &carried {
        write_file(&layout::bed_manifest(root, name), json)?;
    }

    Ok(JudgeState::Fetched {
        manifest: carried.is_some(),
    })
}

/// The file an installed npm package offers as `binary`, from its own `package.json`.
///
/// # Errors
///
/// [`Error::Bed`] when the package declares no bin, declares several and none of them is
/// the name the manifest asked for, or names a bin that is not on disk.
fn npm_bin(package_root: &Path, binary: &str, name: &str) -> Result<PathBuf> {
    let refuse = |problem: String| Error::Bed {
        bed: name.to_owned(),
        problem,
    };

    let path = package_root.join("package.json");
    let json = read_if_present(&path)?
        .ok_or_else(|| refuse(format!("npm installed no {}", path.display())))?;
    let document: Value = serde_json::from_str(&json).map_err(|source| Error::Json {
        path: Some(path.clone()),
        source,
    })?;

    let relative = match document.get("bin") {
        Some(Value::String(only)) => only.clone(),
        Some(Value::Object(bins)) => match bins.get(binary).and_then(Value::as_str) {
            Some(named) => named.to_owned(),
            None => match bins.iter().collect::<Vec<_>>().as_slice() {
                [(_, Value::String(only))] => only.clone(),
                _ => {
                    return Err(refuse(format!(
                        "its package.json declares no bin named {binary}"
                    )));
                }
            },
        },
        _ => return Err(refuse("its package.json declares no bin".to_owned())),
    };

    let bin = package_root.join(relative);
    if !bin.is_file() {
        return Err(refuse(format!(
            "its package.json names {} as a bin, and that file is not there",
            bin.display()
        )));
    }
    // An absolute target, because the link lives in .plotplot/bin/ and is followed from
    // whatever directory a hook happens to run in.
    std::fs::canonicalize(&bin).map_err(|source| Error::Io { path: bin, source })
}

/// A judge pinned as a clone at a commit. There is no release artifact to checksum, so what
/// stands in for the digest is the commit the clone is standing on.
fn from_git(root: &Path, name: &str, git: &GitInstall) -> Result<JudgeState> {
    let artifact_dir = layout::bed_artifact(root, name);
    if artifact_dir.join(".git").exists() {
        let head = git_at(&artifact_dir, &["rev-parse", "HEAD"])?;
        let head = head.trim().to_owned();
        return Ok(if head == git.rev {
            JudgeState::Verified
        } else {
            JudgeState::Mismatch {
                expected: git.rev.clone(),
                actual: head,
            }
        });
    }

    // The contracts already require a full commit sha, and this says so again where the
    // value is about to become an argument: a rev that could read as an option would make
    // git do something other than what the lock asked for.
    if git.rev.len() != 40 || !git.rev.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::Git {
            command: format!("git checkout {}", git.rev),
            stderr: "the lock's rev is not a full commit sha, and the stem checks out nothing else"
                .to_owned(),
        });
    }

    make_parent(&artifact_dir)?;
    let into = artifact_dir.display().to_string();
    // `--` so a url that reads as an option is a url and nothing else.
    git_run(
        None,
        &["clone", "--quiet", "--", git.url.as_str(), into.as_str()],
    )?;
    git_at(
        &artifact_dir,
        &["checkout", "--quiet", "--detach", &git.rev],
    )?;

    let carried_at = artifact_dir.join(layout::GARDEN_JSON);
    let carried = read_if_present(&carried_at)?;
    if let Some(json) = &carried {
        write_file(&layout::bed_manifest(root, name), json)?;
    }
    Ok(JudgeState::Fetched {
        manifest: carried.is_some(),
    })
}

/// `plotplot lock <face>`: the lockfile wrapper as a command.
pub fn run(root: &Path, face: &LockFace, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match face {
        LockFace::Verify => verify_face(root, &Https, stdout, stderr),
    }
}

/// `plotplot lock verify`: one line per judge, exit 0 when every judge is resolved, 3 when
/// any is missing or mismatched, 1 when the run could not reach a verdict at all.
///
/// The fetcher is a parameter so the face itself, and not only the resolving underneath it,
/// can be run against bytes a test supplies.
pub fn verify_face(
    root: &Path,
    fetch: &dyn Fetch,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let lock = match read_lock(root) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            let _ = writeln!(
                stderr,
                "{} is not there, so there are no judges to verify",
                layout::garden_lock(root).display()
            );
            return 1;
        }
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return 1;
        }
    };

    let reports = match verify(root, &lock, platform(), fetch) {
        Ok(reports) => reports,
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return 1;
        }
    };

    for report in &reports {
        if let Err(error) = writeln!(stdout, "{}", line(report)) {
            let _ = writeln!(stderr, "stdout: {error}");
            return 1;
        }
    }

    if resolved(&reports) { 0 } else { 3 }
}

// ---------------------------------------------------------------------------- the edge

/// Unpack a fetched artifact into `into`.
///
/// tar's own unpacker is what writes the files, and it refuses an entry whose path would
/// leave the destination, so a hostile tarball cannot write outside the planted tree.
///
/// The destination is written over rather than emptied first: the garden deletes with
/// `trash` and never with `rm`, and a re-fetch only happens once the record beside the judge
/// is gone, which is a thing a person did on purpose.
///
/// # Errors
///
/// [`Error::Archive`] naming the url when the format is one the stem does not unpack or the
/// bytes are not the archive they claim to be, [`Error::Io`] when the destination cannot be
/// made.
fn unpack(bytes: &[u8], url: &str, into: &Path) -> Result<()> {
    let refuse = |problem: String| Error::Archive {
        path: PathBuf::from(url),
        problem,
    };

    if url.ends_with(".zip") {
        return Err(refuse(
            "zip is not unpacked on this platform; the stem has no Windows target yet".to_owned(),
        ));
    }
    if !(url.ends_with(".tar.gz") || url.ends_with(".tgz")) {
        return Err(refuse(
            "the stem unpacks .tar.gz artifacts, and this url names none".to_owned(),
        ));
    }

    make_dir(into)?;
    tar::Archive::new(flate2::read::GzDecoder::new(bytes))
        .unpack(into)
        .map_err(|error| refuse(one_line(&error.to_string())))
}

/// The sha256 of some bytes, as the lowercase hex the lock spells.
fn digest_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// A file's text, or `None` when it is not there.
fn read_if_present(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// The executable's file name a `garden.json` declares, falling back to the lock's key for
/// a bed that names neither a cli nor a binary name.
fn binary_named_by(path: &Path, json: &str, name: &str) -> Result<String> {
    let parsed = manifest::parse_manifest_at(Some(path), json)?;
    Ok(manifest::to_bed(&parsed)?
        .binary
        .unwrap_or_else(|| name.to_owned()))
}

/// What this judge is called under `.plotplot/bin/` according to what is already cached: its
/// manifest's binary name, or the lock's key when no manifest has been cached yet.
fn cached_binary_name(root: &Path, name: &str) -> Result<String> {
    let path = layout::bed_manifest(root, name);
    match read_if_present(&path)? {
        Some(json) => binary_named_by(&path, &json, name),
        None => Ok(name.to_owned()),
    }
}

fn make_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn make_parent(path: &Path) -> Result<()> {
    match path.parent() {
        Some(parent) => make_dir(parent),
        None => Ok(()),
    }
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    make_parent(path)?;
    std::fs::write(path, contents).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// The exit status of a process as a number, or the word for a signal that carried none.
fn exit_of(status: &std::process::ExitStatus) -> String {
    match status.code() {
        Some(code) => code.to_string(),
        None => "on a signal".to_owned(),
    }
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).map_err(|source| Error::Io {
        path: link.to_path_buf(),
        source,
    })
}

/// A platform without symlinks cannot be given one, and the stem says so rather than
/// leaving a judge nothing can call.
#[cfg(not(unix))]
fn symlink(_target: &Path, link: &Path) -> Result<()> {
    Err(Error::Io {
        path: link.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "this platform has no symlinks, so the stem cannot place an npm-resolved judge",
        ),
    })
}

/// Run git, in `at` when one is given, and hand back its stdout.
///
/// # Errors
///
/// [`Error::Git`] when git could not be run or exited non-zero, carrying what it said.
fn git_run(at: Option<&Path>, args: &[&str]) -> Result<String> {
    let mut command = Command::new("git");
    if let Some(at) = at {
        command.arg("-C").arg(at);
    }
    command.args(args);

    let spelled = || format!("git {}", args.join(" "));
    let output = command.output().map_err(|source| Error::Git {
        command: spelled(),
        stderr: one_line(&source.to_string()),
    })?;
    if !output.status.success() {
        return Err(Error::Git {
            command: spelled(),
            stderr: one_line(&String::from_utf8_lossy(&output.stderr)),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn git_at(at: &Path, args: &[&str]) -> Result<String> {
    git_run(Some(at), args)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn fixture() -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/fixtures/garden.lock");
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn toml_error(text: &str) -> String {
        match parse_lock(text) {
            Err(Error::Toml { path, message }) => {
                assert_eq!(path, PathBuf::from(layout::GARDEN_LOCK));
                message
            }
            other => panic!("expected Error::Toml, got {other:?}"),
        }
    }

    /// The triple this test binary was compiled for, derived from `cfg!` rather than from
    /// the build script, so it is an independent witness of what `platform()` reports.
    fn triple_from_cfg() -> String {
        let arch = std::env::consts::ARCH;
        let vendor = if cfg!(target_vendor = "apple") {
            "apple"
        } else if cfg!(target_vendor = "pc") {
            "pc"
        } else if cfg!(target_vendor = "unknown") {
            "unknown"
        } else {
            panic!("this test knows the apple, pc and unknown vendors; add this one")
        };
        let system = match std::env::consts::OS {
            "macos" | "ios" => "darwin",
            other => other,
        };
        let abi = if cfg!(target_env = "musl") {
            "musl"
        } else if cfg!(target_env = "gnu") {
            "gnu"
        } else if cfg!(target_env = "msvc") {
            "msvc"
        } else {
            ""
        };
        if abi.is_empty() {
            format!("{arch}-{vendor}-{system}")
        } else {
            format!("{arch}-{vendor}-{system}-{abi}")
        }
    }

    #[test]
    fn the_fixture_lock_parses_into_its_season_and_its_judges() {
        let lock = parse_lock(&fixture()).expect("the fixture lock");
        assert_eq!(lock.season, "2026.09");
        assert_eq!(
            lock.judges.keys().map(String::as_str).collect::<Vec<_>>(),
            ["tend2", "tilth"]
        );

        let tilth = lock.judges.get("tilth").expect("the tilth judge");
        assert_eq!(tilth.version, "0.10.1");
        assert_eq!(tilth.npm, None);
        assert_eq!(tilth.git, None);
        assert_eq!(
            tilth
                .platforms
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "aarch64-apple-darwin",
                "aarch64-unknown-linux-musl",
                "x86_64-apple-darwin",
                "x86_64-unknown-linux-musl",
            ]
        );
        let darwin = tilth
            .platforms
            .get("aarch64-apple-darwin")
            .expect("the darwin artifact");
        assert_eq!(
            darwin.url,
            "https://github.com/jahala/tilth/releases/download/v0.10.1/tilth-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            darwin.sha256,
            "38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa55"
        );
        let linux = tilth
            .platforms
            .get("x86_64-unknown-linux-musl")
            .expect("the linux artifact");
        assert_eq!(
            linux.url,
            "https://github.com/jahala/tilth/releases/download/v0.10.1/tilth-x86_64-unknown-linux-musl.tar.gz"
        );
        assert_eq!(
            linux.sha256,
            "3df16574ed9fc33e4d9c63107994c78d48f93ab0099a37232b4410223e2ad2ea"
        );

        let tend2 = lock.judges.get("tend2").expect("the tend2 judge");
        assert_eq!(tend2.version, "1.0.0");
        assert_eq!(tend2.npm.as_deref(), Some("@plotplot/tend2"));
        assert!(tend2.platforms.is_empty());
    }

    #[test]
    fn a_digest_that_is_not_sixty_four_lowercase_hex_is_refused() {
        for bad in [
            "38C36E471F61D5A7101D9E0F9DFD401939C5A363E0F4A96384B86F43A662FA55",
            "38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa5",
            "sha256-38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa55",
            "",
        ] {
            let text = fixture().replace(
                "38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa55",
                bad,
            );
            let message = toml_error(&text);
            assert!(message.starts_with(SCHEMA_REFUSED), "{bad}: {message}");
            assert!(message.contains("sha256"), "{bad}: {message}");
        }
    }

    #[test]
    fn a_lock_without_judges_is_refused() {
        let message = toml_error("season = \"2026.09\"\n");
        assert!(message.starts_with(SCHEMA_REFUSED), "{message}");
        assert!(message.contains("judges"), "{message}");
    }

    #[test]
    fn a_lock_with_no_judge_in_judges_is_refused() {
        let message = toml_error("season = \"2026.09\"\n\n[judges]\n");
        assert!(message.starts_with(SCHEMA_REFUSED), "{message}");
    }

    #[test]
    fn a_season_that_is_not_a_season_is_refused() {
        for bad in ["2026-09", "2026.9", "2026.13", "september"] {
            let text = fixture().replace("season = \"2026.09\"", &format!("season = \"{bad}\""));
            let message = toml_error(&text);
            assert!(message.starts_with(SCHEMA_REFUSED), "{bad}: {message}");
        }
    }

    #[test]
    fn a_judge_with_neither_npm_nor_an_artifact_is_refused() {
        let message = toml_error("season = \"2026.09\"\n\n[judges.weeder]\nversion = \"0.1.0\"\n");
        assert!(message.starts_with(SCHEMA_REFUSED), "{message}");
    }

    #[test]
    fn text_that_is_not_toml_is_refused_without_naming_the_schema() {
        let message = toml_error("season = \n");
        assert!(!message.starts_with(SCHEMA_REFUSED), "{message}");
        assert!(!message.is_empty());
    }

    #[test]
    fn read_lock_on_a_root_without_a_lock_is_none() {
        let root = tempfile::tempdir().expect("a temp root");
        assert!(read_lock(root.path()).expect("no lock").is_none());
    }

    #[test]
    fn read_lock_reads_the_lock_at_the_root() {
        let root = tempfile::tempdir().expect("a temp root");
        std::fs::write(layout::garden_lock(root.path()), fixture()).expect("the lock");
        let lock = read_lock(root.path())
            .expect("a readable lock")
            .expect("a lock at the root");
        assert_eq!(lock.season, "2026.09");
        assert_eq!(lock.judges.len(), 2);
    }

    #[test]
    fn read_lock_names_the_lock_it_could_not_read() {
        let root = tempfile::tempdir().expect("a temp root");
        let path = layout::garden_lock(root.path());
        std::fs::write(&path, "season = \n").expect("a broken lock");
        match read_lock(root.path()) {
            Err(Error::Toml { path: named, .. }) => assert_eq!(named, path),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn platform_is_the_triple_cargo_built_for() {
        let platform = platform();
        assert!(!platform.is_empty());
        assert_eq!(platform, triple_from_cfg());
        let parts = platform.split('-').count();
        assert!((3..=5).contains(&parts), "{platform}");
    }

    #[test]
    fn a_judge_pinned_by_a_commit_keeps_its_pin() {
        let text = concat!(
            "season = \"2026.09\"\n\n",
            "[judges.petals]\n",
            "version = \"0.1.0\"\n\n",
            "[judges.petals.git]\n",
            "url = \"https://github.com/jahala/petals.git\"\n",
            "rev = \"af3129f2858ce211468d99ff96e74631ac92be71\"\n"
        );
        let lock = parse_lock(text).expect("a lock pinned by commit");
        let petals = lock.judges.get("petals").expect("the petals judge");
        assert_eq!(
            petals.git.as_ref().map(|git| git.rev.as_str()),
            Some("af3129f2858ce211468d99ff96e74631ac92be71")
        );
    }

    #[test]
    fn unknown_lock_fields_are_kept_rather_than_dropped() {
        // A key after the first table would belong to that table, so it goes at the top.
        let text = format!("next_season = \"2026.10\"\n{}", fixture());
        let lock = parse_lock(&text).expect("a lock with a field the stem does not read");
        assert_eq!(
            lock.extra.get("next_season"),
            Some(&serde_json::json!("2026.10"))
        );
    }
}

#[cfg(test)]
mod verify_tests {
    use std::path::PathBuf;

    use super::*;
    use crate::fetch::FromMap;

    /// The platform the fixture locks in these tests pin, whatever machine runs them: what
    /// is under test is the resolving, not this build's own triple.
    const PLATFORM: &str = "aarch64-apple-darwin";
    const OTHER_PLATFORM: &str = "x86_64-unknown-linux-musl";
    const URL: &str = "https://example.invalid/weeder-aarch64-apple-darwin.tar.gz";

    /// The judge's own executable, small and real: it reads a payload and allows, which is
    /// the bed contract the umbrella ruled on 2026-09-08.
    const EXECUTABLE: &[u8] = b"#!/bin/sh\ncat >/dev/null\nexit 0\n";
    const SKILL: &[u8] = b"# weeder\n\nThe judge of the diff.\n";

    fn weeder_manifest() -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("contracts/fixtures/manifest/weeder.garden.json");
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// A .tar.gz built here, entry by entry, so the bytes the fetcher serves are a real
    /// archive and not a recording of one.
    fn tarball(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        for (name, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_mode(0o755);
            header.set_mtime(0);
            header.set_size(bytes.len() as u64);
            builder
                .append_data(&mut header, name, *bytes)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        builder
            .into_inner()
            .expect("the tar closes")
            .finish()
            .expect("the gzip closes")
    }

    /// The artifact the tests fetch: the executable, the manifest and the skill at its root.
    fn artifact() -> Vec<u8> {
        tarball(&[
            ("weeder", EXECUTABLE),
            ("garden.json", weeder_manifest().as_bytes()),
            ("SKILL.md", SKILL),
        ])
    }

    fn lock_pinning(judge: &str, platform: &str, url: &str, sha256: &str) -> Lock {
        let mut platforms = BTreeMap::new();
        platforms.insert(
            platform.to_owned(),
            Artifact {
                url: url.to_owned(),
                sha256: sha256.to_owned(),
            },
        );
        let mut judges = BTreeMap::new();
        judges.insert(
            judge.to_owned(),
            Judge {
                version: "0.1.0".to_owned(),
                npm: None,
                platforms,
                git: None,
                extra: BTreeMap::new(),
            },
        );
        Lock {
            season: "2026.09".to_owned(),
            judges,
            extra: BTreeMap::new(),
        }
    }

    fn mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .permissions()
            .mode()
    }

    fn only(reports: &[JudgeReport]) -> &JudgeReport {
        match reports {
            [one] => one,
            other => panic!("expected one report, got {other:?}"),
        }
    }

    /// Every file under `.plotplot/bin/`, sorted; empty when the directory is not there.
    fn placed(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = match std::fs::read_dir(layout::bin_dir(root)) {
            Ok(entries) => entries
                .filter_map(std::result::Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names
    }

    #[test]
    fn a_matching_artifact_is_fetched_unpacked_placed_and_its_manifest_cached() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = artifact();
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a verified lock");
        assert_eq!(only(&reports).state, JudgeState::Fetched { manifest: true });
        assert_eq!(fetcher.calls(), [URL]);

        let binary = layout::judge_binary(root.path(), "weeder");
        assert!(binary.is_file(), "{} is not there", binary.display());
        assert_eq!(mode(&binary) & 0o111, 0o111, "the judge is not executable");
        assert_eq!(
            std::fs::read(&binary).expect("the placed judge"),
            EXECUTABLE
        );

        let recorded = std::fs::read_to_string(layout::judge_digest(root.path(), "weeder"))
            .expect("the digest companion");
        assert_eq!(recorded.trim(), digest);

        let cached = std::fs::read_to_string(layout::bed_manifest(root.path(), "weeder"))
            .expect("the cached manifest");
        assert_eq!(cached, weeder_manifest());

        // The whole artifact is kept, not only what was copied out of it.
        let skill = layout::bed_artifact(root.path(), "weeder").join("SKILL.md");
        assert_eq!(std::fs::read(&skill).expect("the skill"), SKILL);
    }

    #[test]
    fn a_second_verify_is_verified_and_fetches_nothing() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = artifact();
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        verify(root.path(), &lock, PLATFORM, &fetcher).expect("the first verify");
        assert_eq!(fetcher.calls().len(), 1);

        let again = verify(root.path(), &lock, PLATFORM, &fetcher).expect("the second verify");
        assert_eq!(only(&again).state, JudgeState::Verified);
        assert_eq!(fetcher.calls().len(), 1, "the second verify fetched again");
    }

    #[test]
    fn a_digest_the_bytes_do_not_have_is_a_mismatch_and_places_nothing() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = artifact();
        let real = digest_of(&bytes);
        let wrong = "0".repeat(64);
        let lock = lock_pinning("weeder", PLATFORM, URL, &wrong);
        let fetcher = FromMap::serving(URL, bytes);

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a report");
        assert_eq!(
            only(&reports).state,
            JudgeState::Mismatch {
                expected: wrong,
                actual: real,
            }
        );
        assert!(placed(root.path()).is_empty(), "{:?}", placed(root.path()));
        assert!(!layout::bed_manifest(root.path(), "weeder").exists());
    }

    #[test]
    fn a_companion_that_differs_is_a_mismatch_without_fetching_anything() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = artifact();
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        let stale = "1".repeat(64);
        write_file(&layout::judge_binary(root.path(), "weeder"), "an old judge")
            .expect("a judge already on disk");
        write_file(
            &layout::judge_digest(root.path(), "weeder"),
            &format!("{stale}\n"),
        )
        .expect("its companion");

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a report");
        assert_eq!(
            only(&reports).state,
            JudgeState::Mismatch {
                expected: digest,
                actual: stale,
            }
        );
        assert!(
            fetcher.calls().is_empty(),
            "the lock is the truth, not the disk"
        );
        assert_eq!(
            std::fs::read_to_string(layout::judge_binary(root.path(), "weeder"))
                .expect("the judge on disk"),
            "an old judge",
            "the judge was replaced"
        );
    }

    #[test]
    fn a_platform_the_judge_does_not_cover_is_missing_and_names_it() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = artifact();
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", OTHER_PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a report");
        match &only(&reports).state {
            JudgeState::Missing(reason) => assert!(reason.contains(PLATFORM), "{reason}"),
            other => panic!("got {other:?}"),
        }
        assert!(fetcher.calls().is_empty());
    }

    #[test]
    fn an_artifact_with_no_manifest_is_fetched_and_says_so() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = tarball(&[("tilth", EXECUTABLE)]);
        let digest = digest_of(&bytes);
        let lock = lock_pinning("tilth", PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a report");
        assert_eq!(
            only(&reports).state,
            JudgeState::Fetched { manifest: false }
        );
        assert_eq!(placed(root.path()), ["tilth", "tilth.sha256"]);
        assert!(
            !layout::bed_manifest(root.path(), "tilth").exists(),
            "an artifact with no manifest cached one anyway"
        );
    }

    #[test]
    fn the_executable_is_the_one_the_manifest_names_and_not_the_lock_key() {
        let root = tempfile::tempdir().expect("a temp root");
        let manifest = weeder_manifest().replace(
            "\"install\": {\n    \"cargo\": \"weeder\",",
            "\"install\": {\n    \"binary_name\": \"weeder-cli\",\n    \"cargo\": \"weeder\",",
        );
        assert!(manifest.contains("weeder-cli"), "the fixture edit missed");
        let bytes = tarball(&[
            ("weeder-cli", EXECUTABLE),
            ("garden.json", manifest.as_bytes()),
        ]);
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a report");
        assert_eq!(only(&reports).state, JudgeState::Fetched { manifest: true });
        assert_eq!(placed(root.path()), ["weeder-cli", "weeder-cli.sha256"]);

        // And the second run finds it under that name rather than under the lock's key.
        let again = verify(root.path(), &lock, PLATFORM, &fetcher).expect("the second verify");
        assert_eq!(only(&again).state, JudgeState::Verified);
        assert_eq!(fetcher.calls().len(), 1);
    }

    #[test]
    fn an_artifact_missing_the_executable_it_names_is_refused_by_path() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = tarball(&[("garden.json", weeder_manifest().as_bytes())]);
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", PLATFORM, URL, &digest);
        let fetcher = FromMap::serving(URL, bytes);

        match verify(root.path(), &lock, PLATFORM, &fetcher) {
            Err(Error::Archive { path, problem }) => {
                assert_eq!(path, layout::bed_artifact(root.path(), "weeder"));
                assert!(problem.contains("weeder"), "{problem}");
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn a_zip_artifact_is_refused_by_name_rather_than_half_unpacked() {
        let root = tempfile::tempdir().expect("a temp root");
        let url = "https://example.invalid/weeder-x86_64-pc-windows-msvc.zip";
        let bytes = artifact();
        let digest = digest_of(&bytes);
        let lock = lock_pinning("weeder", PLATFORM, url, &digest);
        let fetcher = FromMap::serving(url, bytes);

        match verify(root.path(), &lock, PLATFORM, &fetcher) {
            Err(Error::Archive { path, problem }) => {
                assert_eq!(path, PathBuf::from(url));
                assert!(problem.contains("zip"), "{problem}");
            }
            other => panic!("got {other:?}"),
        }
        assert!(placed(root.path()).is_empty());
    }

    #[test]
    fn a_url_that_gives_up_no_bytes_is_an_error_and_not_a_verdict() {
        let root = tempfile::tempdir().expect("a temp root");
        let lock = lock_pinning("weeder", PLATFORM, URL, &"0".repeat(64));
        let fetcher = FromMap::serving("https://example.invalid/other.tar.gz", Vec::new());

        match verify(root.path(), &lock, PLATFORM, &fetcher) {
            Err(Error::Fetch { url, .. }) => assert_eq!(url, URL),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn a_judge_pinned_by_nothing_at_all_is_missing_rather_than_a_panic() {
        let root = tempfile::tempdir().expect("a temp root");
        let mut judges = BTreeMap::new();
        judges.insert(
            "weeder".to_owned(),
            Judge {
                version: "0.1.0".to_owned(),
                npm: None,
                platforms: BTreeMap::new(),
                git: None,
                extra: BTreeMap::new(),
            },
        );
        let lock = Lock {
            season: "2026.09".to_owned(),
            judges,
            extra: BTreeMap::new(),
        };
        let fetcher = FromMap::serving(URL, Vec::new());

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a report");
        assert!(matches!(only(&reports).state, JudgeState::Missing(_)));
    }

    #[test]
    fn judges_are_reported_in_name_order() {
        let root = tempfile::tempdir().expect("a temp root");
        let mut lock = lock_pinning("weeder", OTHER_PLATFORM, URL, &"0".repeat(64));
        for name in ["tilth", "copeca"] {
            let judge = lock
                .judges
                .get("weeder")
                .cloned()
                .expect("the weeder judge");
            lock.judges.insert(name.to_owned(), judge);
        }
        let fetcher = FromMap::serving(URL, Vec::new());

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("three reports");
        assert_eq!(
            reports
                .iter()
                .map(|report| report.name.as_str())
                .collect::<Vec<_>>(),
            ["copeca", "tilth", "weeder"]
        );
    }

    #[test]
    fn each_state_has_its_own_line() {
        let report = |state| JudgeReport {
            name: "tilth".to_owned(),
            version: "0.10.1".to_owned(),
            state,
        };
        assert_eq!(line(&report(JudgeState::Verified)), "tilth 0.10.1 verified");
        assert_eq!(
            line(&report(JudgeState::Fetched { manifest: true })),
            "tilth 0.10.1 fetched"
        );
        assert_eq!(
            line(&report(JudgeState::Fetched { manifest: false })),
            "tilth 0.10.1 fetched, no manifest in artifact"
        );
        assert_eq!(
            line(&report(JudgeState::Missing("no artifact for x".to_owned()))),
            "tilth 0.10.1 missing: no artifact for x"
        );
        assert_eq!(
            line(&report(JudgeState::Mismatch {
                expected: "aa".to_owned(),
                actual: "bb".to_owned(),
            })),
            "tilth 0.10.1 mismatch: expected aa, got bb"
        );
    }

    #[test]
    fn only_verified_and_fetched_judges_count_as_resolved() {
        let report = |state| JudgeReport {
            name: "tilth".to_owned(),
            version: "0.10.1".to_owned(),
            state,
        };
        assert!(resolved(&[
            report(JudgeState::Verified),
            report(JudgeState::Fetched { manifest: false }),
        ]));
        assert!(!resolved(&[report(JudgeState::Missing("x".to_owned()))]));
        assert!(!resolved(&[report(JudgeState::Mismatch {
            expected: "a".to_owned(),
            actual: "b".to_owned(),
        })]));
    }

    #[test]
    fn the_face_prints_a_line_per_judge_and_answers_zero() {
        let root = tempfile::tempdir().expect("a temp root");
        let bytes = artifact();
        let digest = digest_of(&bytes);
        let url = format!(
            "https://example.invalid/weeder-{}.tar.gz",
            crate::lock::platform()
        );
        let lock = lock_pinning("weeder", crate::lock::platform(), &url, &digest);
        write_file(
            &layout::garden_lock(root.path()),
            &toml::to_string(&lock).expect("the lock serialises"),
        )
        .expect("a lock at the root");
        let fetcher = FromMap::serving(&url, bytes);

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = verify_face(root.path(), &fetcher, &mut stdout, &mut stderr);
        assert_eq!(code, 0, "{}", String::from_utf8_lossy(&stderr));
        assert_eq!(String::from_utf8_lossy(&stdout), "weeder 0.1.0 fetched\n");
        assert!(stderr.is_empty());
    }

    #[test]
    fn the_face_answers_three_when_a_judge_is_missing() {
        let root = tempfile::tempdir().expect("a temp root");
        let lock = lock_pinning("weeder", "sparc-unknown-linux-gnu", URL, &"0".repeat(64));
        write_file(
            &layout::garden_lock(root.path()),
            &toml::to_string(&lock).expect("the lock serialises"),
        )
        .expect("a lock at the root");
        let fetcher = FromMap::serving(URL, Vec::new());

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = verify_face(root.path(), &fetcher, &mut stdout, &mut stderr);
        assert_eq!(code, 3);
        let printed = String::from_utf8_lossy(&stdout);
        assert!(printed.starts_with("weeder 0.1.0 missing: "), "{printed}");
        assert!(printed.contains(crate::lock::platform()), "{printed}");
    }

    #[test]
    fn the_face_answers_one_when_there_is_no_lock_to_verify() {
        let root = tempfile::tempdir().expect("a temp root");
        let fetcher = FromMap::serving(URL, Vec::new());

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = verify_face(root.path(), &fetcher, &mut stdout, &mut stderr);
        assert_eq!(code, 1);
        assert!(stdout.is_empty());
        let said = String::from_utf8_lossy(&stderr);
        assert!(said.contains(layout::GARDEN_LOCK), "{said}");
    }

    // ------------------------------------------------------------------ the git judge

    fn git(at: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(at)
            .args(args)
            .output()
            .expect("git runs on the build machine");
        assert!(
            output.status.success(),
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// A one-commit repository carrying a manifest, and the commit it is standing on.
    fn origin(at: &Path) -> String {
        std::fs::create_dir_all(at).expect("the origin directory");
        git(at, &["init", "--quiet"]);
        git(at, &["config", "user.email", "fixture@example.invalid"]);
        git(at, &["config", "user.name", "fixture"]);
        std::fs::write(at.join(layout::GARDEN_JSON), weeder_manifest()).expect("the manifest");
        git(at, &["add", "."]);
        git(at, &["commit", "--quiet", "-m", "the fixture bed"]);
        git(at, &["rev-parse", "HEAD"]).trim().to_owned()
    }

    fn git_lock(url: &str, rev: &str) -> Lock {
        let mut judges = BTreeMap::new();
        judges.insert(
            "weeder".to_owned(),
            Judge {
                version: "0.1.0".to_owned(),
                npm: None,
                platforms: BTreeMap::new(),
                git: Some(GitInstall {
                    url: url.to_owned(),
                    rev: rev.to_owned(),
                }),
                extra: BTreeMap::new(),
            },
        );
        Lock {
            season: "2026.09".to_owned(),
            judges,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn a_judge_pinned_by_a_commit_is_cloned_checked_out_and_its_manifest_cached() {
        let scratch = tempfile::tempdir().expect("a temp scratch");
        let source = scratch.path().join("origin");
        let rev = origin(&source);
        let root = tempfile::tempdir().expect("a temp root");
        let lock = git_lock(&source.display().to_string(), &rev);
        let fetcher = FromMap::serving(URL, Vec::new());

        let reports = verify(root.path(), &lock, PLATFORM, &fetcher).expect("a clone");
        assert_eq!(only(&reports).state, JudgeState::Fetched { manifest: true });
        assert_eq!(
            std::fs::read_to_string(layout::bed_manifest(root.path(), "weeder"))
                .expect("the cached manifest"),
            weeder_manifest()
        );
        assert_eq!(
            git(
                &layout::bed_artifact(root.path(), "weeder"),
                &["rev-parse", "HEAD"]
            )
            .trim(),
            rev
        );

        let again = verify(root.path(), &lock, PLATFORM, &fetcher).expect("the second verify");
        assert_eq!(only(&again).state, JudgeState::Verified);
    }

    #[test]
    fn a_clone_standing_on_another_commit_is_a_mismatch() {
        let scratch = tempfile::tempdir().expect("a temp scratch");
        let source = scratch.path().join("origin");
        let first = origin(&source);
        let root = tempfile::tempdir().expect("a temp root");
        verify(
            root.path(),
            &git_lock(&source.display().to_string(), &first),
            PLATFORM,
            &FromMap::serving(URL, Vec::new()),
        )
        .expect("the first clone");

        let wanted = "0".repeat(40);
        let reports = verify(
            root.path(),
            &git_lock(&source.display().to_string(), &wanted),
            PLATFORM,
            &FromMap::serving(URL, Vec::new()),
        )
        .expect("a report");
        assert_eq!(
            only(&reports).state,
            JudgeState::Mismatch {
                expected: wanted,
                actual: first,
            }
        );
    }
}
