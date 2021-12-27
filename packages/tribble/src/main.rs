mod errors;
mod options;
mod prep;

use crate::errors::*;
use crate::options::*;
use crate::prep::prep;
use clap::Parser;
use fmterr::fmt_err;
use std::env;
use std::path::PathBuf;

/// The current version of the CLI, extracted from the crate version.
pub const TRIBBLE_VERSION: &str = env!("CARGO_PKG_VERSION");

// All this does is run the program and terminate with the acquired exit code
fn main() {
    // In development, we'll test one of the examples
    if cfg!(debug_assertions) {
        env::set_current_dir("../../examples").unwrap();
    }
    let exit_code = real_main();
    std::process::exit(exit_code)
}

// This manages error handling and returns a definite exit code to terminate with
fn real_main() -> i32 {
    // Get the working directory
    let dir = env::current_dir();
    let dir = match dir {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!(
                "{}",
                fmt_err(&PrepError::CurrentDirUnavailable { source: err })
            );
            return 1;
        }
    };
    let res = core(dir);
    match res {
        // If it worked, we pass the executed command's exit code through
        Ok(exit_code) => exit_code,
        // If something failed, we print the error to `stderr` and return a failure exit code
        Err(err) => {
            eprintln!("{}", fmt_err(&err));
            1
        }
    }
}

fn core(dir: PathBuf) -> Result<i32, Error> {
    // Parse the CLI options with `clap`
    let opts: Opts = Opts::parse();
    // If we're not cleaning up artifacts, create them if needed
    if !matches!(opts.subcmd, Subcommand::Clean) {
        prep(dir.clone())?;
    }
    let exit_code = match opts.subcmd {
        Subcommand::Build => {
            // TODO Build the user's app
            0
        }
        Subcommand::Serve => {
            // TODO Serve the user's app
            0
        }
        Subcommand::Clean => {
            // TODO Delete the `.tribble/` directory
            0
        }
        Subcommand::Deploy => {
            // TODO Deploy the app
            0
        }
    };
    Ok(exit_code)
}
