use crate::errors::*;
use std::fs;
use std::path::PathBuf;

/// Deletes the entire `.tribble/` directory in the event of a corruption. In theory, that shouldn't actually be possible,
/// since there's no caching or similar that happens, and any corruptions would be bugs in Perseus (ignoring user tinkering).
pub fn delete_tribble_dir(dir: PathBuf) -> Result<(), DeleteError> {
    let target = dir.join(".tribble");
    // We'll only delete the directory if it exists, otherwise we're fine
    if target.exists() {
        if let Err(err) = fs::remove_dir_all(&target) {
            return Err(DeleteError::RemoveTribbleDirFailed {
                loc: target.to_str().map(|s| s.to_string()).unwrap(),
                source: err,
            });
        }
    }
    Ok(())
}

/// Deletes the `.tribble/dist/` directory. This is run before any command, since that directory can otherwise get clogged
/// with old workflows and the like.
pub fn delete_dist_dir(dir: PathBuf) -> Result<(), DeleteError> {
    let target = dir.join(".tribble/dist");
    // We'll only delete the directory if it exists, otherwise we're fine
    if target.exists() {
        if let Err(err) = fs::remove_dir_all(&target) {
            return Err(DeleteError::RemoveDistFailed {
                loc: target.to_str().map(|s| s.to_string()).unwrap(),
                source: err,
            });
        }
    }
    // No matter what, it's gone now, so recreate it
    // We also create parent directories because that's an issue for some reason in Docker
    if let Err(err) = fs::create_dir_all(&target) {
        return Err(DeleteError::RemoveDistFailed {
            loc: target.to_str().map(|s| s.to_string()).unwrap(),
            source: err,
        });
    }

    Ok(())
}
