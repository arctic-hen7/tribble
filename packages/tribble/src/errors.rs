#![allow(clippy::enum_variant_names)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    PrepError(#[from] PrepError),
    #[error(transparent)]
    BuildError(#[from] BuildError),
    #[error(transparent)]
    DeleteError(#[from] DeleteError),
    // There's only one kind of deployment error, if we fail to move `.tribble/dist/` to the given output directory
    #[error("couldn't move generated tribble files from '{from}' to '{to}'")]
    DeployError {
        from: String,
        to: String,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    ServeError(#[from] ServeError),
}

#[derive(Error, Debug)]
pub enum PrepError {
    #[error("couldn't extract internal `.tribble/` directory, please ensure you have write permissions here")]
    ExtractionFailed {
        target_dir: Option<String>,
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't update your `.gitignore`, you should add `.tribble/` to it manually")]
    GitignoreUpdateFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't get your current directory (this is probably an error in your system configuration)")]
    CurrentDirUnavailable {
        #[source]
        source: std::io::Error,
    },
}
#[derive(Error, Debug)]
pub enum BuildError {
    #[error("couldn't export the tribble app to static files")]
    ExportFailed {
        #[source]
        source: tribble_app::errors::ExportError,
    },
    #[error("couldn't copy perseus output from '{from}' to '{to}'")]
    CopyOutputFailed {
        from: String,
        to: String,
        #[source]
        source: fs_extra::error::Error,
    },
    #[error("couldn't copy tribble static files from '{from}' to '{to}'")]
    CopyStaticFailed {
        from: String,
        to: String,
        #[source]
        source: fs_extra::error::Error,
    },
    #[error("couldn't rename perseus output from '{from}' to '{to}'")]
    RenameOutputFailed {
        from: String,
        to: String,
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't copy pre-built wasm bundle from '{from}' to '{to}'")]
    CopyWasmBundleFailed {
        from: String,
        to: String,
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't copy pre-built js bundle from '{from}' to '{to}'")]
    CopyJsBundleFailed {
        from: String,
        to: String,
        #[source]
        source: std::io::Error,
    },
}
#[derive(Error, Debug)]
pub enum DeleteError {
    #[error("couldn't remove tribble directory at '{loc}'")]
    RemoveTribbleDirFailed {
        loc: String,
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't remove distribution artifacts at '{loc}'")]
    RemoveDistFailed {
        loc: String,
        #[source]
        source: std::io::Error,
    },
}
#[derive(Error, Debug)]
pub enum ServeError {
    #[error("couldn't set up file watcher, please check your system configuration")]
    WatcherSetupFailed {
        #[source]
        source: notify::Error,
    },
    #[error("an error occurred while watching files for changes")]
    WatcherError {
        #[source]
        source: std::sync::mpsc::RecvError,
    },
    #[error("couldn't watch file at '{filename}', try re-running the command")]
    WatchFileFailed {
        filename: String,
        #[source]
        source: notify::Error,
    },
    #[error("couldn't parse your tribble configuration")]
    ParserError {
        #[source]
        source: tribble_app::errors::ParserError,
    },
}
