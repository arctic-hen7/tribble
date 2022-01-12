use pulldown_cmark::{html, Options, Parser};

/// Parses the given Markdown into an HTML string. This does not perform any santizing of the resulting HTML, so only parse trusted content through here! (This is designed to be used on
/// user-given strings from their Tribble configurations, which they're then serving as their own websites, so security shouldn't be a problem here.)
pub fn parse_md_to_html(markdown: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown, opts);
    let mut html_contents = String::new();
    html::push_html(&mut html_contents, parser);

    html_contents
}
