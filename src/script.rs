//! [`gsap_script`] and [`gsap_stylesheet`]: the tags for the page `<head>`.

use autumn_web::{Markup, html};

use crate::assets::{GSAP_ASSETS, GSAP_CSS, GSAP_JS, INIT_JS, SCROLL_TRIGGER_JS, SPLIT_TEXT_JS};

/// Writes the deferred `<script>` tags: GSAP, ScrollTrigger, SplitText, then `init.js`.
///
/// Each tag has the hashed URL and the SRI hash. Deferred scripts run in document order.
///
/// ```rust
/// let head = autumn_plugin_gsap::gsap_script().into_string();
/// assert!(head.contains("/static/_plugins/gsap/gsap.min."));
/// ```
#[must_use]
pub fn gsap_script() -> Markup {
    html! {
        (GSAP_ASSETS.deferred_script_tag(GSAP_JS))
        (GSAP_ASSETS.deferred_script_tag(SCROLL_TRIGGER_JS))
        (GSAP_ASSETS.deferred_script_tag(SPLIT_TEXT_JS))
        (GSAP_ASSETS.deferred_script_tag(INIT_JS))
    }
}

/// Writes the `<link>` tag for `gsap.css`.
///
/// You need it for [`Gsap::scroll_progress`](crate::Gsap::scroll_progress).
///
/// ```rust
/// let link = autumn_plugin_gsap::gsap_stylesheet().into_string();
/// assert!(link.contains(r#"href="/static/_plugins/gsap/gsap."#));
/// ```
#[must_use]
pub fn gsap_stylesheet() -> Markup {
    GSAP_ASSETS.stylesheet_tag(GSAP_CSS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_tags_load_in_order_with_sri() {
        let html = gsap_script().into_string();
        let mut last = 0;
        for path in [GSAP_JS, SCROLL_TRIGGER_JS, SPLIT_TEXT_JS, INIT_JS] {
            let asset = GSAP_ASSETS.get(path).expect("bundled");
            let at = html
                .find(&format!(r#"src="{}""#, asset.url()))
                .unwrap_or_else(|| panic!("{path} tag in {html}"));
            assert!(at >= last, "{path} loads after the previous file: {html}");
            last = at;
            assert!(
                html.contains(&format!(r#"integrity="{}""#, asset.integrity())),
                "{path}: {html}"
            );
        }
        assert_eq!(html.matches("<script").count(), 4, "{html}");
        assert_eq!(html.matches(" defer").count(), 4, "{html}");
        assert_eq!(
            html.matches(r#"crossorigin="anonymous""#).count(),
            4,
            "{html}"
        );
    }

    #[test]
    fn stylesheet_link_has_sri() {
        let html = gsap_stylesheet().into_string();
        let css = GSAP_ASSETS.get(GSAP_CSS).expect("bundled");
        assert!(html.contains(r#"rel="stylesheet""#), "{html}");
        assert!(html.contains(&format!(r#"href="{}""#, css.url())), "{html}");
        assert!(
            html.contains(&format!(r#"integrity="{}""#, css.integrity())),
            "{html}"
        );
    }
}
