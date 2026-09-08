//! The garden block: the ten lines `AGENTS.md` carries about what is planted here.
//!
//! Every agent on every vendor reads `AGENTS.md`, so this block is the stem's whole context
//! cost at session start. It says what is planted, where to orient, which command gates, and
//! that the hard limits are not advice. It is bounded by two markers, and everything outside
//! them belongs to whoever wrote it: [`replace_in`] copies those bytes through untouched.

use crate::bed::Bed;
use crate::error::{Error, Result};
use crate::layout;

/// Where the block starts. Everything before it in `AGENTS.md` is somebody else's.
pub const BEGIN: &str = "<!-- plotplot:begin -->";
/// Where the block ends.
pub const END: &str = "<!-- plotplot:end -->";

/// The garden block for a repository planted with `beds` in `season`, markers included.
///
/// The beds appear in the order they are given, all on one line, so the block stays inside
/// its ten lines however many beds are planted.
pub fn render(season: &str, beds: &[Bed]) -> String {
    let planted = if beds.is_empty() {
        "nothing yet".to_owned()
    } else {
        beds.iter()
            .map(|bed| format!("{} {}", bed.name, bed.version))
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "\
{BEGIN}
## plotplot: what is planted here

Planted: {planted}.
Season: {season}.

- Orient first: `tend2 next docs/tend2` says what is next, what is running, and what went stale.
- Gates: `plotplot check` runs every planted gate and returns one SARIF log.
- Help: `plotplot doctor` says whether the law is live, and what is missing when it is not.
- The hard limits are enforced at the boundary: `--no-verify` is refused, and `{lock}` and `{hooks}/` are the stem's to write.
{END}",
        lock = layout::GARDEN_LOCK,
        hooks = layout::GITHOOKS_DIR,
    )
}

/// `agents_md` with `block` planted in it.
///
/// When the markers are already there the marked region is replaced and every other byte is
/// copied through. When they are not, the block is appended after one blank line. Applying
/// the same block twice yields the same bytes as applying it once.
///
/// # Errors
///
/// [`Error::Agents`] when the markers do not make exactly one region: a begin without an
/// end, an end without a begin, an end before its begin, or more than one of either. The
/// stem will not guess which bytes the block owns.
pub fn replace_in(agents_md: &str, block: &str) -> Result<String> {
    let begins = agents_md.matches(BEGIN).count();
    let ends = agents_md.matches(END).count();

    let refuse = |problem: String| Err(Error::Agents { problem });
    let file = layout::AGENTS_MD;

    match (begins, ends) {
        (0, 0) => Ok(appended(agents_md, block)),
        (1, 1) => {
            let (begin, end) = match (agents_md.find(BEGIN), agents_md.find(END)) {
                (Some(begin), Some(end)) => (begin, end),
                _ => return refuse(format!("{file} lost a plotplot marker while being read")),
            };
            if begin > end {
                return refuse(format!("{file} has {END} before {BEGIN}"));
            }
            // Sliced with `get`, so a marker in a place these bounds did not expect is a
            // refusal rather than a panic in a git hook.
            let (before, after) = match (agents_md.get(..begin), agents_md.get(end + END.len()..)) {
                (Some(before), Some(after)) => (before, after),
                _ => return refuse(format!("{file} has markers the stem cannot cut on")),
            };
            let mut planted = String::with_capacity(before.len() + block.len() + after.len());
            planted.push_str(before);
            planted.push_str(block);
            planted.push_str(after);
            Ok(planted)
        }
        (_, 0) => refuse(format!("{file} has {BEGIN} without {END}")),
        (0, _) => refuse(format!("{file} has {END} without {BEGIN}")),
        _ => refuse(format!(
            "{file} has {begins} of {BEGIN} and {ends} of {END}; the stem replaces exactly one block"
        )),
    }
}

/// `agents_md` with `block` after it, separated by one blank line and never by two.
///
/// Only newlines are added: no byte already in the file is moved or dropped.
fn appended(agents_md: &str, block: &str) -> String {
    if agents_md.is_empty() {
        return format!("{block}\n");
    }
    let trailing = agents_md.len() - agents_md.trim_end_matches('\n').len();
    let mut planted = String::with_capacity(agents_md.len() + block.len() + 3);
    planted.push_str(agents_md);
    for _ in trailing..2 {
        planted.push('\n');
    }
    planted.push_str(block);
    planted.push('\n');
    planted
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn bed(name: &str, version: &str) -> Bed {
        Bed {
            name: name.to_owned(),
            version: version.to_owned(),
            binary: Some(name.to_owned()),
            skill: None,
            hooks: BTreeMap::new(),
            git_hooks: Vec::new(),
            mcp: None,
            check: None,
        }
    }

    fn beds() -> Vec<Bed> {
        vec![bed("tend2", "1.0.0"), bed("weeder", "0.1.0")]
    }

    /// The lines between the markers, which is what the ten-line budget counts.
    fn body(block: &str) -> Vec<&str> {
        block
            .lines()
            .skip_while(|line| *line != BEGIN)
            .skip(1)
            .take_while(|line| *line != END)
            .collect()
    }

    #[test]
    fn the_block_opens_and_closes_on_its_markers() {
        let block = render("2026.09", &beds());
        assert!(block.starts_with(BEGIN), "{block}");
        assert!(block.ends_with(END), "{block}");
        assert_eq!(block.matches(BEGIN).count(), 1);
        assert_eq!(block.matches(END).count(), 1);
    }

    #[test]
    fn the_body_stays_inside_ten_lines_however_many_beds_are_planted() {
        let many: Vec<Bed> = (0..40)
            .map(|number| bed(&format!("bed{number}"), "9.9.9"))
            .collect();
        for beds in [Vec::new(), beds(), many] {
            let block = render("2026.09", &beds);
            assert!(body(&block).len() <= 10, "{block}");
        }
    }

    #[test]
    fn the_body_names_every_bed_with_its_version_and_the_season() {
        let block = render("2026.09", &beds());
        assert!(block.contains("tend2 1.0.0"), "{block}");
        assert!(block.contains("weeder 0.1.0"), "{block}");
        assert!(block.contains("Season: 2026.09."), "{block}");
    }

    #[test]
    fn an_unplanted_repository_is_told_so_rather_than_shown_an_empty_list() {
        let block = render("2026.09", &[]);
        assert!(block.contains("Planted: nothing yet."), "{block}");
    }

    #[test]
    fn the_body_carries_the_three_commands_and_the_hard_limits() {
        let block = render("2026.09", &beds());
        for phrase in [
            "tend2 next docs/tend2",
            "plotplot check",
            "plotplot doctor",
            "--no-verify",
            "garden.lock",
            ".githooks/",
        ] {
            assert!(block.contains(phrase), "{phrase} is missing:\n{block}");
        }
    }

    #[test]
    fn the_body_keeps_the_brands_voice() {
        let block = render("2026.09", &beds());
        let body = body(&block).join("\n");
        assert!(!body.contains('!'), "{block}");
        assert!(!body.contains('\u{2014}'), "{block}");
        for product in ["plotplot", "tend2", "weeder"] {
            let shouted = format!("{}{}", product[..1].to_uppercase(), &product[1..]);
            assert!(
                !body.contains(&shouted),
                "{shouted} is capitalised:\n{block}"
            );
        }
    }

    /// [`replace_in`]'s bytes, for the cases that must succeed.
    fn planted(agents_md: &str, block: &str) -> String {
        replace_in(agents_md, block).expect("markers that make one region")
    }

    #[test]
    fn a_file_without_markers_gets_the_block_after_one_blank_line() {
        let block = render("2026.09", &beds());
        assert_eq!(
            planted("# a repository\n", &block),
            format!("# a repository\n\n{block}\n")
        );
        assert_eq!(
            planted("# no trailing newline", &block),
            format!("# no trailing newline\n\n{block}\n")
        );
        assert_eq!(
            planted("# already blank\n\n", &block),
            format!("# already blank\n\n{block}\n")
        );
        assert_eq!(planted("", &block), format!("{block}\n"));
    }

    #[test]
    fn replacing_touches_no_byte_outside_the_markers() {
        let old = render("2026.09", &beds());
        let new = render("2026.10", &[bed("tilth", "0.10.1")]);
        let before = "# top\n\nkeep me\n\n";
        let after = "\n\n## bottom\n\nkeep me too\n";

        assert_eq!(
            planted(&format!("{before}{old}{after}"), &new),
            format!("{before}{new}{after}")
        );
    }

    #[test]
    fn planting_is_idempotent_whether_the_markers_were_there_or_not() {
        let block = render("2026.09", &beds());
        for start in [
            "# a repository\n",
            "",
            "# marked\n\n<!-- plotplot:begin -->\nold\n<!-- plotplot:end -->\n",
        ] {
            let once = replace_in(start, &block).expect("the first planting");
            let twice = replace_in(&once, &block).expect("the second planting");
            assert_eq!(once, twice, "from {start:?}");
        }
    }

    #[test]
    fn markers_that_do_not_make_one_region_are_refused() {
        let block = render("2026.09", &beds());
        let refusals = [
            format!("{BEGIN}\nold\n"),
            format!("old\n{END}\n"),
            format!("{END}\nold\n{BEGIN}\n"),
            format!("{BEGIN}\na\n{END}\n{BEGIN}\nb\n{END}\n"),
        ];
        for agents in refusals {
            match replace_in(&agents, &block) {
                Err(Error::Agents { problem }) => {
                    assert!(problem.starts_with(layout::AGENTS_MD), "{problem}");
                    assert!(!problem.contains('\n'), "{problem}");
                }
                other => panic!("{agents:?} was not refused: {other:?}"),
            }
        }
    }

    #[test]
    fn a_marker_inside_the_prose_is_still_a_marker() {
        let block = render("2026.09", &beds());
        let agents = format!("before {BEGIN} old {END} after\n");
        assert_eq!(planted(&agents, &block), format!("before {block} after\n"));
    }
}
