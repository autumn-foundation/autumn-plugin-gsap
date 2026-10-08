//! Typed ScrollTrigger values.

use std::fmt;
use std::time::Duration;

use crate::fmt::{num, secs};

/// When an animation starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Play {
    /// When the trigger scrolls into view (default).
    #[default]
    Scroll,
    /// When the script scans the element (page load or htmx swap).
    Load,
}

/// One edge in a [`ScrollPos`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Edge {
    /// The top edge.
    Top,
    /// The middle.
    Center,
    /// The bottom edge.
    Bottom,
    /// A percent of the height from the top. A value that is not finite writes `0%`.
    Percent(f32),
    /// Pixels from the top. A value that is not finite writes `0px`.
    Px(f32),
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Top => f.write_str("top"),
            Self::Center => f.write_str("center"),
            Self::Bottom => f.write_str("bottom"),
            Self::Percent(v) => write!(f, "{}%", num(v).as_deref().unwrap_or("0")),
            Self::Px(v) => write!(f, "{}px", num(v).as_deref().unwrap_or("0")),
        }
    }
}

/// A ScrollTrigger `start` or `end` value.
///
/// ```rust
/// use autumn_plugin_gsap::{Edge, ScrollPos};
///
/// assert_eq!(ScrollPos::new(Edge::Top, Edge::Percent(80.0)).to_string(), "top 80%");
/// assert_eq!(ScrollPos::Distance(500.0).to_string(), "+=500");
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollPos {
    /// The trigger edge meets the viewport edge.
    Edges {
        /// The edge of the trigger element.
        trigger: Edge,
        /// The edge of the viewport.
        viewport: Edge,
    },
    /// Pixels of scroll after `start` (only for `end`).
    /// A value that is not finite or below zero writes `+=0`.
    Distance(f32),
}

impl ScrollPos {
    /// The trigger edge meets the viewport edge.
    #[must_use]
    pub const fn new(trigger: Edge, viewport: Edge) -> Self {
        Self::Edges { trigger, viewport }
    }
}

impl fmt::Display for ScrollPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Edges { trigger, viewport } => write!(f, "{trigger} {viewport}"),
            Self::Distance(px) => {
                let px = num(px.max(0.0)).unwrap_or_else(|| "0".to_owned());
                write!(f, "+={px}")
            }
        }
    }
}

/// One ScrollTrigger toggle action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Play forward.
    Play,
    /// Pause.
    Pause,
    /// Resume.
    Resume,
    /// Play backward.
    Reverse,
    /// Play again from the start.
    Restart,
    /// Go to the start and stop.
    Reset,
    /// Go to the end.
    Complete,
    /// Do nothing.
    None,
}

impl Action {
    /// The GSAP name.
    const fn gsap(self) -> &'static str {
        match self {
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Reverse => "reverse",
            Self::Restart => "restart",
            Self::Reset => "reset",
            Self::Complete => "complete",
            Self::None => "none",
        }
    }
}

/// ScrollTrigger `toggleActions`: what to do on enter, leave, enter back and leave back.
///
/// ```rust
/// use autumn_plugin_gsap::{Action, ToggleActions};
///
/// let t = ToggleActions::new(Action::Play, Action::None, Action::None, Action::Reverse);
/// assert_eq!(t.to_string(), "play none none reverse");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToggleActions {
    /// The trigger scrolls in, forward.
    pub on_enter: Action,
    /// The trigger scrolls out, forward.
    pub on_leave: Action,
    /// The trigger scrolls in, backward.
    pub on_enter_back: Action,
    /// The trigger scrolls out, backward.
    pub on_leave_back: Action,
}

impl ToggleActions {
    /// Makes the four actions.
    #[must_use]
    pub const fn new(
        on_enter: Action,
        on_leave: Action,
        on_enter_back: Action,
        on_leave_back: Action,
    ) -> Self {
        Self {
            on_enter,
            on_leave,
            on_enter_back,
            on_leave_back,
        }
    }
}

impl fmt::Display for ToggleActions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.on_enter.gsap(),
            self.on_leave.gsap(),
            self.on_enter_back.gsap(),
            self.on_leave_back.gsap()
        )
    }
}

/// ScrollTrigger `scrub`: link the animation progress to the scroll position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scrub {
    /// Follow the scroll position directly (`true`).
    Linked,
    /// Catch up with the scroll position in this time.
    Smooth(Duration),
}

impl fmt::Display for Scrub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Linked => f.write_str("true"),
            Self::Smooth(d) => f.write_str(&secs(d)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{is_scroll_pos, is_secs};
    use proptest::prelude::*;

    #[test]
    fn edges_write_gsap_words() {
        assert_eq!(Edge::Top.to_string(), "top");
        assert_eq!(Edge::Center.to_string(), "center");
        assert_eq!(Edge::Bottom.to_string(), "bottom");
        assert_eq!(Edge::Percent(85.0).to_string(), "85%");
        assert_eq!(Edge::Px(-120.5).to_string(), "-120.5px");
        assert_eq!(Edge::Percent(f32::NAN).to_string(), "0%");
        assert_eq!(Edge::Px(f32::INFINITY).to_string(), "0px");
    }

    #[test]
    fn scroll_pos_writes_two_edges_or_a_distance() {
        assert_eq!(
            ScrollPos::new(Edge::Top, Edge::Percent(85.0)).to_string(),
            "top 85%"
        );
        assert_eq!(
            ScrollPos::new(Edge::Bottom, Edge::Top).to_string(),
            "bottom top"
        );
        assert_eq!(ScrollPos::Distance(500.0).to_string(), "+=500");
        assert_eq!(ScrollPos::Distance(-5.0).to_string(), "+=0");
        assert_eq!(ScrollPos::Distance(f32::NAN).to_string(), "+=0");
    }

    #[test]
    fn toggle_actions_write_four_words() {
        let all = ToggleActions::new(Action::Play, Action::Pause, Action::Resume, Action::Reverse);
        assert_eq!(all.to_string(), "play pause resume reverse");
        let rest = ToggleActions::new(
            Action::Restart,
            Action::Reset,
            Action::Complete,
            Action::None,
        );
        assert_eq!(rest.to_string(), "restart reset complete none");
    }

    #[test]
    fn scrub_writes_true_or_seconds() {
        assert_eq!(Scrub::Linked.to_string(), "true");
        assert_eq!(Scrub::Smooth(Duration::from_millis(500)).to_string(), "0.5");
        assert!(is_secs(&Scrub::Smooth(Duration::from_secs(1)).to_string()));
    }

    #[test]
    fn play_defaults_to_scroll() {
        assert_eq!(Play::default(), Play::Scroll);
    }

    fn edge() -> impl Strategy<Value = Edge> {
        let f = proptest::num::f32::ANY;
        prop_oneof![
            Just(Edge::Top),
            Just(Edge::Center),
            Just(Edge::Bottom),
            f.prop_map(Edge::Percent),
            f.prop_map(Edge::Px),
        ]
    }

    proptest! {
        #[test]
        fn every_scroll_pos_matches_the_js_grammar(
            a in edge(),
            b in edge(),
            d in proptest::num::f32::ANY,
        ) {
            let edges = ScrollPos::new(a, b).to_string();
            prop_assert!(is_scroll_pos(&edges), "{edges}");
            let dist = ScrollPos::Distance(d).to_string();
            prop_assert!(is_scroll_pos(&dist), "{dist}");
        }
    }
}
