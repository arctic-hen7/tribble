use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrepError {
    #[error("couldn't extract internal `.tribble/` directory, please ensure you have write permissions here")]
    ExtractionFailed {
        target_dir: Option<String>,
        #[source]
        source: std::io::Error
    },
    #[error("couldn't update your `.gitignore`, you should add `.tribble/` to it manually")]
    GitignoreUpdateFailed {
        #[source]
        source: std::io::Error
    }
}
