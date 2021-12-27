mod error_pages;
mod errors;
pub mod parser;
mod svg;
mod templates;

// #[cfg(target_arch = "wasm32")]
mod export;
// We want the CLI to be able to export the app directly (without relying on the Perseus CLI)
// #[cfg(target_arch = "wasm32")]
pub use export::export;

use perseus::define_app;

define_app! {
    templates: [
        crate::templates::index::get_template::<G>(),
        crate::templates::workflow::get_template::<G>()
    ],
    error_pages: crate::error_pages::get_error_pages()
}