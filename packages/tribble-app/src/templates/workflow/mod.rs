mod get_build_paths;
mod get_build_state;
mod view;

use perseus::{Html, Template};

pub fn get_template<G: Html>() -> Template<G> {
    Template::new("workflow")
        .template(view::workflow)
        .build_state_fn(get_build_state::get_build_state)
        .build_paths_fn(get_build_paths::get_build_paths)
}
