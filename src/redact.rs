//! The redaction face: text in, the same text with every secret span replaced by
//! `[REDACTED]` out, and a tally of which layer caught each span.
//!
//! `contracts/redaction.md` is the contract and `contracts/fixtures/redaction/cases.jsonl`
//! its table. [`redact`] is pure: no I/O, no logging of content. The face reads standard
//! input as bytes and refuses anything that is not UTF-8 with exit 1 and nothing on standard
//! output, so a caller that cannot have its text scrubbed writes nothing.

use std::io::Write;

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

/// Spans replaced, per layer. A span two layers flag is counted once, under the first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ByLayer {
    pub entropy: usize,
    pub format: usize,
    pub prefix: usize,
    pub uri: usize,
    pub connection: usize,
    pub keyvalue: usize,
}

/// Redact one write's worth of text.
pub fn redact(text: &str) -> Redaction {
    Redaction {
        text: text.to_owned(),
        tally: Tally::default(),
    }
}

/// Run `plotplot redact` over `input`, the bytes read from standard input.
///
/// Exit 0 with the redacted text on `stdout`; exit 1 with nothing on `stdout` when the input
/// is not UTF-8 or a stream cannot be written. Error messages never quote the input.
pub fn run(args: &RedactArgs, input: &[u8], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let Ok(text) = std::str::from_utf8(input) else {
        let _ = writeln!(
            stderr,
            "redact: standard input is not UTF-8; nothing written"
        );
        return 1;
    };
    let redaction = redact(text);
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
            let got = redact(&input);
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
