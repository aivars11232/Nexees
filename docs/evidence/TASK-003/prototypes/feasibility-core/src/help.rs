//! Passive offline help: Markdown to HTML with active content removed while
//! parsing, so no sanitizer has to undo it afterwards.
//!
//! Raw HTML is shown as literal text. A link survives only if it is relative
//! or `https:`; an image only if it is relative (a bundled file), so rendering
//! never fetches anything. Dropped links and images keep their text. The
//! viewer that shows this output runs no script either.

use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};

pub fn render(markdown: &str) -> String {
    // One entry per open link or image: was its start tag dropped?
    let mut dropped = Vec::new();
    let events = Parser::new_ext(markdown, Options::ENABLE_TABLES).filter_map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Some(Event::Text(raw)),
        Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
            let keep = scheme(&dest_url).is_none_or(|s| s == "https");
            dropped.push(!keep);
            keep.then_some(Event::Start(Tag::Link { link_type, dest_url, title, id }))
        }
        Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
            let keep = scheme(&dest_url).is_none();
            dropped.push(!keep);
            keep.then_some(Event::Start(Tag::Image { link_type, dest_url, title, id }))
        }
        end @ Event::End(TagEnd::Link | TagEnd::Image) => {
            (!dropped.pop().unwrap_or(true)).then_some(end)
        }
        other => Some(other),
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

/// The URL's scheme in lowercase, judged the way a browser would after it
/// strips whitespace and control characters. A protocol-relative `//host`
/// URL counts as a scheme so it is never treated as a bundled file.
pub(crate) fn scheme(url: &str) -> Option<String> {
    let url: String = url.chars().filter(|c| !c.is_ascii_whitespace() && !c.is_control()).collect();
    if url.starts_with("//") {
        return Some(String::new());
    }
    let end = url.find([':', '/', '?', '#'])?;
    url[end..].starts_with(':').then(|| url[..end].to_ascii_lowercase())
}
