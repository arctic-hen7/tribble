use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Parser(ParserError),
}
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("filesystem error occurred while attempting to parse config file at '{filename}")]
    FsError {
        filename: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing error occurred while attempting to deserialize config at '{filename}'")]
    ParseRawError {
        filename: String,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("the root config file at '{filename}' did not define any languages (you must define at least one)")]
    NoLanguages { filename: String },
    #[error("the root config file at '{filename}' linked to another root config file at '{linked}', but root config files can only link to language config files")]
    RootLinksToRoot { filename: String, linked: String },
}
