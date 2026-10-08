//! The typed [`Timeline`] builder.

use std::fmt;
use std::time::Duration;

use crate::attrs::Attr;
use crate::fmt::secs;
use crate::options::{Common, common_setters};

/// The start time of a tween in a [`Timeline`] (the GSAP position parameter).
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_gsap::Position;
///
/// assert_eq!(Position::WithPrevious.to_string(), "<");
/// assert_eq!(Position::Overlap(Duration::from_millis(200)).to_string(), "-=0.2");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Position {
    /// After the most recent step ends (`>`).
    /// With no position, a step starts at the end of the timeline.
    /// The two differ when the most recent step ends before the end of the timeline,
    /// for example a short step at `<`.
    After,
    /// When the most recent step starts (`<`).
    WithPrevious,
    /// This time after the most recent step starts (`<0.2`).
    AfterPreviousStart(Duration),
    /// This time after the end of the timeline (`+=0.2`).
    Gap(Duration),
    /// This time before the end of the timeline (`-=0.2`).
    Overlap(Duration),
    /// At this time from the timeline start (`1.5`).
    At(Duration),
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::After => f.write_str(">"),
            Self::WithPrevious => f.write_str("<"),
            Self::AfterPreviousStart(d) => write!(f, "<{}", secs(d)),
            Self::Gap(d) => write!(f, "+={}", secs(d)),
            Self::Overlap(d) => write!(f, "-={}", secs(d)),
            Self::At(d) => f.write_str(&secs(d)),
        }
    }
}

/// A GSAP timeline. It plays the `data-gsap` children in document order, as one sequence.
///
/// The timeline options (`play`, `scrub`, `pin`, `repeat`, ...) apply to the whole sequence.
/// The `duration` and `ease` options are defaults for the children.
/// On a child, `init.js` reads only the timing, stagger, split and position options.
/// It ignores the ScrollTrigger options and `animate_on_reduced_motion` of a child.
/// [`Parallax`](crate::Parallax) and the progress bar cannot be timeline steps.
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_gsap::{Gsap, Position, Timeline};
/// use autumn_web::html;
///
/// let hero = Timeline::new().wrap(html! {
///     (Gsap::fade_up().wrap(html! { h1 { "Title" } }))
///     (Gsap::fade().position(Position::Overlap(Duration::from_millis(300)))
///         .wrap(html! { p { "Text" } }))
/// });
/// assert!(hero.into_string().contains("data-gsap-timeline"));
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
#[must_use]
pub struct Timeline {
    common: Common,
}

impl Timeline {
    /// Makes a timeline with default options.
    pub fn new() -> Self {
        Self::default()
    }

    common_setters!();

    /// The `data-gsap-timeline` attribute and the option attributes, in a fixed order.
    #[must_use]
    pub fn attributes(&self) -> Vec<Attr> {
        let mut out = vec![("data-gsap-timeline", String::new())];
        self.common.push(&mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::is_position;
    use crate::scroll::{Edge, Play, ScrollPos, Scrub};
    use crate::{Ease, EaseDir, Repeat};
    use autumn_web::html;
    use proptest::prelude::*;

    #[test]
    fn positions_write_gsap_values() {
        let ms = Duration::from_millis;
        assert_eq!(Position::After.to_string(), ">");
        assert_eq!(Position::WithPrevious.to_string(), "<");
        assert_eq!(Position::AfterPreviousStart(ms(250)).to_string(), "<0.25");
        assert_eq!(Position::Gap(ms(500)).to_string(), "+=0.5");
        assert_eq!(Position::Overlap(ms(200)).to_string(), "-=0.2");
        assert_eq!(Position::At(ms(1500)).to_string(), "1.5");
    }

    #[test]
    fn default_timeline_writes_only_the_marker() {
        let t = Timeline::new();
        assert_eq!(t.attributes(), vec![("data-gsap-timeline", String::new())]);
    }

    #[test]
    fn options_write_attributes() {
        let t = Timeline::new()
            .play(Play::Load)
            .duration(Duration::from_millis(600))
            .ease(Ease::Sine(EaseDir::InOut))
            .repeat(Repeat::Forever)
            .yoyo()
            .start(ScrollPos::new(Edge::Top, Edge::Top))
            .end(ScrollPos::Distance(1200.0))
            .scrub(Scrub::Linked)
            .pin();
        let a = t.attributes();
        assert_eq!(a[0], ("data-gsap-timeline", String::new()));
        for want in [
            ("data-gsap-on", "load"),
            ("data-gsap-duration", "0.6"),
            ("data-gsap-ease", "sine.inOut"),
            ("data-gsap-repeat", "-1"),
            ("data-gsap-yoyo", ""),
            ("data-gsap-start", "top top"),
            ("data-gsap-end", "+=1200"),
            ("data-gsap-scrub", "true"),
            ("data-gsap-pin", ""),
        ] {
            assert!(
                a.iter().any(|(k, v)| *k == want.0 && v == want.1),
                "{want:?} in {a:?}"
            );
        }
    }

    #[test]
    fn wrap_in_writes_tag_and_class() {
        let html = Timeline::new()
            .class("hero")
            .wrap_in(crate::Tag::Section, html! {})
            .into_string();
        assert_eq!(
            html,
            r#"<section class="hero" data-gsap-timeline></section>"#
        );
    }

    #[test]
    fn wrap_keeps_children() {
        let html = Timeline::new()
            .wrap(html! { p data-gsap="fade" { "x" } })
            .into_string();
        assert_eq!(
            html,
            r#"<div data-gsap-timeline><p data-gsap="fade">x</p></div>"#
        );
    }

    fn position() -> impl Strategy<Value = Position> {
        let d = (0u64..100_000).prop_map(Duration::from_millis);
        prop_oneof![
            Just(Position::After),
            Just(Position::WithPrevious),
            d.clone().prop_map(Position::AfterPreviousStart),
            d.clone().prop_map(Position::Gap),
            d.clone().prop_map(Position::Overlap),
            d.prop_map(Position::At),
        ]
    }

    proptest! {
        #[test]
        fn every_position_matches_the_js_grammar(p in position()) {
            let s = p.to_string();
            prop_assert!(is_position(&s), "{s}");
        }
    }
}
