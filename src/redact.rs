//! The redaction face: text in, the same text with every secret span replaced by
//! `[REDACTED]` out, and a tally of which layer caught each span.
//!
//! `contracts/redaction.md` is the contract and `contracts/fixtures/redaction/cases.jsonl`
//! its table. [`redact`] is pure: no I/O, no logging of content. The face reads standard
//! input as bytes and refuses anything that is not UTF-8 with exit 1 and nothing on standard
//! output, so a caller that cannot have its text scrubbed writes nothing.
//!
//! Six layers each return the spans they flag. The spans are merged, so a span that several
//! layers flag, or that overlaps another, is replaced once, and it is counted under the most
//! specific layer that flagged it: format, prefix, uri, connection, keyvalue, then entropy,
//! which is the net for shapes nobody has a rule for.

use std::io::Write;
use std::sync::OnceLock;

use regex::{Captures, Regex};
use serde::Serialize;

use crate::cli::RedactArgs;

/// The literal every replaced span becomes.
pub const REDACTED: &str = "[REDACTED]";

/// The text after redaction and how many spans each layer replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redaction {
    pub text: String,
    pub tally: Tally,
}

/// The object `--tally` writes to standard error.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Tally {
    pub total: usize,
    pub by: ByLayer,
}

/// Spans replaced, per layer. A span two layers flag is counted once, under the more
/// specific of the two.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ByLayer {
    pub entropy: usize,
    pub format: usize,
    pub prefix: usize,
    pub uri: usize,
    pub connection: usize,
    pub keyvalue: usize,
}

/// The layers, most specific first. The order decides which layer a merged span counts under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Format,
    Prefix,
    Uri,
    Connection,
    Keyvalue,
    Entropy,
}

/// A byte range of the input that one layer flagged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Span {
    start: usize,
    end: usize,
    layer: Layer,
}

/// Known provider formats. Where a pattern has a capture group, only the group is the secret
/// and the rest is context that stays.
const FORMATS: &[(&str, &str)] = &[
    (
        "jwt",
        r"\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]*",
    ),
    (
        "http-authorization",
        r"(?i)\bauthorization[ \t]*[:=][ \t]*(?:basic|bearer|token)[ \t]+([A-Za-z0-9._~+/-]{8,}=*)",
    ),
    // A block cut off before its END line is still redacted to the end of the text. The
    // dashes are written as a count so this source carries no key header a scanner reads as
    // a key.
    (
        "pem-private-key",
        r"-{5}BEGIN [A-Z0-9 ]*PRIVATE[ ]KEY(?: BLOCK)?-{5}(?s:.*?)(?:-{5}END [A-Z0-9 ]*PRIVATE[ ]KEY(?: BLOCK)?-{5}|\z)",
    ),
];

/// Provider key prefixes. Capture group 1 is the tail, whose minimum length is the shortest
/// a real key of that provider carries, so a prefix named in prose is never a key.
const PREFIXES: &[(&str, &str)] = &[
    ("anthropic-openai", r"\bsk-([A-Za-z0-9_-]{20,})"),
    ("stripe", r"\b[rs]k_(?:live|test)_([A-Za-z0-9]{20,})"),
    ("github", r"\b(?:gh[pousr]_|github_pat_)([A-Za-z0-9_]{30,})"),
    ("slack", r"\bxox[abeoprs]-([A-Za-z0-9-]{10,})"),
    ("npm", r"\bnpm_([A-Za-z0-9]{30,})"),
    ("google", r"\bAIza([A-Za-z0-9_-]{30,})"),
    (
        "sendgrid",
        r"\bSG\.([A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,})",
    ),
    ("supabase", r"\bsb_secret_([A-Za-z0-9_-]{20,})"),
    ("aws-access-key-id", r"\b(?:AKIA|ASIA)([A-Z0-9]{16})\b"),
];

/// The last word of a key that names a credential. `pass` and `pwd` are the short spellings
/// environment files use.
const CREDENTIAL_WORDS: &[&str] = &[
    "password", "passwd", "pass", "pwd", "secret", "token", "auth", "apikey",
];

/// Bits of Shannon entropy per character above which a long token reads as random.
///
/// A random base62 or base64 token of 32 characters or more measures about 4.6 to 5.2, while
/// a hex digest cannot exceed 4.0 (its alphabet has 16 symbols) and identifiers and paths
/// made of words measure under 4.0. 4.2 sits in the gap between the two.
const ENTROPY_THRESHOLD: f64 = 4.2;

/// The shortest token the entropy layer judges. Below this, entropy measured over so few
/// characters cannot tell a word from a key.
const ENTROPY_MIN_LEN: usize = 32;

/// Every pattern, compiled once.
struct Patterns {
    format: Vec<Regex>,
    prefix: Vec<Regex>,
    uri: Regex,
    connection_keyword: Regex,
    connection_jdbc: Regex,
    keyvalue_json: Regex,
    keyvalue_assign: Regex,
    keyvalue_colon: Regex,
    token: Regex,
    uuid: Regex,
}

/// Redact one write's worth of text.
///
/// # Errors
///
/// A message when the stem's own patterns do not compile, which is a defect of the stem;
/// the face then fails closed. The message never quotes the text.
pub fn redact(text: &str) -> Result<Redaction, String> {
    let patterns = patterns().as_ref().map_err(Clone::clone)?;
    let mut spans = Vec::new();
    spans.extend(format(patterns, text));
    spans.extend(prefix(patterns, text));
    spans.extend(uri(patterns, text));
    spans.extend(connection(patterns, text));
    spans.extend(keyvalue(patterns, text));
    spans.extend(entropy(patterns, text));
    Ok(replace(text, merge(spans)))
}

fn patterns() -> &'static Result<Patterns, String> {
    static PATTERNS: OnceLock<Result<Patterns, String>> = OnceLock::new();
    PATTERNS.get_or_init(compile)
}

fn compile() -> Result<Patterns, String> {
    let one = |name: &str, pattern: &str| {
        Regex::new(pattern).map_err(|error| format!("the {name} pattern does not compile: {error}"))
    };
    let table = |rows: &[(&str, &str)]| {
        rows.iter()
            .map(|(name, pattern)| one(name, pattern))
            .collect::<Result<Vec<_>, _>>()
    };
    Ok(Patterns {
        format: table(FORMATS)?,
        prefix: table(PREFIXES)?,
        uri: one(
            "uri",
            r"\b[A-Za-z][A-Za-z0-9+.-]*://[^\s:/?#@]+:([^\s/?#@]+)@",
        )?,
        // Group 1 and group 3 are the separators around the field; one of them must be a
        // semicolon, or this is a lone assignment and the keyvalue layer's to judge.
        connection_keyword: one(
            "connection-keyword",
            r#"(?im)(^|;)[ \t]*(?:password|pwd)[ \t]*=[ \t]*('(?:[^'\r\n]|'')*'|"(?:[^"\r\n]|"")*"|\{(?:[^}\r\n]|\}\})*\}|[^;\r\n]*?)[ \t]*(;|\r?$)"#,
        )?,
        connection_jdbc: one(
            "connection-jdbc",
            r"(?i)\bjdbc:[a-z0-9]+:\S*?[?&;]password=([^&;\s]+)",
        )?,
        keyvalue_json: one(
            "keyvalue-json",
            r#""([A-Za-z_][A-Za-z0-9_.-]*)"[ \t]*:[ \t]*"((?:[^"\\\r\n]|\\.)*)""#,
        )?,
        // The value may not start with `=`, so a comparison such as `token == other` is not
        // an assignment.
        keyvalue_assign: one(
            "keyvalue-assign",
            r#"(?m)(?:^|[^A-Za-z0-9_.-])([A-Za-z_][A-Za-z0-9_.-]*)[ \t]*=[ \t]*("(?:[^"\\\r\n]|\\.)*"|'[^'\r\n]*'|[^=\s"'`;&,][^\s"'`;&,]*)"#,
        )?,
        // A colon form counts only as a whole line, `key: value`, so a sentence that puts a
        // colon after a credential word is prose.
        keyvalue_colon: one(
            "keyvalue-colon",
            r#"(?m)^[ \t]*(?:-[ \t]+)?([A-Za-z_][A-Za-z0-9_.-]*)[ \t]*:[ \t]+("(?:[^"\\\r\n]|\\.)*"|'[^'\r\n]*'|[^\s"'#]+)[ \t]*(?:#.*?)?\r?$"#,
        )?,
        token: one("token", r"[A-Za-z0-9+/_-]{32,}={0,2}")?,
        uuid: one(
            "uuid",
            r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}$",
        )?,
    })
}

/// The span of capture group 1 when the pattern has one, else of the whole match.
fn secret_of(captures: &Captures, layer: Layer) -> Option<Span> {
    let found = captures.get(1).or_else(|| captures.get(0))?;
    span(found.start(), found.end(), layer)
}

fn span(start: usize, end: usize, layer: Layer) -> Option<Span> {
    (start < end).then_some(Span { start, end, layer })
}

/// Layer: known provider formats.
fn format(patterns: &Patterns, text: &str) -> Vec<Span> {
    patterns
        .format
        .iter()
        .flat_map(|pattern| pattern.captures_iter(text))
        .filter_map(|captures| secret_of(&captures, Layer::Format))
        .collect()
}

/// Layer: provider key prefixes with a tail that looks generated. A tail of lowercase words
/// and separators, such as `sk-learn-the-basics-of-the-library`, is a name.
fn prefix(patterns: &Patterns, text: &str) -> Vec<Span> {
    patterns
        .prefix
        .iter()
        .flat_map(|pattern| pattern.captures_iter(text))
        .filter(|captures| {
            captures.get(1).is_some_and(|tail| {
                tail.as_str()
                    .chars()
                    .any(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
            })
        })
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            span(whole.start(), whole.end(), Layer::Prefix)
        })
        .collect()
}

/// Layer: the password in a URI's userinfo, and only the password.
fn uri(patterns: &Patterns, text: &str) -> Vec<Span> {
    patterns
        .uri
        .captures_iter(text)
        .filter_map(|captures| captures.get(1))
        .filter(|password| !is_placeholder(password.as_str()))
        .filter_map(|password| span(password.start(), password.end(), Layer::Uri))
        .collect()
}

/// Layer: the password field of a keyword connection string and of a JDBC URL.
fn connection(patterns: &Patterns, text: &str) -> Vec<Span> {
    let keyword = patterns
        .connection_keyword
        .captures_iter(text)
        .filter(|captures| {
            captures.get(1).is_some_and(|m| m.as_str() == ";")
                || captures.get(3).is_some_and(|m| m.as_str() == ";")
        })
        .filter_map(|captures| captures.get(2));
    let jdbc = patterns
        .connection_jdbc
        .captures_iter(text)
        .filter_map(|captures| captures.get(1));
    keyword
        .chain(jdbc)
        .filter(|value| !is_placeholder(value.as_str()))
        .filter_map(|value| span(value.start(), value.end(), Layer::Connection))
        .collect()
}

/// Layer: a value whose key names a credential.
///
/// In a shell assignment the quotes belong to the word, so the whole quoted word goes. In
/// JSON and YAML the quotes are the syntax around the value, so they stay and the document
/// still parses.
fn keyvalue(patterns: &Patterns, text: &str) -> Vec<Span> {
    let json = patterns
        .keyvalue_json
        .captures_iter(text)
        .filter_map(|captures| {
            let key = captures.get(1)?;
            let value = captures.get(2)?;
            Some((key.as_str(), value.start(), value.end()))
        });
    let colon = patterns
        .keyvalue_colon
        .captures_iter(text)
        .filter_map(inside_quotes);
    let assign = patterns
        .keyvalue_assign
        .captures_iter(text)
        .filter_map(|captures| {
            let key = captures.get(1)?;
            let value = captures.get(2)?;
            let inner = value.as_str().trim_matches(['"', '\'']);
            // The judged text is the value inside its quotes; the replaced span keeps them.
            (!is_placeholder(inner)).then_some((key.as_str(), value.start(), value.end()))
        });
    json.chain(colon)
        .filter(|(_, start, end)| !is_placeholder(&text[*start..*end]))
        .chain(assign)
        .filter(|(key, _, _)| names_a_credential(key))
        .filter_map(|(_, start, end)| span(start, end, Layer::Keyvalue))
        .collect()
}

/// A key and the span of its value, without the quotes when the value is quoted.
fn inside_quotes<'t>(captures: Captures<'t>) -> Option<(&'t str, usize, usize)> {
    let key = captures.get(1)?;
    let value = captures.get(2)?;
    let (start, end) = if value.as_str().starts_with(['"', '\'']) {
        (value.start() + 1, value.end() - 1)
    } else {
        (value.start(), value.end())
    };
    Some((key.as_str(), start, end))
}

/// Layer: long tokens that read as random.
///
/// Never a hex digest of 40 or 64 characters (a git sha, a sha256), nor a UUID, nor a
/// Subresource Integrity digest such as `sha512-<base64>`: a digest is never a secret, and a
/// lock whose digests are scrubbed verifies nothing. A token must also mix at least two of
/// lowercase, uppercase and digits, since a run of one case joined by separators is a name.
fn entropy(patterns: &Patterns, text: &str) -> Vec<Span> {
    patterns
        .token
        .find_iter(text)
        .filter(|token| {
            let token = token.as_str();
            token.len() >= ENTROPY_MIN_LEN
                && !is_hex_digest(token)
                && !patterns.uuid.is_match(token)
                && !is_integrity_digest(token)
                && character_classes(token) >= 2
                && shannon_entropy(token) > ENTROPY_THRESHOLD
        })
        .filter_map(|token| span(token.start(), token.end(), Layer::Entropy))
        .collect()
}

/// A value that stands in for a secret rather than holding one: empty, already redacted, or
/// a reference to where the secret lives (`${NAME}`, `$NAME`, `{fromEnv:NAME}`, `{{ name }}`).
fn is_placeholder(value: &str) -> bool {
    value.is_empty()
        || value.contains(REDACTED)
        || value.starts_with('$')
        || value.starts_with("{fromEnv:")
        || value.starts_with("{{")
}

/// Whether a key's last word names a credential: `DB_PASSWORD`, `client_secret`,
/// `_authToken`, `api_key`. A key that names a property of a credential, such as
/// `PASSWORD_MIN_LENGTH`, ends in another word and is not one.
fn names_a_credential(key: &str) -> bool {
    let words = key_words(key);
    match words.as_slice() {
        [.., last] if CREDENTIAL_WORDS.contains(&last.as_str()) => true,
        [.., api, key] => api == "api" && key == "key",
        _ => false,
    }
}

/// A key split into lowercase words at separators and at camel-case boundaries, keeping an
/// acronym whole: `_authToken` is `auth token`, `APIKey` is `api key`, `sha256` is one word.
fn key_words(key: &str) -> Vec<String> {
    let chars: Vec<char> = key.chars().collect();
    let mut words = Vec::new();
    let mut word = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            continue;
        }
        let previous = i.checked_sub(1).and_then(|j| chars.get(j)).copied();
        let next = chars.get(i + 1).copied();
        let starts_word = c.is_ascii_uppercase()
            && match previous {
                Some(p) if p.is_ascii_lowercase() || p.is_ascii_digit() => true,
                Some(p) if p.is_ascii_uppercase() => next.is_some_and(|n| n.is_ascii_lowercase()),
                _ => false,
            };
        if starts_word && !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
        word.push(c.to_ascii_lowercase());
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

fn is_hex_digest(token: &str) -> bool {
    matches!(token.len(), 40 | 64) && token.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_integrity_digest(token: &str) -> bool {
    ["sha1-", "sha256-", "sha384-", "sha512-"]
        .iter()
        .any(|algorithm| token.starts_with(algorithm))
}

/// How many of lowercase, uppercase and digits a token uses.
fn character_classes(token: &str) -> usize {
    [
        token.chars().any(|c| c.is_ascii_lowercase()),
        token.chars().any(|c| c.is_ascii_uppercase()),
        token.chars().any(|c| c.is_ascii_digit()),
    ]
    .iter()
    .filter(|present| **present)
    .count()
}

/// Shannon entropy in bits per character.
fn shannon_entropy(token: &str) -> f64 {
    let mut counts = std::collections::BTreeMap::new();
    for c in token.chars() {
        *counts.entry(c).or_insert(0_usize) += 1;
    }
    let length = token.chars().count() as f64;
    counts
        .values()
        .map(|&count| {
            let p = count as f64 / length;
            -p * p.log2()
        })
        .sum()
}

/// Overlapping spans become one, counted under the most specific layer among them. Spans
/// that only touch stay two.
fn merge(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_by_key(|span| (span.start, std::cmp::Reverse(span.end)));
    let mut merged: Vec<Span> = Vec::new();
    for span in spans {
        match merged.last_mut() {
            Some(last) if span.start < last.end => {
                last.end = last.end.max(span.end);
                last.layer = last.layer.min(span.layer);
            }
            _ => merged.push(span),
        }
    }
    merged
}

/// One pass over the text, each merged span replaced by the literal and counted.
fn replace(text: &str, spans: Vec<Span>) -> Redaction {
    let mut out = String::with_capacity(text.len());
    let mut tally = Tally::default();
    let mut at = 0;
    for span in spans {
        out.push_str(&text[at..span.start]);
        out.push_str(REDACTED);
        at = span.end;
        tally.total += 1;
        let count = match span.layer {
            Layer::Format => &mut tally.by.format,
            Layer::Prefix => &mut tally.by.prefix,
            Layer::Uri => &mut tally.by.uri,
            Layer::Connection => &mut tally.by.connection,
            Layer::Keyvalue => &mut tally.by.keyvalue,
            Layer::Entropy => &mut tally.by.entropy,
        };
        *count += 1;
    }
    out.push_str(&text[at..]);
    Redaction { text: out, tally }
}

/// Run `plotplot redact` over `input`, the bytes read from standard input.
///
/// Exit 0 with the redacted text on `stdout`; exit 1 with nothing on `stdout` when the input
/// is not UTF-8, the patterns do not compile, or a stream cannot be written. Error messages never quote the input.
pub fn run(args: &RedactArgs, input: &[u8], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let Ok(text) = std::str::from_utf8(input) else {
        let _ = writeln!(
            stderr,
            "redact: standard input is not UTF-8; nothing written"
        );
        return 1;
    };
    let redaction = match redact(text) {
        Ok(redaction) => redaction,
        Err(error) => {
            let _ = writeln!(stderr, "redact: {error}; nothing written");
            return 1;
        }
    };
    if args.tally {
        let tally = match serde_json::to_string(&redaction.tally) {
            Ok(tally) => tally,
            Err(error) => {
                let _ = writeln!(stderr, "redact: the tally could not be encoded: {error}");
                return 1;
            }
        };
        let _ = writeln!(stderr, "{tally}");
    }
    match stdout.write_all(redaction.text.as_bytes()) {
        Ok(()) => 0,
        Err(error) => {
            let _ = writeln!(stderr, "redact: stdout: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use serde::Deserialize;

    use super::*;

    /// One row of the fixture table, `in` and `out` still base64 as they are at rest.
    #[derive(Deserialize)]
    struct Row {
        id: String,
        layer: String,
        #[serde(rename = "in")]
        input: String,
        out: String,
        count: usize,
    }

    fn rows() -> Vec<Row> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("contracts/fixtures/redaction/cases.jsonl");
        let table = std::fs::read_to_string(&path).expect("the fixture table");
        table
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_str(line).expect("a fixture row"))
            .collect()
    }

    fn decode(id: &str, field: &str, value: &str) -> String {
        let bytes = STANDARD
            .decode(value)
            .unwrap_or_else(|_| panic!("{id}: `{field}` is not base64"));
        String::from_utf8(bytes).unwrap_or_else(|_| panic!("{id}: `{field}` is not UTF-8"))
    }

    fn by_layer(tally: &Tally, layer: &str) -> usize {
        match layer {
            "entropy" => tally.by.entropy,
            "format" => tally.by.format,
            "prefix" => tally.by.prefix,
            "uri" => tally.by.uri,
            "connection" => tally.by.connection,
            "keyvalue" => tally.by.keyvalue,
            _ => 0,
        }
    }

    /// Every row of the contract's table, in order. Failures name the row id and never
    /// print a decoded value, because the values are shaped like credentials.
    #[test]
    fn every_fixture_row_redacts_to_its_out_with_its_tally() {
        let rows = rows();
        assert_eq!(rows.len(), 34, "the fixture table has 34 rows");
        for row in rows {
            let input = decode(&row.id, "in", &row.input);
            let want = decode(&row.id, "out", &row.out);
            let got = redact(&input).expect("the patterns compile");
            assert!(
                got.text == want,
                "{}: output does not match the fixture",
                row.id
            );
            assert_eq!(got.tally.total, row.count, "{}: tally total", row.id);
            if row.layer != "none" {
                assert!(
                    by_layer(&got.tally, &row.layer) >= 1,
                    "{}: the {} layer caught nothing",
                    row.id,
                    row.layer
                );
            }
        }
    }

    fn redacted(text: &str) -> Redaction {
        redact(text).expect("the patterns compile")
    }

    #[test]
    fn a_comparison_is_not_an_assignment() {
        let text = "if token == expected_value { return }";
        assert_eq!(redacted(text).text, text);
    }

    #[test]
    fn an_integrity_digest_is_never_a_secret() {
        let text = "\"integrity\": \"sha512-Zm9vYmFyQmF6UXV4MTIzNDU2Nzg5MGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6QUJDREVGR0g=\"";
        assert_eq!(redacted(text).text, text);
    }

    #[test]
    fn prose_that_puts_a_colon_after_a_credential_word_stays() {
        let text = "Token: rotate it every week";
        assert_eq!(redacted(text).text, text);
    }

    #[test]
    fn a_prefix_followed_by_lowercase_words_is_a_name() {
        let text = "see sk-learn-the-basics-of-the-library-first";
        assert_eq!(redacted(text).text, text);
    }

    #[test]
    fn a_reference_in_a_uri_password_stays() {
        let text = "postgres://app:${DB_PASSWORD}@db:5432/orders";
        assert_eq!(redacted(text).text, text);
    }

    #[test]
    fn a_quoted_connection_password_goes_whole_even_with_a_semicolon_inside() {
        let got = redacted("Server=db;Password='a;b c';Encrypt=true");
        assert_eq!(got.text, "Server=db;Password=[REDACTED];Encrypt=true");
        assert_eq!(got.tally.by.connection, 1);
        assert_eq!(got.tally.total, 1);
    }

    #[test]
    fn a_yaml_credential_with_crlf_endings_is_caught_on_its_own_line() {
        let got = redacted("user: app\r\npassword: plainword\r\nport: 5432\r\n");
        assert_eq!(
            got.text,
            "user: app\r\npassword: [REDACTED]\r\nport: 5432\r\n"
        );
    }

    #[test]
    fn a_private_key_cut_off_before_its_end_line_is_redacted_to_the_end() {
        // Assembled at run time, so this source carries no key header a scanner reads as a key.
        let header = format!("{0}BEGIN RSA PRIVATE {1}{0}", "-".repeat(5), "KEY");
        let got = redacted(&format!("key follows\n{header}\nMIIEabc\n"));
        assert_eq!(got.text, "key follows\n[REDACTED]");
        assert_eq!(got.tally.by.format, 1);
    }

    #[test]
    fn overlapping_spans_are_replaced_once_under_the_most_specific_layer() {
        let merged = merge(vec![
            Span {
                start: 4,
                end: 20,
                layer: Layer::Entropy,
            },
            Span {
                start: 0,
                end: 10,
                layer: Layer::Keyvalue,
            },
            Span {
                start: 20,
                end: 25,
                layer: Layer::Entropy,
            },
        ]);
        assert_eq!(
            merged,
            vec![
                Span {
                    start: 0,
                    end: 20,
                    layer: Layer::Keyvalue
                },
                Span {
                    start: 20,
                    end: 25,
                    layer: Layer::Entropy
                },
            ]
        );
    }

    #[test]
    fn keys_split_into_words_at_separators_and_camel_case() {
        assert_eq!(key_words("_authToken"), ["auth", "token"]);
        assert_eq!(key_words("APIKey"), ["api", "key"]);
        assert_eq!(
            key_words("PASSWORD_MIN_LENGTH"),
            ["password", "min", "length"]
        );
        assert!(names_a_credential("APIKey"));
        assert!(!names_a_credential("sha256"));
        assert!(!names_a_credential("PASSWORD_MIN_LENGTH"));
    }

    #[test]
    fn the_face_refuses_input_that_is_not_utf8_and_writes_nothing() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(
            &RedactArgs { tally: true },
            &[0x66, 0xff, 0x6f],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, 1);
        assert!(stdout.is_empty());
    }

    #[test]
    fn the_tally_is_one_json_object_with_every_layer() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(
            &RedactArgs { tally: true },
            b"prose",
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, 0);
        let tally: serde_json::Value = serde_json::from_slice(&stderr).expect("a JSON tally");
        assert!(tally["total"].is_u64());
        for layer in [
            "entropy",
            "format",
            "prefix",
            "uri",
            "connection",
            "keyvalue",
        ] {
            assert!(tally["by"][layer].is_u64(), "{layer}");
        }
    }
}
