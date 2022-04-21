mod error_pages;
pub mod errors;
pub mod parser;
mod svg;
mod templates;

#[cfg(all(not(target_arch = "wasm32"), feature = "export"))]
mod export;
// We want the CLI to be able to export the app directly (without relying on the Perseus CLI)
#[cfg(all(not(target_arch = "wasm32"), feature = "export"))]
pub use export::export;

use perseus::{Html, PerseusApp, PerseusRoot, Plugins};
use perseus_size_opt::{perseus_size_opt, SizeOpts};

#[perseus::main]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new()
        .template(templates::workflow::get_template)
        .error_pages(error_pages::get_error_pages)
        .plugins(Plugins::new().plugin(perseus_size_opt, SizeOpts::default_2018()))
        .index_view(|| {
            sycamore::view! {
                html {
                    head {
                        meta(charset = "UTF-8")
                        meta(name = "viewport", content = "width=device-width, initial-scale=1.0")
                        link(rel = "stylesheet", href = ".perseus/static/tailwind.css")
                    }
                    body {
                        PerseusRoot()
                    }
                }
            }
        })
}
