use std::env;

use perseus::RenderFnResultWithCause;
use serde::{Deserialize, Serialize};

use crate::{
    errors::ParserError,
    parser::{Config, Workflow},
};

#[derive(Serialize, Deserialize)]
pub struct WorkflowProps {
    pub workflow: Workflow,
    pub input_err_msg: String,
}

#[perseus::autoserde(build_state)]
pub async fn get_build_state(
    path: String,
    locale: String,
) -> RenderFnResultWithCause<WorkflowProps> {
    let root_cfg_path =
        env::var("TRIBBLE_CONF").unwrap_or_else(|_| "../../../tribble.yml".to_string());
    let root_cfg = Config::new(&root_cfg_path)?;
    let input_err_msg;
    // Get the workflows for the appropriate locale (if applicable)
    let workflows = match root_cfg {
        Config::Root { languages } => {
            // We want the language file for the current locale
            let lang_cfg_path = match languages.get(&locale) {
                Some(path) => path,
                // A language mismatch between Perseus and Tribble shouldn't be possible, because Tribble configures Perseus' locale settings
                None => unreachable!(),
            };
            let lang_cfg = Config::new(lang_cfg_path)?;
            match lang_cfg {
                Config::Language {
                    workflows,
                    input_err_msg: input_err_msg_l,
                } => {
                    input_err_msg = input_err_msg_l;
                    workflows
                }
                // A root file links to another root file (we only test one language in the build paths stage, so we may not have picked this up)
                Config::Root { .. } => {
                    return Err(ParserError::RootLinksToRoot {
                        filename: root_cfg_path,
                        linked: lang_cfg_path.to_string(),
                    }
                    .into())
                }
            }
        }
        Config::Language {
            workflows,
            input_err_msg: input_err_msg_l,
        } => {
            input_err_msg = input_err_msg_l;
            workflows
        }
    };
    // Strip off the `workflow/` section of the path (Perseus guarantees that it will start with this)
    let path = path.strip_prefix("workflow/").unwrap();
    // Each workflow should match exactly to a page path (the pages are generated from the keys of the `workflows` map)
    let workflow = match workflows.get(path) {
        Some(workflow) => workflow,
        None => unreachable!(),
    };

    Ok(WorkflowProps {
        workflow: workflow.clone(),
        input_err_msg,
    })
}
