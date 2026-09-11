//! A marked region inside a file somebody else also writes.
//!
//! The garden block in `AGENTS.md` and the stem's lines in `.github/CODEOWNERS` are both
//! regions: two markers the stem owns, and everything outside them belongs to whoever wrote
//! it. [`replace_in`] copies those bytes through untouched, and each caller turns a refusal
//! into its own file's error.

/// The two markers bounding a region, and the file they sit in, which a refusal names first.
#[derive(Clone, Copy, Debug)]
pub struct Markers<'a> {
    pub file: &'a str,
    pub begin: &'a str,
    pub end: &'a str,
}

/// `text` with `region` planted in it.
///
/// When the markers are already there the marked region is replaced and every other byte is
/// copied through. When they are not, the region is appended after one blank line. Applying
/// the same region twice yields the same bytes as applying it once.
///
/// # Errors
///
/// One line, starting with the file's name, when the markers do not make exactly one region:
/// a begin without an end, an end without a begin, an end before its begin, or more than one
/// of either. The stem will not guess which bytes the region owns.
pub fn replace_in(
    text: &str,
    region: &str,
    markers: Markers,
) -> std::result::Result<String, String> {
    let Markers { file, begin, end } = markers;
    let begins = text.matches(begin).count();
    let ends = text.matches(end).count();

    match (begins, ends) {
        (0, 0) => Ok(appended(text, region)),
        (1, 1) => {
            let (start, stop) = match (text.find(begin), text.find(end)) {
                (Some(start), Some(stop)) => (start, stop),
                _ => return Err(format!("{file} lost a plotplot marker while being read")),
            };
            if start > stop {
                return Err(format!("{file} has {end} before {begin}"));
            }
            // Sliced with `get`, so a marker in a place these bounds did not expect is a
            // refusal rather than a panic in a git hook.
            let (before, after) = match (text.get(..start), text.get(stop + end.len()..)) {
                (Some(before), Some(after)) => (before, after),
                _ => return Err(format!("{file} has markers the stem cannot cut on")),
            };
            let mut planted = String::with_capacity(before.len() + region.len() + after.len());
            planted.push_str(before);
            planted.push_str(region);
            planted.push_str(after);
            Ok(planted)
        }
        (_, 0) => Err(format!("{file} has {begin} without {end}")),
        (0, _) => Err(format!("{file} has {end} without {begin}")),
        _ => Err(format!(
            "{file} has {begins} of {begin} and {ends} of {end}; the stem replaces exactly one region"
        )),
    }
}

/// `text` with `region` after it, separated by one blank line and never by two.
///
/// Only newlines are added: no byte already in the file is moved or dropped.
fn appended(text: &str, region: &str) -> String {
    if text.is_empty() {
        return format!("{region}\n");
    }
    let trailing = text.len() - text.trim_end_matches('\n').len();
    let mut planted = String::with_capacity(text.len() + region.len() + 3);
    planted.push_str(text);
    for _ in trailing..2 {
        planted.push('\n');
    }
    planted.push_str(region);
    planted.push('\n');
    planted
}
