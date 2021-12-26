mod error_pages;
mod errors;
pub mod parser;
mod svg;
mod templates;

use perseus::define_app;

define_app! {
    templates: [
        crate::templates::index::get_template::<G>(),
        crate::templates::workflow::get_template::<G>()
    ],
    error_pages: crate::error_pages::get_error_pages()
}
