//! The stem's region of `.gitignore`: everything under `.plotplot/` stays out of git's view.
//!
//! What lives there is machine-written: fetched judges and their records, the manifests read
//! from their artifacts, the bundles, the friction journal, receipt drafts, caches. None of
//! it is committed (`plotplot check --base` refuses a tree that carries any of it), so one
//! line covers every tree the stem or a bed writes there, including trees added later. Every
//! line outside the region belongs to whoever wrote it.

use crate::error::{Error, Result};
use crate::layout;
use crate::plant::region::{self, Markers};

/// Where the region starts.
pub const BEGIN: &str = "# plotplot:begin";
/// Where it ends.
pub const END: &str = "# plotplot:end";

/// The region, markers included.
pub fn render() -> String {
    format!(
        "{BEGIN}\n\
         # Written by plotplot init: the stem's machine-written state, never committed.\n\
         /{dir}/\n\
         {END}",
        dir = layout::PLOTPLOT_DIR
    )
}

/// `gitignore` with the region planted in it, every other byte copied through.
///
/// # Errors
///
/// [`Error::Gitignore`] when the markers do not make exactly one region.
pub fn replace_in(gitignore: &str) -> Result<String> {
    let markers = Markers {
        file: layout::GITIGNORE,
        begin: BEGIN,
        end: END,
    };
    region::replace_in(gitignore, &render(), markers)
        .map_err(|problem| Error::Gitignore { problem })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_region_ignores_the_whole_machine_written_tree() {
        let region = render();
        assert!(region.starts_with(BEGIN), "{region}");
        assert!(region.ends_with(END), "{region}");
        assert!(region.lines().any(|line| line == "/.plotplot/"), "{region}");
    }

    #[test]
    fn planting_twice_is_planting_once_and_the_planters_lines_stay() {
        let once = replace_in("node_modules/\n").expect("no markers yet");
        assert!(once.starts_with("node_modules/\n"), "{once}");
        assert_eq!(replace_in(&once).expect("one region"), once);
    }

    #[test]
    fn markers_that_make_no_single_region_name_the_file() {
        let error = replace_in("# plotplot:begin\n/.plotplot/\n").expect_err("no end");
        assert!(error.to_string().starts_with(".gitignore"), "{error}");
    }
}
