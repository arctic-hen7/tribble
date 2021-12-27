use std::path::PathBuf;

use include_dir::{include_dir, Dir};
use crate::errors::*;
use std::fs::{self, OpenOptions};
use std::io::Write;

// We include the `.tribble/` directory because it contains pre-built Wasm artifacts
static TRIBBLE_DIR: Dir = include_dir!("./.tribble");

/// Prepares the directory for running Tribble by writing the directory. Tribble has no prerequisites (not even `cargo`), so
/// this doesn't need to do any prerequisite checking.
pub fn prep(dir: PathBuf) -> Result<(), PrepError> {
    let tribble_dir = dir.join("./.tribble");
    if tribble_dir.exists() {
        // The directory already exists, so we'll assume we don't need to do anything
        // This will run before every command, so we don't want to be needlessly recreating things
        Ok(())
    } else {
        // Write the stored directory to that location, creating the directory first
        if let Err(err) = fs::create_dir(&tribble_dir) {
            return Err(PrepError::ExtractionFailed {
                target_dir: tribble_dir.to_str().map(|s| s.to_string()),
                source: err,
            });
        }
        // Notably, this function will not do anything or tell us if the directory already exists...
        if let Err(err) = TRIBBLE_DIR.extract(&tribble_dir) {
            return Err(PrepError::ExtractionFailed {
                target_dir: tribble_dir.to_str().map(|s| s.to_string()),
                source: err,
            });
        }

        // If we aren't already gitignoring the Tribble directory, update `.gitignore` to do so
        if let Ok(contents) = fs::read_to_string(".gitignore") {
            if contents.contains(".tribble/") {
                return Ok(());
            }
        }
        let file = OpenOptions::new()
            .append(true)
            .create(true) // If it doesn't exist, create it
            .open(".gitignore");
        let mut file = match file {
            Ok(file) => file,
            Err(err) => return Err(PrepError::GitignoreUpdateFailed { source: err }),
        };
        // Check for errors with appending to the file
        if let Err(err) = file.write_all(b".tribble/") {
            return Err(PrepError::GitignoreUpdateFailed { source: err });
        }
        Ok(())
    }
}
