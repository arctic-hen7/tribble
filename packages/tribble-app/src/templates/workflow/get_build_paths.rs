use std::env;

use perseus::RenderFnResult;

use crate::{errors::ParserError, parser::Config};

// Note: for a workflow named `index`, we'll generate a path literally called `index`
// TODO maybe generatea  root page for selecting workflows?
pub async fn get_build_paths() -> RenderFnResult<Vec<String>> {
    // We need a root config file to work with
    // With the CLI, this variable will always be defined
    // In dev, we use the `basic` example
    let root_cfg_file_path =
        env::var("TRIBBLE_CONF").unwrap_or_else(|_| "../../../examples/basic.yml".to_string());
    let root_cfg = Config::new(&root_cfg_file_path)?;
    match root_cfg {
        Config::Root { languages } => {
            // We assume workflows are the same for all languages, so we can choose a random one
            match languages.keys().collect::<Vec<&String>>().get(0) {
                Some(key) => {
                    let language_cfg_path = languages.get(&key.to_string()).unwrap(); // We selected a random key, it certainly should have an entry!
                    let language_cfg = Config::new(language_cfg_path)?;
                    match language_cfg {
                        Config::Language { workflows, .. } => {
                            // For each workflow, generate a separate page
                            Ok(workflows.keys().cloned().collect::<Vec<String>>())
                        }
                        // If a root file links to another root file, that's an invalid structure
                        Config::Root { .. } => Err(ParserError::RootLinksToRoot {
                            filename: root_cfg_file_path,
                            linked: language_cfg_path.to_string(),
                        }
                        .into()),
                    }
                }
                // We obviously need at least one language file with stuff in it to generate anything
                None => Err(ParserError::NoLanguages {
                    filename: root_cfg_file_path,
                }
                .into()),
            }
        }
        Config::Language { workflows, .. } => {
            // For each workflow, generate a separate page
            Ok(workflows.keys().cloned().collect::<Vec<String>>())
        }
    }
}
