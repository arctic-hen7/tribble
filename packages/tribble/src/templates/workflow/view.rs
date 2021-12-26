use super::get_build_state::WorkflowProps;
use crate::parser::{
    Endpoint, Input, InputSectionElem, InputType, Section, SectionElem, SelectOption,
};
use crate::svg;
use js_sys::Function;
use std::collections::HashMap;
use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlOptionElement;

/// The context of a workflow (these need to be accessed by multiple different parts of the workflow).
#[derive(Clone, Debug)]
struct WorkflowCtx {
    /// The history of the user's progression through different sections, along with the different tags they've accumulated. If the user moves back, we'll update `history_pos`, if they
    /// then act without using the forward button (even if they do the same thing as it would've), this will be truncated to their position (but their form inputs are preserved
    /// regardless).
    ///
    /// New sections are added here immediately, and their tags are added afterwards.
    history: Signal<Vec<SectionResult>>,
    /// The user's current position in their history.
    history_pos: Signal<usize>,
    /// The current location (either a section or an endpoint).
    loc: Signal<String>,
    /// The values types in different inputs, which can be later referenced for interpolation. Selects with multiple values submit their values as a comma-delimited list.
    form_values: Signal<HashMap<String, Signal<String>>>,
}
impl WorkflowCtx {
    fn new(index_loc: String) -> Self {
        Self {
            loc: Signal::new(index_loc.clone()),
            // The history should start on the first section
            history: Signal::new(vec![SectionResult {
                name: index_loc,
                tags: Vec::new(),
            }]),
            history_pos: Signal::new(0), // There's no history at this point, so this is safe
            form_values: Signal::default(),
        }
    }
}

/// The results from a section. A vector of these can be used to track history.
#[derive(Clone, Debug)]
struct SectionResult {
    tags: Vec<String>,
    name: String,
}

#[perseus::template(Workflow)]
#[component(Workflow<G>)]
pub fn workflow(props: WorkflowProps) -> View<G> {
    let index_loc = props.workflow.index.clone();
    // If we're in the browser, immediately tell it that we want to prompt the user before they leave the page
    // we'll only actually do this if we're in dev mode
    if G::IS_BROWSER {
        let window = web_sys::window().unwrap();
        #[cfg(debug_assertions)]
        window.set_onbeforeunload(Some(&Function::new_with_args(
            "ev",
            "ev.preventDefault();ev.returnValue = \"\"",
        )));
    }

    view! {
        // We pass tags around with context to avoid throwing `Signal`s over the place
        ContextProvider(ContextProviderProps {
            value: WorkflowCtx::new(index_loc),
            children: || view! {
                WorkflowInner(props)
            }
        })
    }
}

#[component(WorkflowInner<G>)]
pub fn workflow_inner(
    WorkflowProps {
        workflow,
        input_err_msg,
    }: WorkflowProps,
) -> View<G> {
    // This will be the name of a section, or, if it's prefixed with `endpoint:`, an endpoint
    let loc = use_context::<WorkflowCtx>().loc;
    // This will either store an endpoint or a section, fully rendered
    let page: ReadSignal<View<G>> = create_memo(cloned!(workflow, loc => move || {
        let loc = &*loc.get();
        if loc.starts_with("endpoint:") {
            let loc = loc.strip_prefix("endpoint:").unwrap();
            let endpoint_props = match workflow.endpoints.get(loc) {
                Some(props) => props,
                None => todo!("handle errors in pages (no such endpoint)")
            };
            match endpoint_props {
                Endpoint::Report { preamble, text } => view! {
                    RenderReportEndpoint(RenderReportEndpointProps { preamble: preamble.to_string(), text: text.to_string() })
                },
                Endpoint::Instructional(text) => {
                    let text = text.to_string();
                    view! {
                        p { (text) }
                    }
                }
            }
        } else {
            let section_props = match workflow.sections.get(loc) {
                Some(props) => RenderSectionProps { section: props.clone(), input_err_msg: input_err_msg.clone(), name: loc.to_string() },
                None => todo!("handle errors in pages (no such section)")
            };
            view! {
                RenderSection(section_props)
            }
        }
    }));

    view! {
        // We set the caret color at the top-level (changes the outlines of form inputs, cursor color, etc.)
        div(class = "flex justify-center items-center w-full h-full") {
            div(class = "section-container xs:shadow-md xs:rounded-lg text-center flex-col", id = "section-content") {
                HistoryBreadcrumbs()
                // We want to alert screenreaders that this entire section can be swapped out for new content
                main(class = "section-content", aria-live = "assertive", aria-atomic = true) {
                    (*active_page.get())
                }
            }
        }
    }
}

struct RenderSectionProps {
    section: Section,
    input_err_msg: String,
    name: String,
}

/// Renders a section. We loop through the elements without keying or the like because, every time we re-render the list of props, we'll be changing all of them.
#[component(RenderSection<G>)]
fn render_section(
    RenderSectionProps {
        section,
        input_err_msg,
        name,
    }: RenderSectionProps,
) -> View<G> {
    // We keep a local map of form values that we'll add to the global one on a progression (otherwise we're doing unecessary context reads)
    // We make this reactive at the value level, because each value will be updating (whereas the global one has values set, because they're submitted after the section is moved on from)
    // This also stores the input's properties and a reactive boolean that will be `true` if an error message needs to be shown
    #[allow(clippy::type_complexity)]
    let form_values: Signal<HashMap<String, (Signal<String>, InputSectionElem, Signal<bool>)>> =
        Signal::new(HashMap::new());

    let ctx = use_context::<WorkflowCtx>();
    let elems = View::new_fragment(
        section
            .iter()
            .map(cloned!(ctx => move |section_elem| {
                let rendered = match section_elem {
                    SectionElem::Text(text) => {
                        let text = text.clone();
                        view! {
                            p { (text) }
                        }
                    },
                    SectionElem::Progression { text, link, tags } => {
                        let text = text.to_string();
                        let link = link.to_string();
                        let new_tags = tags.clone();
                        let progression_handler = cloned!(ctx, form_values, name => move |_| {
                            // If the user selects this progression, we need to set the new location and update the tags
                            let history = (*ctx.history.get()).clone();
                            let mut tags = Vec::new(); // This is for just the tags accumulated in this section
                            tags.extend(new_tags.iter().cloned());
                            // All the form values for this section should be sent to the global store for later inteprolation
                            let form_values_global = (*ctx.form_values.get()).clone();
                            let mut do_change = true;
                            for (_id, (value_signal, input, show_err)) in form_values.get().iter() {
                                let value = (*value_signal.get()).clone();
                                if value.is_empty() && !input.optional {
                                    show_err.clone().set(true);
                                    do_change = false;
                                } else {
                                    show_err.clone().set(false);
                                    // If this is a select input, some of its options might want to add tags if they've been selected
                                    if let Input::Select { options, .. } = input.input.clone() {
                                        // We need to create a map of simple option text names to the tags they might add
                                        let options_simple: HashMap<String, Vec<String>> = options.iter().map(|opt| match opt {
                                            SelectOption::Simple(text) => (text.to_string(), vec![]),
                                            SelectOption::WithTags { text, tags } => (text.to_string(), tags.clone())
                                        }).collect();
                                        let values_vec = value.split(", "); // This is the reason we don't allow commas in select options!
                                        for selected_value in values_vec {
                                            let tags_to_add = match options_simple.get(selected_value) {
                                                Some(tags) => tags,
                                                // This is impossible because we're indexing based on options the user only typed in one place
                                                None => unreachable!()
                                            }.clone();
                                            tags.extend(tags_to_add);
                                        }
                                    }
                                    // Do the same for boolean inputs
                                    if let Input::Text { input_type: InputType::Boolean { tags: Some(new_tags) } } = input.input.clone() {
                                        if value == "true" {
                                            tags.extend(new_tags);
                                        }
                                    }
                                    // The value has already been registered globally, so we don't need to do any more
                                }
                            }
                            if do_change {
                                ctx.form_values.set(form_values_global);
                                let history_pos = *ctx.history_pos.get();
                                // The history position points to an element that contains the current section result (with tags waiting to be filled out), so delete everything
                                // after that (in case we've gone back in the history), a progression resets all following history
                                let mut history: Vec<SectionResult> = history
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| {
                                        // Only permit elements that are on or below the current history position (which is this section)
                                        i <= &history_pos
                                    })
                                    .map(|(_, v)| v)
                                    .cloned()
                                    .collect();
                                // Update the history for this section with the tags we've accumulated
                                history[history_pos] = SectionResult {
                                    name: name.clone(),
                                    tags
                                };
                                // Add a history element for the next section we're about to go to
                                history.push(SectionResult {
                                    name: link.clone(), // This could be an endpoint, in which case it won't ever accumulate any tags, so there are no problems there
                                    tags: Vec::new() // This will be filled out when we reach the next progression element
                                });
                                // Update the user's position in the history to the next section they're about to go to
                                ctx.history_pos.set(history_pos + 1);
                                ctx.history.set(history);
                                // This reactively updates the section being displayed to the user (though we can do more stuff after this if we want)
                                ctx.loc.set(link.clone());
                            }
                        });
                        view! {
                            button(
                                on:click = progression_handler,
                                class = "group inline-flex items-center p-5 text-lg shadow-md hover:shadow-lg transition-shadow duration-200 rounded-lg"
                            ) {
                                (text)
                                div(class = "h-5 w-5 group-hover:ml-1 transition-all ease-in-out duration-200") {
                                    (svg!(r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>"#))
                                }
                            }
                        }
                    },
                    SectionElem::Input(input_props @ InputSectionElem { id, default, label, input, optional: _ }) => {
                        // If we've moved back through the history, there may be records for this input (which we should autofill)
                        let mut form_values_map = (*form_values.get()).clone();
                        let show_err = Signal::new(false);
                        let mut form_values_global = (*ctx.form_values.get()).clone();
                        let input_value = if form_values_global.contains_key(id) {
                            Signal::new((*form_values_global.get(id).unwrap().get()).clone())
                        } else {
                            Signal::new(String::new())
                        };
                        // Register the value locally (so that progression elements can play with it)
                        form_values_map.insert(id.to_string(), (input_value.clone(), input_props.clone(), show_err.clone()));
                        form_values.set(form_values_map);
                        // Register the value globally (that way values are still saved even if the user moves back in history before submitting this section)
                        form_values_global.insert(id.to_string(), input_value.clone());
                        ctx.form_values.set(form_values_global);

                        // If we have pre-existing data, we'll override the default
                        let default = if !input_value.get().is_empty() {
                            (*input_value.get()).clone()
                        } else {
                            default.clone().unwrap_or_else(|| "".to_string())
                        };
                        // Without this, the default values don't actually do anything
                        // We still set them with `value` though for progressive enhancement and accessibility
                        input_value.set(default.clone());

                        let id = id.to_string();
                        let id_for_err_label = id.clone();
                        let label = label.clone();

                        let err_label = create_memo(cloned!(show_err, input_err_msg, id_for_err_label => move || {
                            if *show_err.get() {
                                let id_for_err_label = id_for_err_label.clone();
                                view! {
                                    label(for = id_for_err_label) { (input_err_msg) }
                                }
                            } else {
                                View::empty()
                            }
                        }));

                        let input_rendered = match input {
                            Input::Text { input_type } => {
                                // We make all the placeholders empty because that allows the CSS `:placeholder-shown` selector to work
                                match input_type {
                                    // Multiline inputs use a `textarea` rather than an `input`, so we split off here
                                    InputType::Multiline => view! {
                                        // We want to keep the integrity of the page, so it's only resizeable in the y-direction
                                        label(class = "custom-input") {
                                            textarea(bind:value = input_value, class = "resize-y", placeholder = "") { (default) }
                                            span() { (label) }
                                        }
                                        ((*err_label.get()).clone())
                                    },
                                    // A lot of the other input types are the same (other than their `type`), except for the ones with extra properties
                                    InputType::Number { min, max } => match (*min, *max) {
                                        (Some(min), Some(max)) => view! {
                                            label(class = "custom-input") {
                                                input(bind:value = input_value, type = "number", min = min, max = max, value = default, id = id, placeholder = "") {}
                                                span() { (label) }
                                            }
                                            ((*err_label.get()).clone())
                                        },
                                        (Some(min), None) => view! {
                                            label(class = "custom-input") {
                                                input(bind:value = input_value, type = "number", min = min, value = default, id = id, placeholder = "") {}
                                                span() { (label) }
                                            }
                                            ((*err_label.get()).clone())
                                        },
                                        (None, Some(max)) => view! {
                                            label(class = "custom-input") {
                                                input(bind:value = input_value, type = "number", max = max, value = default, id = id, placeholder = "") {}
                                                span() { (label) }
                                            }
                                            ((*err_label.get()).clone())
                                        },
                                        (None, None) => view! {
                                            label(class = "custom-input") {
                                                input(bind:value = input_value, type = "number", value = default, id = id, placeholder = "") {}
                                                span() { (label) }
                                            }
                                            ((*err_label.get()).clone())
                                        },
                                    },
                                    InputType::Range { min, max } => {
                                        let min = *min;
                                        let max = *max;
                                        view! {
                                            // For a range, the min/max are mandatory
                                            label(class = "custom-input") {
                                                input(bind:value = input_value, type = "range", min = min, max = max, value = default, id = id, placeholder = "") {}
                                                span() { (label) }
                                            }
                                            ((*err_label.get()).clone())
                                        }
                                    },
                                    InputType::Boolean { .. } => {
                                        let checked = Signal::new(
                                            match default.as_str() {
                                                "true" => true,
                                                // Anything other than `true` is treated as `false`
                                                // Additionally, we directly normalize it as such (in case the user leaves it as false)
                                                _ => {
                                                    input_value.set("false".to_string());
                                                    false
                                                }
                                            }
                                        );
                                        // Based on that boolean state, we update the global string state
                                        create_effect(cloned!(input_value, checked => move || {
                                            let bool_val = *checked.get();
                                            input_value.set(bool_val.to_string());
                                        }));
                                        view! {
                                            label(class = "switch") {
                                                span() { (label) }
                                                input(type = "checkbox", bind:checked = checked, id = id) {}
                                                span() {}
                                            }
                                            ((*err_label.get()).clone())
                                        }
                                   }
                                    _ => {
                                        let input_type = input_type.to_string();
                                        view! {
                                            label(class = "custom-input") {
                                                input(bind:value = input_value, type = input_type, value = default, id = id, placeholder = "") {}
                                                span() { (label) }
                                            }
                                            ((*err_label.get()).clone())
                                        }
                                    }
                                }
                            },
                            Input::Select { options, can_select_multiple } => {
                                let default_opts: Vec<&str> = default.split(", ").collect();
                                let opts_rendered = View::new_fragment(
                                    options
                                        .iter()
                                        .map(|opt| match opt {
                                            SelectOption::Simple(text) => {
                                                let text = text.to_string();
                                                let value = text.clone();
                                                let is_selected = default_opts.contains(&value.as_str());
                                                view! {
                                                    option(value = value, selected = is_selected) { (text) }
                                                }
                                            },
                                            SelectOption::WithTags { text, .. } => {
                                                let text = text.to_string();
                                                let value = text.clone();
                                                let is_selected = default_opts.contains(&value.as_str());
                                                view! {
                                                    option(value = value, selected = is_selected) { (text) }
                                                }
                                            }
                                        })
                                        .collect()
                                );

                                let multi_select_handler = cloned!(input_value => move |ev: web_sys::Event| {
                                    let el: web_sys::HtmlSelectElement = ev.target().unwrap().unchecked_into();
                                    let selected_opts = el.selected_options();
                                    let selected_opts = js_sys::Array::from(&selected_opts).to_vec(); // An `HtmlCollection` will always be iterable
                                    let values: Vec<String> = selected_opts.iter().map(|opt| opt.clone().unchecked_into::<HtmlOptionElement>().value()).collect();
                                    // We convert the vector into a comma-delimited list, which will be interpolated if necessary
                                    let values_list = values.join(", ");
                                    input_value.set(values_list);
                                });

                                // This is only used for single selects
                                // If we don't have a default set, we should show a placeholder
                                let show_placeholder = Signal::new(default.is_empty());
                                let show_placeholder_class = create_memo(cloned!(show_placeholder => move || {
                                    if *show_placeholder.get() {
                                        "".to_string()
                                    } else {
                                        "no-placeholder".to_string()
                                    }
                                }));

                                match can_select_multiple {
                                    true => view! {
                                        label(class = "custom-input") {
                                            div(class = "select-wrapper select-multiple") {
                                                select(on:input = multi_select_handler, multiple = true) {
                                                    (opts_rendered)
                                                }
                                            }
                                            span() { (label) }
                                        }
                                        ((*err_label.get()).clone())
                                    },
                                    false => view! {
                                        label(class = "custom-input") {
                                            div(class = format!(
                                                "select-wrapper {}",
                                                show_placeholder_class.get()
                                            )) {
                                                // When this is changed in any way, we make sure the placeholder is no longer shown (you can't go back to the empty option)
                                                select(bind:value = input_value, on:change = cloned!(show_placeholder => move |_| {
                                                    show_placeholder.set(false);
                                                })) {
                                                    // If we don't have a blank (for i18n) default option, the user would have to select another option and then reselect whatever the browser makes the default to select it (not good UX!)
                                                    (if default.is_empty() {
                                                        view! {
                                                            option(value = "", selected = true, disabled = true) { "" }
                                                        }
                                                    } else {
                                                        View::empty()
                                                    })
                                                    (opts_rendered)
                                                }
                                            }
                                            span() { (label) }
                                        }
                                        ((*err_label.get()).clone())
                                    }
                                }
                            },
                        };
                        view! {
                            div(class = "w-full text-left") {
                                (input_rendered)
                            }
                        }
                    }
                };
                // We wrap that because there should be space between the elements in a section
                view! {
                    div(class = "my-2") {
                        (rendered)
                    }
                }
            }))
            .collect()
    );

    view! { (elems) }
}

struct RenderReportEndpointProps {
    preamble: String,
    text: String,
}

/// Renders a report endpoint.
#[component(RenderReportEndpoint<G>)]
fn render_report_endpoint(
    RenderReportEndpointProps { preamble, text }: RenderReportEndpointProps,
) -> View<G> {
    let ctx = use_context::<WorkflowCtx>();
    // Flatten the tags into one single vector
    let mut flattened_tags: Vec<String> = Vec::new();
    let history = ctx.history.get();
    for SectionResult { tags, .. } in history.iter() {
        flattened_tags.extend(tags.clone());
    }
    // Join the tags together with commas (the user doesn't need to see these, they'll be parsed by the Tribble bot)
    let tags_str = flattened_tags.join(",");
    // We now encode that internal data with base64
    let encoded_tags = base64::encode(tags_str);

    // Interpolate form values into the text
    // Except in very specific cases, it's faster to do this by simply trying to interpolate all form values
    let mut interpolated_text = text;
    let form_values = ctx.form_values.get();
    for (id, value) in form_values.iter() {
        interpolated_text = interpolated_text.replace(&format!("${{{}}}", id), &value.get());
    }
    // Now collate everything together in one convenient block
    let report_text = format!(
        "{}\n\n{}",
        interpolated_text,
        // We hide the tags away in internal details
        format!(
            "<section>\n<details>Tribble internal data</details>\n\n{}\n\n</section>",
            encoded_tags
        )
    );

    let copy_handler = cloned!(report_text => move |_| {
        wasm_bindgen_futures::spawn_local(cloned!(report_text => async move {
            // Write the text to the clipboard
            let window = web_sys::window().unwrap();
            let clipboard = window.navigator().clipboard().unwrap();
            // We want to copy the tags as well
            let promise = clipboard.write_text(&report_text);
            let fut = wasm_bindgen_futures::JsFuture::from(promise);
            match fut.await {
                Ok(_) => (),
                Err(_) => todo!("handle errors in pages")
            }
        }));
    });

    view! {
        p(class = "mb-2") { (preamble) }
        // The report itself is preformatted
        pre(class = "group overflow-x-auto break-words whitespace-pre-wrap") {
            div(class = "relative") {
                button(
                    on:click = copy_handler,
                    class = "absolute top-0 right-0 mt-1 mr-1 rounded-md invisible group-hover:visible bg-neutral-200 hover:bg-neutral-300 text-black transition-all duration-200 opacity-0 group-hover:opacity-100"
                ) {
                    (svg!(r#"<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 p-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>"#))
                }
            }
            code(id = "tribble-report") {
                (report_text)
            }
        }
    }
}

/// A navigational breadcrumbs element that allows the user to step through their progress.
#[component(HistoryBreadcrumbs<G>)]
fn history_breadcrumbs() -> View<G> {
    let ctx = use_context::<WorkflowCtx>();

    // This state is derived from the history, but it could equally be derived from the history position (notably though, the history is updated after the history position)
    let sections_list = create_memo(cloned!(ctx => move || {
        let history = ctx.history.get();
        let history_len = history.len() - 1; // For borrowing issues

        // We should only display the breadcrumbs if there's more than one element (otherwise it's just a random word)
        if history_len > 0 {
            View::new_fragment(
                history
                    .iter()
                    .enumerate()
                // The index in this corresponds to a position in the `history` vector
                    .map(|(i, SectionResult { name, .. })| {
                        let display_name = if name.starts_with("endpoint:") {
                            name.strip_prefix("endpoint:").unwrap()
                        } else {
                            name
                        }.to_string();
                        let history_pos = *ctx.history_pos.get();
                        let name = name.to_string();
                        let click_handler = cloned!(ctx, name => move |_| {
                            // Update the history position (we might be going forwards, or backwards, either way we preserve the rest)
                            ctx.history_pos.set(i);
                            // Update the location to be this section (again, it can't be an endpoint)
                            ctx.loc.set(name.clone());
                        });

                        // If this is the current item, it shouldn't be a link
                        let item_contents = if i == history_pos {
                            view! {
                                div(class = "p-0.5 xs:p-1 rounded-md") { (display_name) }
                            }
                        } else {
                            view! {
                                button(
                                    on:click = click_handler,
                                    class = "p-0.5 xs:p-1 text-neutral-500 hover:text-black transition-colors duration-200 rounded-md",
                                ) { (display_name) }
                            }
                        };
                        view! {
                            li(class = "mx-1") {
                                (item_contents)
                            }
                            // If this isn't hte last element, add a separator
                            (if i == history_len {
                                View::empty()
                            } else {
                                view! {
                                    div(class = "font-black text-purple-500 h-4 w-4 mt-[0.07rem]") {
                                        (svg!(r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>"#))
                                    }
                                }
                            })
                        }
                    })
                    .collect()
            )
        } else {
            View::empty()
        }
    }));

    view! {
        nav(aria-live = "assertive") {
            // We center vertically so that the text separators stay in line with the button padding
            ol(class = "w-full flex items-center text-sm") {
                // TODO Compress this to the first and last with dots based on screen size
                (*sections_list.get())
            }
        }
    }
}
