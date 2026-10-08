//! Attribute lists and the wrapper element.

use std::fmt::Write as _;

use autumn_web::{Markup, PreEscaped};
use maud::Escaper;

/// One HTML attribute: name and value. An empty value writes a bare attribute.
pub type Attr = (&'static str, String);

/// The HTML element that [`Gsap::wrap_in`](crate::Gsap::wrap_in) and
/// [`Timeline::wrap_in`](crate::Timeline::wrap_in) write.
///
/// Put the animation on the semantic element. For example, put SplitText on the
/// heading itself, so its `aria-label` is on the heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Tag {
    /// `<div>` (default).
    #[default]
    Div,
    /// `<section>`.
    Section,
    /// `<article>`.
    Article,
    /// `<header>`.
    Header,
    /// `<footer>`.
    Footer,
    /// `<aside>`.
    Aside,
    /// `<nav>`.
    Nav,
    /// `<main>`.
    Main,
    /// `<figure>`.
    Figure,
    /// `<blockquote>`.
    Blockquote,
    /// `<ul>`.
    Ul,
    /// `<ol>`.
    Ol,
    /// `<li>`.
    Li,
    /// `<p>`. Put only inline content in it. The HTML parser closes a `<p>` before a `<div>`.
    P,
    /// `<span>`.
    Span,
    /// `<h1>`.
    H1,
    /// `<h2>`.
    H2,
    /// `<h3>`.
    H3,
    /// `<h4>`.
    H4,
    /// `<h5>`.
    H5,
    /// `<h6>`.
    H6,
}

impl Tag {
    /// All tags, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::Div,
        Self::Section,
        Self::Article,
        Self::Header,
        Self::Footer,
        Self::Aside,
        Self::Nav,
        Self::Main,
        Self::Figure,
        Self::Blockquote,
        Self::Ul,
        Self::Ol,
        Self::Li,
        Self::P,
        Self::Span,
        Self::H1,
        Self::H2,
        Self::H3,
        Self::H4,
        Self::H5,
        Self::H6,
    ];

    /// The HTML tag name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Section => "section",
            Self::Article => "article",
            Self::Header => "header",
            Self::Footer => "footer",
            Self::Aside => "aside",
            Self::Nav => "nav",
            Self::Main => "main",
            Self::Figure => "figure",
            Self::Blockquote => "blockquote",
            Self::Ul => "ul",
            Self::Ol => "ol",
            Self::Li => "li",
            Self::P => "p",
            Self::Span => "span",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
        }
    }
}

/// Writes `<div …attrs>inner</div>`. Values are HTML-escaped.
#[cfg(test)]
pub(crate) fn wrap_div(attrs: &[Attr], inner: &Markup) -> Markup {
    wrap_el(Tag::Div, attrs, inner)
}

/// Writes `<tag …attrs>inner</tag>`. Values are HTML-escaped.
pub(crate) fn wrap_el(tag: Tag, attrs: &[Attr], inner: &Markup) -> Markup {
    let mut html = format!("<{}", tag.name());
    for (name, value) in attrs {
        html.push(' ');
        html.push_str(name);
        if !value.is_empty() {
            html.push_str("=\"");
            // Writing to a `String` cannot fail.
            let _ = Escaper::new(&mut html).write_str(value);
            html.push('"');
        }
    }
    html.push('>');
    html.push_str(&inner.0);
    html.push_str("</");
    html.push_str(tag.name());
    html.push('>');
    PreEscaped(html)
}

/// Adds a bare attribute when `on` is `true`.
pub(crate) fn push_flag(out: &mut Vec<Attr>, name: &'static str, on: bool) {
    if on {
        out.push((name, String::new()));
    }
}

/// Adds `name="value"` when `value` is `Some`.
pub(crate) fn push_opt(out: &mut Vec<Attr>, name: &'static str, value: Option<String>) {
    if let Some(value) = value {
        out.push((name, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autumn_web::html;

    #[test]
    fn wrap_div_writes_attributes_in_order() {
        let attrs = vec![
            ("data-gsap", "fade-up".to_owned()),
            ("data-gsap-pin", String::new()),
            ("data-gsap-delay", "0.2".to_owned()),
        ];
        let html = wrap_div(&attrs, &html! { p { "hi" } }).into_string();
        assert_eq!(
            html,
            r#"<div data-gsap="fade-up" data-gsap-pin data-gsap-delay="0.2"><p>hi</p></div>"#
        );
    }

    #[test]
    fn wrap_div_escapes_values() {
        let attrs = vec![("data-gsap-trigger", r#"a[href="x"]&<b>"#.to_owned())];
        let html = wrap_div(&attrs, &html! {}).into_string();
        assert_eq!(
            html,
            r#"<div data-gsap-trigger="a[href=&quot;x&quot;]&amp;&lt;b&gt;"></div>"#
        );
    }

    #[test]
    fn wrap_el_uses_the_tag_name() {
        let attrs = vec![
            ("class", "card".to_owned()),
            ("data-gsap", "fade".to_owned()),
        ];
        let html = wrap_el(Tag::H2, &attrs, &html! { "Title" }).into_string();
        assert_eq!(html, r#"<h2 class="card" data-gsap="fade">Title</h2>"#);
    }

    #[test]
    fn tag_names_are_lowercase_html() {
        for tag in Tag::ALL {
            let name = tag.name();
            assert!(
                name.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()),
                "{name}"
            );
        }
        assert_eq!(Tag::default(), Tag::Div);
        assert_eq!(Tag::Blockquote.name(), "blockquote");
    }

    #[test]
    fn push_helpers_skip_absent_values() {
        let mut out = Vec::new();
        push_flag(&mut out, "data-gsap-pin", false);
        push_opt(&mut out, "data-gsap-end", None);
        assert_eq!(out, Vec::<Attr>::new());
        push_flag(&mut out, "data-gsap-pin", true);
        push_opt(&mut out, "data-gsap-end", Some("bottom top".to_owned()));
        assert_eq!(
            out,
            vec![
                ("data-gsap-pin", String::new()),
                ("data-gsap-end", "bottom top".to_owned()),
            ]
        );
    }
}
