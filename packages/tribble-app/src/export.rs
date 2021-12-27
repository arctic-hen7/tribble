use crate::errors::*;
use crate::{get_locales, get_templates_map, get_translations_manager, APP_ROOT};
use futures::executor::block_on;
use perseus::internal::build::build_app;
use perseus::internal::export::export_app;
use perseus::internal::get_path_prefix_server;
use perseus::SsrNode;

/// Exports the app. This acts as an app-specific alternative to the Perseus exporting logic, which means we need no
/// dependencies whatsoever for Tribble to run, making it truly programming language agnostic.
///
/// This is designed to be executed from a root directory that has a `.tribble/` folder with the necessary data.
pub fn export() -> Result<(), ExportError> {
    // The only plugins we use are tinker-time, so they don't matter here

    // We use a lot of custom stuff here to adapt it to the location of execution
    let immutable_store = perseus::stores::ImmutableStore::new(".tribble/perseus".to_string());
    // We don't need this in exporting, but the build process does
    let mutable_store =
        perseus::stores::FsMutableStore::new(".tribble/perseus/mutable".to_string());
    // Tribble does i18n outside Perseus, so this isn't a problem (it'll be a `DummyTranslator`)
    let translations_manager = block_on(get_translations_manager());
    let locales = get_locales();

    // Build the site for all the common locales (done in parallel), denying any non-exportable features
    let templates_map = get_templates_map::<SsrNode>();
    let build_fut = build_app(
        &templates_map,
        &locales,
        (&immutable_store, &mutable_store),
        &translations_manager,
        // We use another binary to handle normal building
        true,
    );
    if let Err(err) = block_on(build_fut) {
        return Err(ExportError::BuildFailed { source: err });
    }
    // Turn the build artifacts into self-contained static files
    let export_fut = export_app(
        &templates_map,
        // Perseus always uses one HTML file, and there's no point in letting a plugin change that
        ".tribble/utils/index.html",
        &locales,
        APP_ROOT,
        &immutable_store,
        &translations_manager,
        get_path_prefix_server(),
    );
    if let Err(err) = block_on(export_fut) {
        return Err(ExportError::ExportFailed { source: err });
    }

    // The static content is already in the `.tribble/` directory, so we don't have to worry about that
    // We don't have any static aliases, so we don't have to worry about those

    Ok(())
}
