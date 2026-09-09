//! The two facts only the build knows. The target triple cargo sets, which
//! `lock::platform()` reads back through `env!` so the lock's per-platform artifact is
//! chosen by the triple that actually produced the binary rather than by a guess; and the
//! SARIF `$schema` URL the contracts pin, lifted out of `contracts/pins.json` here so that a
//! pins file the stem cannot read fails the build instead of failing a gate run.

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=contracts/pins.json");

    match std::env::var("TARGET") {
        Ok(target) => println!("cargo::rustc-env=PLOTPLOT_TARGET={target}"),
        Err(error) => {
            println!("cargo::error=cargo did not set TARGET for the build script: {error}")
        }
    }

    match sarif_schema_url(include_str!("contracts/pins.json")) {
        Ok(url) => println!("cargo::rustc-env=PLOTPLOT_SARIF_SCHEMA_URL={url}"),
        Err(problem) => println!("cargo::error=contracts/pins.json: {problem}"),
    }
}

/// The `$schema` URL `contracts/pins.json` names for SARIF, or what is wrong with the pins.
fn sarif_schema_url(pins: &str) -> Result<String, String> {
    let pins: serde_json::Value =
        serde_json::from_str(pins).map_err(|error| format!("it is not JSON: {error}"))?;
    pins.pointer("/sarif_schema/url")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "it names no sarif_schema.url for the stem to write into a log".to_owned())
}
