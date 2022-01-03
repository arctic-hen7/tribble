mod error_pages;
pub mod errors;
pub mod parser;
mod svg;
mod templates;

// #[cfg(target_arch = "wasm32")]
mod export;
// We want the CLI to be able to export the app directly (without relying on the Perseus CLI)
// #[cfg(target_arch = "wasm32")]
pub use export::export;

use perseus::{define_app, Plugins};
use perseus_size_opt::{perseus_size_opt, SizeOpts};

define_app! {
    templates: [
        crate::templates::workflow::get_template::<G>()
    ],
    error_pages: crate::error_pages::get_error_pages(),
    plugins: Plugins::new().plugin(perseus_size_opt, SizeOpts::default())
}
