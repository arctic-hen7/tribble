use perseus::Template;
use sycamore::prelude::*;

#[perseus::template(IndexPage)]
#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    view! {
        p { "Hello World!" }
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::new("index").template(index_page).head(head)
}

#[perseus::head]
pub fn head() -> View<SsrNode> {
    view! {
        title { "Tribble" }
    }
}
