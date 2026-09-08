//! The one fact only cargo knows at build time: the target triple this binary is built for.
//! `lock::platform()` reads it back through `env!`, so the lock's per-platform artifact is
//! chosen by the triple that actually produced the binary rather than by a guess.

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    match std::env::var("TARGET") {
        Ok(target) => println!("cargo::rustc-env=PLOTPLOT_TARGET={target}"),
        Err(error) => {
            println!("cargo::error=cargo did not set TARGET for the build script: {error}")
        }
    }
}
