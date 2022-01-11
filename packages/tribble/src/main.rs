mod build;
mod delete;
mod errors;
mod options;
mod prep;
mod serve;

use crate::delete::delete_dist_dir;
use crate::delete::delete_tribble_dir;
use crate::errors::*;
use crate::options::*;
use crate::prep::prep;
use clap::Parser;
use fmterr::fmt_err;
use notify::{watcher, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tribble_app::parser::Config;

/// The current version of the CLI, extracted from the crate version.
pub const TRIBBLE_VERSION: &str = env!("CARGO_PKG_VERSION");

// All this does is run the program and terminate with the acquired exit code
#[tokio::main]
async fn main() {
    // In development, we'll test one of the examples
    if cfg!(debug_assertions) {
        env::set_current_dir("../../examples").unwrap();
    }
    let exit_code = real_main().await;
    std::process::exit(exit_code)
}

// This manages error handling and returns a definite exit code to terminate with
async fn real_main() -> i32 {
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
    let res = core(dir).await;
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

async fn core(dir: PathBuf) -> Result<i32, Error> {
    // Parse the CLI options with `clap`
    let opts: Opts = Opts::parse();
    let root_cfg_path = opts.config;
    // Set the `TRIBBLE_CONF` environment variable to what the user provided (used by the static exporting binary)
    env::set_var("TRIBBLE_CONF", &root_cfg_path);
    // If we're not cleaning up artifacts, create them if needed and remove the `dist/` directory
    if !matches!(opts.subcmd, Subcommand::Clean) {
        prep(dir.clone())?;
        delete_dist_dir(dir.clone())?;
    }
    let exit_code = match opts.subcmd {
        Subcommand::Build => {
            // Build the user's app
            crate::build::build(dir)?
        }
        Subcommand::Serve {
            no_build,
            host,
            port,
            watch,
        } => {
            // Start up the server as another task after an initial build
            if !no_build {
                let build_exit_code = crate::build::build(dir.clone())?;
                if build_exit_code != 0 {
                    return Ok(build_exit_code);
                }
            }

            if watch {
                let dir_2 = dir.clone();
                tokio::spawn(async move { crate::serve::serve(dir_2, host, port).await });
                // Now watch for changes
                let (tx, rx) = channel();
                let mut watcher = watcher(tx, Duration::from_secs(2))
                    .map_err(|err| ServeError::WatcherSetupFailed { source: err })?;
                // Watch the root configuration
                watcher
                    .watch(&root_cfg_path, RecursiveMode::Recursive)
                    .map_err(|err| ServeError::WatchFileFailed {
                        filename: root_cfg_path.clone(),
                        source: err,
                    })?;
                // Parse that to get any language files
                let cfg = Config::new(&root_cfg_path)
                    .map_err(|err| ServeError::ParserError { source: err })?;
                if let Config::Root { languages } = cfg {
                    for (_, lang_file_cfg_path) in languages {
                        watcher
                            .watch(&lang_file_cfg_path, RecursiveMode::Recursive)
                            .map_err(|err| ServeError::WatchFileFailed {
                                filename: lang_file_cfg_path,
                                source: err,
                            })?
                    }
                }

                let res: Result<i32, Error> = loop {
                    match rx.recv() {
                        Ok(_) => {
                            // Delete the distribution artifacts (the serve ris hilariously fine with this)
                            delete_dist_dir(dir.clone())?;
                            // Regardless of the event type, rebuild the app
                            if !no_build {
                                let build_exit_code = crate::build::build(dir.clone())?;
                                if build_exit_code != 0 {
                                    break Ok(build_exit_code);
                                }
                            }
                            // TODO Reload the browser automatically
                        }
                        Err(err) => break Err(ServeError::WatcherError { source: err }.into()),
                    }
                };
                return res;
            } else {
                crate::serve::serve(dir, host, port).await;
                0
            }
        }
        Subcommand::Clean => {
            // Delete the `.tribble/` directory
            delete_tribble_dir(dir)?;
            0
        }
        Subcommand::Deploy { output, path } => {
            // Set the base path in Perseus based on `--path`
            env::set_var("PERSEUS_BASE_PATH", path);
            // Build the app
            let build_exit_code = crate::build::build(dir.clone())?;
            if build_exit_code != 0 {
                return Ok(build_exit_code);
            }
            // Move the contents of `.tribble/dist` out to the output directory
            let from = dir.join(".tribble/dist");
            if let Err(err) = fs::rename(&from, &output) {
                return Err(Error::DeployError {
                    from: from.to_str().map(|s| s.to_string()).unwrap(),
                    to: output,
                    source: err,
                });
            }

            0
        }
    };
    Ok(exit_code)
}
