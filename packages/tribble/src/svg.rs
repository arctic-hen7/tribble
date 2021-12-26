/// A macro to make using SVGs in Sycamore significantly easier. The argument provided to this should best be a raw
/// string.
#[macro_export]
macro_rules! svg {
    ($svg:expr) => {
        view! {
            div(dangerously_set_inner_html = $svg)
        }
    };
}
