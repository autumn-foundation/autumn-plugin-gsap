//! Options that [`Gsap`](crate::Gsap) and [`Timeline`](crate::Timeline) share.

use std::time::Duration;

use crate::attrs::{Attr, push_flag, push_opt};
use crate::ease::Ease;
use crate::fmt::secs;
use crate::scroll::{Play, ScrollPos, Scrub, ToggleActions};

/// The play time that `init.js` uses when you set no duration.
#[cfg(test)]
pub(crate) const DEFAULT_DURATION: Duration = Duration::from_millis(800);

/// How many times an animation plays again after the first play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Repeat {
    /// Play again `n` more times.
    Times(u32),
    /// Play again with no end (`-1`).
    Forever,
}

/// Shared playback and ScrollTrigger options.
///
/// Each `bool` maps to one bare attribute, so the flags stay separate fields.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Common {
    pub(crate) play: Play,
    pub(crate) delay: Option<Duration>,
    pub(crate) duration: Option<Duration>,
    pub(crate) ease: Option<Ease>,
    pub(crate) repeat: Option<Repeat>,
    pub(crate) yoyo: bool,
    pub(crate) repeat_delay: Option<Duration>,
    pub(crate) trigger: Option<String>,
    pub(crate) start: Option<ScrollPos>,
    pub(crate) end: Option<ScrollPos>,
    pub(crate) toggle_actions: Option<ToggleActions>,
    pub(crate) replay: bool,
    pub(crate) scrub: Option<Scrub>,
    pub(crate) pin: bool,
    pub(crate) markers: bool,
    pub(crate) animate_reduced: bool,
    pub(crate) id: Option<String>,
    pub(crate) class: Option<String>,
}

impl Common {
    /// The `id` and `class` attributes for the wrapper element.
    pub(crate) fn html_attrs(&self) -> Vec<Attr> {
        let mut out = Vec::new();
        push_opt(&mut out, "id", self.id.clone());
        push_opt(&mut out, "class", self.class.clone());
        out
    }

    /// Adds the attributes for the options that differ from the defaults.
    pub(crate) fn push(&self, out: &mut Vec<Attr>) {
        push_opt(
            out,
            "data-gsap-on",
            (self.play == Play::Load).then(|| "load".to_owned()),
        );
        push_opt(out, "data-gsap-delay", self.delay.map(secs));
        push_opt(out, "data-gsap-duration", self.duration.map(secs));
        push_opt(out, "data-gsap-ease", self.ease.map(|e| e.to_string()));
        push_opt(
            out,
            "data-gsap-repeat",
            self.repeat.map(|r| match r {
                Repeat::Times(n) => n.to_string(),
                Repeat::Forever => "-1".to_owned(),
            }),
        );
        push_flag(out, "data-gsap-yoyo", self.yoyo);
        push_opt(out, "data-gsap-repeat-delay", self.repeat_delay.map(secs));
        push_opt(out, "data-gsap-trigger", self.trigger.clone());
        push_opt(out, "data-gsap-start", self.start.map(|p| p.to_string()));
        push_opt(out, "data-gsap-end", self.end.map(|p| p.to_string()));
        push_opt(
            out,
            "data-gsap-toggle-actions",
            self.toggle_actions.map(|t| t.to_string()),
        );
        push_opt(
            out,
            "data-gsap-once",
            self.replay.then(|| "false".to_owned()),
        );
        push_opt(out, "data-gsap-scrub", self.scrub.map(|s| s.to_string()));
        push_flag(out, "data-gsap-pin", self.pin);
        push_flag(out, "data-gsap-markers", self.markers);
        push_opt(
            out,
            "data-gsap-reduced",
            self.animate_reduced.then(|| "animate".to_owned()),
        );
    }
}

/// Adds the shared builder methods to a type with a `common: Common` field.
macro_rules! common_setters {
    () => {
        /// Starts on scroll (default) or on load. See [`Play`](crate::Play).
        /// With [`Play::Load`](crate::Play::Load), `init.js` ignores the ScrollTrigger options.
        pub const fn play(mut self, play: crate::Play) -> Self {
            self.common.play = play;
            self
        }

        /// Waits this time before the start (`data-gsap-delay`).
        pub const fn delay(mut self, delay: std::time::Duration) -> Self {
            self.common.delay = Some(delay);
            self
        }

        /// Sets the play time (`data-gsap-duration`). The default is 0.8 s.
        /// On a [`Timeline`](crate::Timeline), this is the default for each step.
        /// A step with no duration uses the timeline value. Times have millisecond precision.
        pub const fn duration(mut self, duration: std::time::Duration) -> Self {
            self.common.duration = Some(duration);
            self
        }

        /// Sets the ease (`data-gsap-ease`). The default is `power3.out`.
        /// On a [`Timeline`](crate::Timeline), this is the default for each step.
        pub const fn ease(mut self, ease: crate::Ease) -> Self {
            self.common.ease = Some(ease);
            self
        }

        /// Plays again after the first play (`data-gsap-repeat`).
        pub const fn repeat(mut self, repeat: crate::Repeat) -> Self {
            self.common.repeat = Some(repeat);
            self
        }

        /// Plays backward on each second repeat (`data-gsap-yoyo`).
        pub const fn yoyo(mut self) -> Self {
            self.common.yoyo = true;
            self
        }

        /// Waits this time between repeats (`data-gsap-repeat-delay`).
        pub const fn repeat_delay(mut self, delay: std::time::Duration) -> Self {
            self.common.repeat_delay = Some(delay);
            self
        }

        /// Uses another element as the ScrollTrigger trigger (`data-gsap-trigger`).
        /// The value is a CSS selector. An unknown selector uses the element itself.
        /// A blank string sets no trigger.
        pub fn trigger(mut self, selector: impl Into<String>) -> Self {
            self.common.trigger = Some(selector.into()).filter(|s| !s.trim().is_empty());
            self
        }

        /// Sets the ScrollTrigger start (`data-gsap-start`). The default is `top 85%`.
        pub const fn start(mut self, pos: crate::ScrollPos) -> Self {
            self.common.start = Some(pos);
            self
        }

        /// Sets the ScrollTrigger end (`data-gsap-end`).
        pub const fn end(mut self, pos: crate::ScrollPos) -> Self {
            self.common.end = Some(pos);
            self
        }

        /// Sets the ScrollTrigger toggle actions (`data-gsap-toggle-actions`).
        pub const fn toggle_actions(mut self, actions: crate::ToggleActions) -> Self {
            self.common.toggle_actions = Some(actions);
            self
        }

        /// Keeps the ScrollTrigger after the first play (`data-gsap-once="false"`).
        /// The animation plays forward on enter and backward on leave back
        /// (`play none none reverse`). The default plays one time.
        /// [`toggle_actions`](Self::toggle_actions) and [`scrub`](Self::scrub) replace this option.
        pub const fn replay(mut self) -> Self {
            self.common.replay = true;
            self
        }

        /// Links the progress to the scroll position (`data-gsap-scrub`).
        /// With scrub, the default range is `top bottom` to `bottom top`.
        /// Scrub ignores [`toggle_actions`](Self::toggle_actions) and [`replay`](Self::replay).
        pub const fn scrub(mut self, scrub: crate::Scrub) -> Self {
            self.common.scrub = Some(scrub);
            self
        }

        /// Pins the trigger element while the animation is active (`data-gsap-pin`).
        /// `init.js` refuses a pin when the trigger is outside the element.
        pub const fn pin(mut self) -> Self {
            self.common.pin = true;
            self
        }

        /// Shows the ScrollTrigger debug markers (`data-gsap-markers`).
        pub const fn markers(mut self) -> Self {
            self.common.markers = true;
            self
        }

        /// Sets the `id` of the wrapper element. A blank string sets no `id`.
        pub fn id(mut self, id: impl Into<String>) -> Self {
            self.common.id = Some(id.into()).filter(|s| !s.trim().is_empty());
            self
        }

        /// Sets the `class` of the wrapper element. A blank string sets no `class`.
        pub fn class(mut self, class: impl Into<String>) -> Self {
            self.common.class = Some(class.into()).filter(|s| !s.trim().is_empty());
            self
        }

        /// Puts `markup` in a `<div>` with the attributes.
        #[allow(clippy::needless_pass_by_value)]
        #[must_use]
        pub fn wrap(self, markup: autumn_web::Markup) -> autumn_web::Markup {
            self.wrap_in(crate::Tag::Div, markup)
        }

        /// Puts `markup` in a `tag` element with the `id`, `class` and animation attributes.
        #[allow(clippy::needless_pass_by_value)]
        #[must_use]
        pub fn wrap_in(self, tag: crate::Tag, markup: autumn_web::Markup) -> autumn_web::Markup {
            let mut attrs = self.common.html_attrs();
            attrs.extend(self.attributes());
            crate::attrs::wrap_el(tag, &attrs, &markup)
        }

        /// Also animates when the user prefers reduced motion
        /// (`data-gsap-reduced="animate"`). Use this only for motion that is necessary.
        pub const fn animate_on_reduced_motion(mut self) -> Self {
            self.common.animate_reduced = true;
            self
        }
    };
}

pub(crate) use common_setters;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_js_uses_the_same_default_duration() {
        let js = include_str!("../assets/init.js");
        let want = format!("duration: {},", secs(DEFAULT_DURATION));
        assert!(js.contains(&want), "init.js DEFAULTS must contain `{want}`");
    }

    #[test]
    fn documented_defaults_match_init_js() {
        let js = include_str!("../assets/init.js");
        for want in [r#"ease: "power3.out","#, r#"start: "top 85%","#] {
            assert!(js.contains(want), "init.js DEFAULTS must contain `{want}`");
        }
    }

    #[test]
    fn html_attrs_hold_only_id_and_class() {
        let c = Common {
            id: Some("a".to_owned()),
            class: Some("b".to_owned()),
            pin: true,
            ..Common::default()
        };
        assert_eq!(
            c.html_attrs(),
            vec![("id", "a".to_owned()), ("class", "b".to_owned())]
        );
    }
}
