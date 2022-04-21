use crate::__perseus_main as main;
use crate::errors::*;
use perseus::internal::build::{build_app, BuildProps};
use perseus::internal::export::{export_app, ExportProps};
use perseus::internal::get_path_prefix_server;
use perseus::{PerseusApp, SsrNode};

/// Exports the app. This acts as an app-specific alternative to the Perseus exporting logic, which means we need no
/// dependencies whatsoever for Tribble to run, making it truly programming language agnostic.
///
/// This is designed to be executed from a root directory that has a `.tribble/` folder with the necessary data.
pub async fn export() -> Result<(), ExportError> {
    // The only plugins we use are tinker-time, so they don't matter here
    let app = main::<SsrNode>();
    let plugins = app.get_plugins();

    // We use a lot of custom stuff here to adapt it to the location of execution
    let immutable_store = perseus::stores::ImmutableStore::new(".tribble/perseus".to_string());
    // We don't need this in exporting, but the build process does
    let mutable_store =
        perseus::stores::FsMutableStore::new(".tribble/perseus/mutable".to_string());

    let locales = app.get_locales();
    // Generate the global state
    let gsc = app.get_global_state_creator();
    let global_state = match gsc.get_build_state().await {
        Ok(global_state) => global_state,
        Err(err) => {
            return Err(ExportError::GscFailed { source: err });
        }
    };
    let templates_map = app.get_templates_map();
    let index_view_str = app.get_index_view_str();
    let root_id = app.get_root();
    // This consumes `self`, so we get it finally
    // Tribble does i18n outside Perseus, so this isn't a problem (it'll be a `DummyTranslator`)
    let translations_manager = app.get_translations_manager().await;

    // Build the site for all the common locales (done in parallel), denying any non-exportable features
    let build_res = build_app(BuildProps {
        templates: &templates_map,
        locales: &locales,
        immutable_store: &immutable_store,
        mutable_store: &mutable_store,
        translations_manager: &translations_manager,
        global_state: &global_state,
        exporting: true,
    })
    .await;
    if let Err(err) = build_res {
        return Err(ExportError::BuildFailed { source: err });
    }
    // The app has now been built, so we can safely instantiate the HTML shell (which needs access to the render config, generated in the above build step)
    // It doesn't matter if the type parameters here are wrong, this function doesn't use them
    let index_view =
        PerseusApp::get_html_shell(index_view_str, &root_id, &immutable_store, &plugins).await;
    // Turn the build artifacts into self-contained static files
    let export_res = export_app(ExportProps {
        templates: &templates_map,
        html_shell: index_view,
        locales: &locales,
        immutable_store: &immutable_store,
        translations_manager: &translations_manager,
        path_prefix: get_path_prefix_server(),
        global_state: &global_state,
    })
    .await;
    if let Err(err) = export_res {
        return Err(ExportError::ExportFailed { source: err });
    }

    // The static content is already in the `.tribble/` directory, so we don't have to worry about that
    // We don't have any static aliases, so we don't have to worry about those

    Ok(())
}
