//! The stem's region of `weeder.toml`: the pull request gate at weeder's constitution tier.
//!
//! weeder stops a commit that changes a path its config names under `[guardrails] paths`,
//! unless a `Weeder-allow` trailer gives the reason, so an edit to the gate is refused at the
//! commit where it is made, before any pull request exists. The rest of the file is the
//! planter's. TOML allows a table once, so when the planter's own `[guardrails]` table
//! already names the gate there is nothing to plant, and when it names other paths the stem
//! asks for the one line rather than editing a table it does not own.

use crate::error::{Error, Result};
use crate::layout;
use crate::plant::region::{self, Markers};

/// Where the region starts.
pub const BEGIN: &str = "# plotplot:begin";
/// Where it ends.
pub const END: &str = "# plotplot:end";

/// The region holding `gate` under `[guardrails] paths`, markers included.
pub fn render(gate: &str) -> String {
    format!(
        "{BEGIN}\n\
         # Written by plotplot init: the pull request gate is held where the constitution is.\n\
         [guardrails]\n\
         paths = [\"{gate}\"]\n\
         {END}"
    )
}

/// `weeder_toml` as it should be with `gate` held, or `None` when the planter's own
/// `[guardrails]` table already names it and the file is left as it is.
///
/// # Errors
///
/// [`Error::WeederConfig`] when the file outside the region is not TOML, when the planter's
/// own `[guardrails]` table does not name `gate` (saying which line to add), or when the
/// markers do not make one region.
pub fn planted(weeder_toml: &str, gate: &str) -> Result<Option<String>> {
    let refuse = |problem: String| Error::WeederConfig { problem };
    let markers = Markers {
        file: layout::WEEDER_TOML,
        begin: BEGIN,
        end: END,
    };

    // The planter's bytes: the file with the stem's region taken out.
    let without = region::replace_in(weeder_toml, "", markers).map_err(refuse)?;
    let theirs: toml::Table = without.parse().map_err(|error: toml::de::Error| {
        refuse(format!(
            "{}: {}",
            layout::WEEDER_TOML,
            crate::manifest::one_line(&error.to_string())
        ))
    })?;

    match theirs.get("guardrails") {
        None => {
            let text = region::replace_in(weeder_toml, &render(gate), markers).map_err(refuse)?;
            Ok(Some(format!("{}\n", text.trim_end_matches('\n'))))
        }
        Some(guardrails) => {
            let names_gate = guardrails
                .get("paths")
                .and_then(toml::Value::as_array)
                .is_some_and(|paths| paths.iter().any(|path| path.as_str() == Some(gate)));
            if names_gate && !weeder_toml.contains(BEGIN) {
                Ok(None)
            } else if names_gate {
                // The planter took the gate into their own table; the stem's region is
                // now a second [guardrails] table and comes out.
                Ok(Some(format!("{}\n", without.trim_end_matches('\n'))))
            } else {
                Err(refuse(format!(
                    "{}: its own [guardrails] table does not hold the pull request gate; add \
                     \"{gate}\" to its paths, then run plotplot init again",
                    layout::WEEDER_TOML
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GATE: &str = ".github/workflows/plotplot-check.yml";

    #[test]
    fn an_empty_file_gets_the_region_and_planting_twice_is_planting_once() {
        let once = planted("", GATE).expect("plantable").expect("a region");
        assert!(once.starts_with(BEGIN), "{once}");
        assert!(once.ends_with(&format!("{END}\n")), "{once}");
        assert_eq!(planted(&once, GATE).expect("plantable"), Some(once));
    }

    #[test]
    fn a_file_that_is_not_toml_names_the_file() {
        let error = planted("[guard\n", GATE).expect_err("not TOML");
        assert!(error.to_string().starts_with("weeder.toml: "), "{error}");
    }
}
