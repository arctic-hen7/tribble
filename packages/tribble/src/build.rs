use crate::errors::*;
use std::fs;
use std::path::PathBuf;

/// Shorthand macro for copying a directory.
macro_rules! copy_dir {
    ($from:expr, $to:expr, $err:ident) => {{
        let from = &$from;
        let to = &$to;
        if let Err(err) = fs_extra::dir::copy(&from, &to, &fs_extra::dir::CopyOptions::new()) {
            return Err(BuildError::$err {
                to: to.to_str().map(|s| s.to_string()).unwrap(),
                from: from.to_str().map(|s| s.to_string()).unwrap(),
                source: err,
            }
            .into());
        }
    }};
}

/// Builds the user's configuration file into a fully-fledged app. This mostly consists of file juggling.
pub fn build(dir: PathBuf) -> Result<i32, BuildError> {
    let dir = dir.join(".tribble");
    // Run the static exporting code in the app
    match tribble_app::export() {
        Ok(()) => (),
        Err(err) => return Err(BuildError::ExportFailed { source: err }),
    };
    // Copy the exported output into the root of `.tribble/` and rename that to `dist/`
    copy_dir!(dir.join("perseus/exported"), dir, CopyOutputFailed);
    let from = dir.join("exported");
    let to = dir.join("dist");
    if let Err(err) = fs::rename(&from, &to) {
        return Err(BuildError::RenameOutputFailed {
            to: to.to_str().map(|s| s.to_string()).unwrap(),
            from: from.to_str().map(|s| s.to_string()).unwrap(),
            source: err,
        });
    }
    // Copy the `utils/static/` directory into `dist/.perseus/`
    copy_dir!(
        dir.join("utils/static"),
        dir.join("dist/.perseus"),
        CopyStaticFailed
    );
    // Copy the bundles into `dist/.perseus/`
    // `bundle.wasm`
    let from = dir.join("utils/bundle.wasm");
    let to = dir.join("dist/.perseus/bundle.wasm");
    if let Err(err) = fs::copy(&from, &to) {
        return Err(BuildError::CopyWasmBundleFailed {
            to: to.to_str().map(|s| s.to_string()).unwrap(),
            from: from.to_str().map(|s| s.to_string()).unwrap(),
            source: err,
        });
    }
    // `bundle.js`
    let from = dir.join("utils/bundle.js");
    let to = dir.join("dist/.perseus/bundle.js");
    if let Err(err) = fs::copy(&from, &to) {
        return Err(BuildError::CopyJsBundleFailed {
            to: to.to_str().map(|s| s.to_string()).unwrap(),
            from: from.to_str().map(|s| s.to_string()).unwrap(),
            source: err,
        });
    }

    Ok(0)
}
