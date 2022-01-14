mod get_build_paths;
mod get_build_state;
mod parse_md;
mod view;

use perseus::{Html, SsrNode, Template};
use sycamore::view::View;

pub fn get_template<G: Html>() -> Template<G> {
    Template::new("workflow")
        .template(view::workflow)
        .head(head)
        .build_state_fn(get_build_state::get_build_state)
        .build_paths_fn(get_build_paths::get_build_paths)
}

#[perseus::head]
fn head(props: get_build_state::WorkflowProps) -> View<SsrNode> {
    sycamore::view! {
        title { (props.workflow.title) }
    }
}
