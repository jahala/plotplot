//! `garden.lock`, the pinned judges.
//!
//! A stamp's meaning depends on the judge's version, so the lock is what makes proof
//! reproducible. It is TOML at rest; the contracts' schema describes the JSON a conforming
//! TOML parser produces from it, so the stem parses the TOML to a JSON value, enforces the
//! schema on that value, and only then reads the model.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::layout;
use crate::manifest::{GitInstall, compile, one_line};

/// The contracts' lock schema, v1.2.0, embedded so the binary carries its own contract.
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
            ["tend2", "tilth", "weeder"]
        );

        let weeder = lock.judges.get("weeder").expect("the weeder judge");
        assert_eq!(weeder.version, "0.1.0");
        assert_eq!(weeder.npm, None);
        assert_eq!(
            weeder
                .platforms
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["aarch64-apple-darwin", "x86_64-unknown-linux-musl"]
        );
        let darwin = weeder
            .platforms
            .get("aarch64-apple-darwin")
            .expect("the darwin artifact");
        assert_eq!(
            darwin.url,
            "https://github.com/jahala/weeder/releases/download/v0.1.0/weeder-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            darwin.sha256,
            "6ae4f3e626840edcd08b1dea6118c88d01ffabb845a36d19f19c885d47d5095b"
        );
        let linux = weeder
            .platforms
            .get("x86_64-unknown-linux-musl")
            .expect("the linux artifact");
        assert_eq!(
            linux.url,
            "https://github.com/jahala/weeder/releases/download/v0.1.0/weeder-x86_64-unknown-linux-musl.tar.gz"
        );
        assert_eq!(
            linux.sha256,
            "1ad2bdae7f784ffc5dc5710896a67321fc4b5a193c49ea3ae4e3131b3cf32d75"
        );

        let tilth = lock.judges.get("tilth").expect("the tilth judge");
        assert_eq!(tilth.version, "1.0.0");
        assert_eq!(tilth.npm, None);
        assert_eq!(tilth.platforms.len(), 1);
        assert_eq!(
            tilth
                .platforms
                .get("aarch64-apple-darwin")
                .map(|artifact| artifact.sha256.as_str()),
            Some("da865417c06f59ac11bd105f3578014785dd998107b61060b8122d140d45819d")
        );

        let tend2 = lock.judges.get("tend2").expect("the tend2 judge");
        assert_eq!(tend2.version, "1.0.0");
        assert_eq!(tend2.npm.as_deref(), Some("@plotplot/tend2"));
        assert!(tend2.platforms.is_empty());
    }

    #[test]
    fn a_digest_that_is_not_sixty_four_lowercase_hex_is_refused() {
        for bad in [
            "6AE4F3E626840EDCD08B1DEA6118C88D01FFABB845A36D19F19C885D47D5095B",
            "6ae4f3e626840edcd08b1dea6118c88d01ffabb845a36d19f19c885d47d5095",
            "sha256-6ae4f3e626840edcd08b1dea6118c88d01ffabb845a36d19f19c885d47d5095b",
            "",
        ] {
            let text = fixture().replace(
                "6ae4f3e626840edcd08b1dea6118c88d01ffabb845a36d19f19c885d47d5095b",
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
        assert_eq!(lock.judges.len(), 3);
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
