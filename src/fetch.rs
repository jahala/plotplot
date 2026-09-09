//! The one place the stem reaches the network.
//!
//! `lock verify` decides things about bytes: is this digest the digest the lock pins, does
//! this artifact carry a manifest, where does the executable go. None of that needs a
//! socket. So the socket lives behind [`Fetch`], the deciding lives in [`crate::lock`], and
//! a test hands the deciding a map of bytes instead of a network. The unit under test is
//! never replaced by that: the verification and placement logic is the same code either way.

use crate::error::{Error, Result};
use crate::manifest::one_line;

/// The most bytes the stem will hold in memory for one artifact.
///
/// A pinned judge is a release tarball of a small tool; a hundred and twenty-eight mebibytes
/// is far past any of them and still far short of exhausting a laptop. A server that sends
/// more is answered with [`Error::Fetch`] rather than with the machine's memory.
pub const MAX_ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;

/// Bytes from a url. The whole artifact, because its digest is over the whole artifact.
pub trait Fetch {
    /// The bytes `url` serves.
    ///
    /// # Errors
    ///
    /// [`Error::Fetch`] naming the url and what went wrong: a status outside 2xx, a name
    /// that does not resolve, a connection that failed, a body past [`MAX_ARTIFACT_BYTES`].
    fn fetch(&self, url: &str) -> Result<Vec<u8>>;
}

/// The real network, over ureq's rustls.
pub struct Https;

impl Fetch for Https {
    fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        let refuse = |problem: String| Error::Fetch {
            url: url.to_owned(),
            problem,
        };

        let mut response = ureq::get(url).call().map_err(|error| {
            refuse(match &error {
                ureq::Error::StatusCode(code) => format!("the server answered {code}"),
                other => one_line(&other.to_string()),
            })
        })?;

        // ureq turns 4xx and 5xx into `StatusCode` above, so what reaches here is a status
        // the client accepted. A 1xx or 3xx that survived redirect following carries no
        // artifact, and the digest of an empty body would be a lie about what was fetched.
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(refuse(format!("the server answered {status}")));
        }

        response
            .body_mut()
            .with_config()
            .limit(MAX_ARTIFACT_BYTES)
            .read_to_vec()
            .map_err(|error| refuse(one_line(&error.to_string())))
    }
}

/// A [`Fetch`] that serves bytes from a map and counts what it was asked for.
///
/// Test-only, and deliberately so: it stands in for the socket, never for the code that
/// decides what the bytes mean.
#[cfg(test)]
pub struct FromMap {
    bytes: std::collections::BTreeMap<String, Vec<u8>>,
    calls: std::cell::RefCell<Vec<String>>,
}

#[cfg(test)]
impl FromMap {
    /// A fetcher serving one url.
    pub fn serving(url: &str, bytes: Vec<u8>) -> Self {
        let mut map = std::collections::BTreeMap::new();
        map.insert(url.to_owned(), bytes);
        Self {
            bytes: map,
            calls: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// Every url this fetcher was asked for, in the order it was asked.
    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

#[cfg(test)]
impl Fetch for FromMap {
    fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        self.calls.borrow_mut().push(url.to_owned());
        self.bytes.get(url).cloned().ok_or_else(|| Error::Fetch {
            url: url.to_owned(),
            problem: "this fetcher serves no bytes for it".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://example.invalid/weeder-aarch64-apple-darwin.tar.gz";

    #[test]
    fn the_map_serves_the_bytes_it_was_given_and_records_the_call() {
        let fetcher = FromMap::serving(URL, b"artifact".to_vec());
        assert_eq!(fetcher.fetch(URL).expect("the mapped bytes"), b"artifact");
        assert_eq!(fetcher.calls(), [URL]);
    }

    #[test]
    fn the_map_refuses_a_url_it_does_not_serve_and_still_records_the_call() {
        let fetcher = FromMap::serving(URL, b"artifact".to_vec());
        let other = "https://example.invalid/tilth-aarch64-apple-darwin.tar.gz";
        match fetcher.fetch(other) {
            Err(Error::Fetch { url, problem }) => {
                assert_eq!(url, other);
                assert!(!problem.is_empty());
            }
            other => panic!("got {other:?}"),
        }
        assert_eq!(fetcher.calls(), [other]);
    }

    #[test]
    fn a_url_that_is_not_a_url_is_a_fetch_error_and_never_a_panic() {
        match Https.fetch("not a url") {
            Err(Error::Fetch { url, problem }) => {
                assert_eq!(url, "not a url");
                assert!(!problem.is_empty());
                assert!(!problem.contains('\n'), "{problem}");
            }
            other => panic!("got {other:?}"),
        }
    }
}
