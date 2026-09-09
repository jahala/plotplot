//! The binary. It reads the arguments and the current directory, hands both to the library
//! as the repository root, and turns the number the library returns into a process status.
//! The library never reads the current directory; this is the only place that does.

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use plotplot::cli;

fn main() -> ExitCode {
    let args = cli::Args::parse();

    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();

    let root = match std::env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            let _ = writeln!(stderr, "the current directory could not be read: {error}");
            return ExitCode::from(1);
        }
    };

    let code = cli::run(args, &root, &mut stdout, &mut stderr);
    let _ = stdout.flush();
    let _ = stderr.flush();

    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
