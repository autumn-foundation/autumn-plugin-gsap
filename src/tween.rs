//! The typed [`Gsap`] tween builder.

use std::time::Duration;

use autumn_web::{Markup, html};

use crate::attrs::{Attr, push_flag, push_opt};
use crate::ease::Ease;
use crate::fmt::{num, secs};
use crate::options::{Common, common_setters};
use crate::props::Props;
use crate::timeline::Position;

/// A preset start state. The element animates from this state to its CSS state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Preset {
    /// Opacity 0.
    Fade,
    /// Opacity 0, 32px down.
    FadeUp,
    /// Opacity 0, 32px up.
    FadeDown,
    /// Opacity 0, 32px right. The element moves left.
    FadeLeft,
    /// Opacity 0, 32px left. The element moves right.
    FadeRight,
    /// Opacity 0, scale 0.92.
    Scale,
    /// Opacity 0, scale 0.6.
    ZoomIn,
    /// Opacity 0, scale 1.3.
    ZoomOut,
    /// Opacity 0, 100% of its height down.
    SlideUp,
    /// Opacity 0, 100% of its height up.
    SlideDown,
    /// Opacity 0, 100% of its width right.
    SlideLeft,
    /// Opacity 0, 100% of its width left.
    SlideRight,
    /// Opacity 0, rotation -12 degrees.
    RotateIn,
    /// Opacity 0, blur 12px.
    BlurIn,
    /// Opacity 0, 3D rotation -90 degrees around the x axis.
    FlipX,
}

impl Preset {
    /// The `data-gsap` value.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fade => "fade",
            Self::FadeUp => "fade-up",
            Self::FadeDown => "fade-down",
            Self::FadeLeft => "fade-left",
            Self::FadeRight => "fade-right",
            Self::Scale => "scale",
            Self::ZoomIn => "zoom-in",
            Self::ZoomOut => "zoom-out",
            Self::SlideUp => "slide-up",
            Self::SlideDown => "slide-down",
            Self::SlideLeft => "slide-left",
            Self::SlideRight => "slide-right",
            Self::RotateIn => "rotate-in",
            Self::BlurIn => "blur-in",
            Self::FlipX => "flip-x",
        }
    }

    /// All presets, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::Fade,
        Self::FadeUp,
        Self::FadeDown,
        Self::FadeLeft,
        Self::FadeRight,
        Self::Scale,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::SlideUp,
        Self::SlideDown,
        Self::SlideLeft,
        Self::SlideRight,
        Self::RotateIn,
        Self::BlurIn,
        Self::FlipX,
    ];
}

/// Where a stagger starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StaggerFrom {
    /// The first item (default).
    Start,
    /// The middle item.
    Center,
    /// The last item.
    End,
    /// The first and last items.
    Edges,
    /// A random order.
    Random,
    /// The item at this index.
    Index(u32),
}

/// The text unit that SplitText animates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Split {
    /// Each character.
    Chars,
    /// Each word.
    Words,
    /// Each line.
    Lines,
}

/// One declarative GSAP tween: a preset (or [`Props`]) and options.
///
/// Make one with a preset function, set options, then call [`Gsap::wrap`].
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_gsap::{Ease, EaseDir, Gsap};
/// use autumn_web::html;
///
/// let card = Gsap::fade_up()
///     .delay(Duration::from_millis(150))
///     .ease(Ease::Back(EaseDir::Out, 1.7))
///     .wrap(html! { h2 { "Hello" } })
///     .into_string();
/// assert!(card.contains(r#"data-gsap="fade-up""#));
/// assert!(card.contains(r#"data-gsap-delay="0.15""#));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct Gsap {
    /// The preset, or `None` for custom [`Props`].
    preset: Option<Preset>,
    from: Option<Props>,
    to: Option<Props>,
    stagger: Option<Duration>,
    stagger_from: Option<StaggerFrom>,
    stagger_ease: Option<Ease>,
    split: Option<Split>,
    split_mask: bool,
    position: Option<Position>,
    common: Common,
}

macro_rules! presets {
    ($($(#[$doc:meta])* $fn_name:ident => $variant:ident;)*) => {
        $(
            $(#[$doc])*
            pub fn $fn_name() -> Self {
                Self::preset(Preset::$variant)
            }
        )*
    };
}

impl Gsap {
    /// Makes a tween for `preset` with default options.
    pub fn preset(preset: Preset) -> Self {
        Self {
            preset: Some(preset),
            from: None,
            to: None,
            stagger: None,
            stagger_from: None,
            stagger_ease: None,
            split: None,
            split_mask: false,
            position: None,
            common: Common::default(),
        }
    }

    presets! {
        /// Fades in.
        fade => Fade;
        /// Fades in and moves up 32px.
        fade_up => FadeUp;
        /// Fades in and moves down 32px.
        fade_down => FadeDown;
        /// Fades in and moves left 32px.
        fade_left => FadeLeft;
        /// Fades in and moves right 32px.
        fade_right => FadeRight;
        /// Fades in and grows from scale 0.92.
        scale => Scale;
        /// Fades in and grows from scale 0.6.
        zoom_in => ZoomIn;
        /// Fades in and shrinks from scale 1.3.
        zoom_out => ZoomOut;
        /// Fades in and moves up its full height.
        slide_up => SlideUp;
        /// Fades in and moves down its full height.
        slide_down => SlideDown;
        /// Fades in and moves left its full width.
        slide_left => SlideLeft;
        /// Fades in and moves right its full width.
        slide_right => SlideRight;
        /// Fades in and turns from -12 degrees.
        rotate_in => RotateIn;
        /// Fades in and removes a 12px blur.
        blur_in => BlurIn;
        /// Fades in and turns from -90 degrees around the x axis.
        flip_x => FlipX;
    }

    /// Makes a custom tween (`data-gsap="custom"`). Empty props write no JSON.
    fn custom(from: Option<Props>, to: Option<Props>) -> Self {
        let mut g = Self::preset(Preset::Fade);
        g.preset = None;
        g.from = from.filter(|p| !p.is_empty());
        g.to = to.filter(|p| !p.is_empty());
        g
    }

    /// Animates from `props` to the CSS state (`gsap.from`).
    /// Empty props write no JSON. With no props, `init.js` does not animate the element.
    pub fn from_props(props: Props) -> Self {
        Self::custom(Some(props), None)
    }

    /// Animates from the CSS state to `props` (`gsap.to`).
    /// Empty props write no JSON. With no props, `init.js` does not animate the element.
    pub fn to_props(props: Props) -> Self {
        Self::custom(None, Some(props))
    }

    /// Animates from `from` to `to` (`gsap.fromTo`).
    /// Empty props write no JSON. With no props, `init.js` does not animate the element.
    pub fn from_to(from: Props, to: Props) -> Self {
        Self::custom(Some(from), Some(to))
    }

    /// Scroll-linked vertical drift. See [`Parallax`].
    pub const fn parallax(factor: f32) -> Parallax {
        Parallax {
            factor,
            id: None,
            class: None,
            animate_reduced: false,
        }
    }

    /// A fixed bar at the top of the page. It fills while the page scrolls.
    /// Add [`gsap_stylesheet`](crate::gsap_stylesheet) for its style.
    ///
    /// ```rust
    /// let bar = autumn_plugin_gsap::Gsap::scroll_progress().into_string();
    /// assert!(bar.contains(r#"data-gsap="scroll-progress""#));
    /// ```
    #[must_use]
    pub fn scroll_progress() -> Markup {
        html! {
            div class="gsap-progress" data-gsap="scroll-progress" aria-hidden="true" {}
        }
    }

    common_setters!();

    /// Animates the direct children, one after another (`data-gsap-stagger`).
    /// `each` is the time between two children.
    pub const fn stagger(mut self, each: Duration) -> Self {
        self.stagger = Some(each);
        self
    }

    /// Sets where the stagger starts (`data-gsap-stagger-from`).
    pub const fn stagger_from(mut self, from: StaggerFrom) -> Self {
        self.stagger_from = Some(from);
        self
    }

    /// Sets how the stagger times spread (`data-gsap-stagger-ease`).
    pub const fn stagger_ease(mut self, ease: Ease) -> Self {
        self.stagger_ease = Some(ease);
        self
    }

    /// Splits the text with SplitText and animates each unit (`data-gsap-split`).
    /// The default stagger for split text is 0.03 s.
    pub const fn split(mut self, unit: Split) -> Self {
        self.split = Some(unit);
        self
    }

    /// Clips each split unit, so it slides in from behind a mask (`data-gsap-split-mask`).
    pub const fn split_mask(mut self) -> Self {
        self.split_mask = true;
        self
    }

    /// Sets the start time in a [`Timeline`](crate::Timeline) (`data-gsap-position`).
    /// With no position, the step starts at the end of the timeline.
    /// Outside a timeline, `init.js` ignores the position.
    pub const fn position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// The `data-gsap*` attributes, in a fixed order.
    /// Use them to write the attributes on your own element.
    #[must_use]
    pub fn attributes(&self) -> Vec<Attr> {
        let kind = self.preset.map_or("custom", Preset::name);
        let mut out = vec![("data-gsap", kind.to_owned())];
        push_opt(
            &mut out,
            "data-gsap-from",
            self.from.as_ref().map(Props::to_json),
        );
        push_opt(
            &mut out,
            "data-gsap-to",
            self.to.as_ref().map(Props::to_json),
        );
        self.common.push(&mut out);
        push_opt(&mut out, "data-gsap-stagger", self.stagger.map(secs));
        push_opt(
            &mut out,
            "data-gsap-stagger-from",
            self.stagger_from.map(|f| match f {
                StaggerFrom::Start => "start".to_owned(),
                StaggerFrom::Center => "center".to_owned(),
                StaggerFrom::End => "end".to_owned(),
                StaggerFrom::Edges => "edges".to_owned(),
                StaggerFrom::Random => "random".to_owned(),
                StaggerFrom::Index(i) => i.to_string(),
            }),
        );
        push_opt(
            &mut out,
            "data-gsap-stagger-ease",
            self.stagger_ease.map(|e| e.to_string()),
        );
        push_opt(
            &mut out,
            "data-gsap-split",
            self.split.map(|s| {
                match s {
                    Split::Chars => "chars",
                    Split::Words => "words",
                    Split::Lines => "lines",
                }
                .to_owned()
            }),
        );
        push_flag(
            &mut out,
            "data-gsap-split-mask",
            self.split_mask && self.split.is_some(),
        );
        push_opt(
            &mut out,
            "data-gsap-position",
            self.position.map(|p| p.to_string()),
        );
        out
    }
}

/// A scroll-linked vertical drift (`data-gsap="parallax"`).
///
/// The element moves `factor` × its own height while it crosses the viewport.
/// A positive factor moves it down. A factor that is not finite writes no value
/// (`init.js` then uses 0.3). Keep the factor small, for example from -0.5 to 0.5.
///
/// ```rust
/// use autumn_plugin_gsap::{Gsap, Tag};
/// use autumn_web::html;
///
/// let art = Gsap::parallax(-0.3).class("art").wrap_in(Tag::Figure, html! {});
/// assert!(art.into_string().contains(r#"data-gsap-parallax="-0.3""#));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct Parallax {
    factor: f32,
    id: Option<String>,
    class: Option<String>,
    animate_reduced: bool,
}

impl Parallax {
    /// Sets the `id` of the wrapper element. A blank string sets no `id`.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into()).filter(|s| !s.trim().is_empty());
        self
    }

    /// Sets the `class` of the wrapper element. A blank string sets no `class`.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into()).filter(|s| !s.trim().is_empty());
        self
    }

    /// Also animates when the user prefers reduced motion (`data-gsap-reduced="animate"`).
    pub const fn animate_on_reduced_motion(mut self) -> Self {
        self.animate_reduced = true;
        self
    }

    /// The `data-gsap*` attributes, in a fixed order.
    #[must_use]
    pub fn attributes(&self) -> Vec<Attr> {
        let mut out = vec![("data-gsap", "parallax".to_owned())];
        push_opt(&mut out, "data-gsap-parallax", num(self.factor));
        push_opt(
            &mut out,
            "data-gsap-reduced",
            self.animate_reduced.then(|| "animate".to_owned()),
        );
        out
    }

    /// Puts `markup` in a `<div>` with the attributes.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn wrap(self, markup: Markup) -> Markup {
        self.wrap_in(crate::Tag::Div, markup)
    }

    /// Puts `markup` in a `tag` element with the `id`, `class` and animation attributes.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn wrap_in(self, tag: crate::Tag, markup: Markup) -> Markup {
        let mut attrs = Vec::new();
        push_opt(&mut attrs, "id", self.id.clone());
        push_opt(&mut attrs, "class", self.class.clone());
        attrs.extend(self.attributes());
        crate::attrs::wrap_el(tag, &attrs, &markup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scroll::{Action, Edge, Play, ScrollPos, Scrub, ToggleActions};
    use crate::{EaseDir, Repeat};

    fn attrs(g: &Gsap) -> Vec<(String, String)> {
        g.attributes()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect()
    }

    fn has(g: &Gsap, name: &str, value: &str) -> bool {
        attrs(g).iter().any(|(k, v)| k == name && v == value)
    }

    #[test]
    fn presets_write_their_names() {
        let cases = [
            (Gsap::fade(), "fade"),
            (Gsap::fade_up(), "fade-up"),
            (Gsap::fade_down(), "fade-down"),
            (Gsap::fade_left(), "fade-left"),
            (Gsap::fade_right(), "fade-right"),
            (Gsap::scale(), "scale"),
            (Gsap::zoom_in(), "zoom-in"),
            (Gsap::zoom_out(), "zoom-out"),
            (Gsap::slide_up(), "slide-up"),
            (Gsap::slide_down(), "slide-down"),
            (Gsap::slide_left(), "slide-left"),
            (Gsap::slide_right(), "slide-right"),
            (Gsap::rotate_in(), "rotate-in"),
            (Gsap::blur_in(), "blur-in"),
            (Gsap::flip_x(), "flip-x"),
        ];
        for (g, name) in cases {
            assert_eq!(attrs(&g), vec![("data-gsap".to_owned(), name.to_owned())]);
        }
    }

    #[test]
    fn preset_names_are_unique_and_kebab_case() {
        let names: Vec<_> = Preset::ALL.iter().map(|p| p.name()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
        for n in names {
            assert_ne!(n, "");
            assert!(
                n.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'),
                "{n}"
            );
        }
    }

    #[test]
    fn defaults_write_only_the_preset() {
        let g = Gsap::fade_up().play(Play::Scroll);
        assert_eq!(attrs(&g).len(), 1, "{:?}", attrs(&g));
    }

    #[test]
    fn an_explicit_duration_is_always_written() {
        // A timeline child must override the timeline default, also with 0.8 s.
        let g = Gsap::fade().duration(Duration::from_millis(800));
        assert!(has(&g, "data-gsap-duration", "0.8"));
    }

    #[test]
    fn empty_strings_write_no_attribute() {
        let g = Gsap::fade().id("").class(" ").trigger("  ");
        assert_eq!(attrs(&g).len(), 1, "{:?}", attrs(&g));
        let html = g.wrap(html! {}).into_string();
        assert_eq!(html, r#"<div data-gsap="fade"></div>"#);
    }

    #[test]
    fn empty_props_write_no_json() {
        let g = Gsap::from_to(Props::new(), Props::new().x(f32::NAN));
        assert_eq!(
            attrs(&g),
            vec![("data-gsap".to_owned(), "custom".to_owned())]
        );
    }

    #[test]
    fn timing_options_write_attributes() {
        let g = Gsap::fade()
            .play(Play::Load)
            .delay(Duration::from_millis(200))
            .duration(Duration::from_millis(1200))
            .ease(Ease::Expo(EaseDir::Out))
            .repeat(Repeat::Times(2))
            .yoyo()
            .repeat_delay(Duration::from_millis(100));
        assert!(has(&g, "data-gsap-on", "load"));
        assert!(has(&g, "data-gsap-delay", "0.2"));
        assert!(has(&g, "data-gsap-duration", "1.2"));
        assert!(has(&g, "data-gsap-ease", "expo.out"));
        assert!(has(&g, "data-gsap-repeat", "2"));
        assert!(has(&g, "data-gsap-yoyo", ""));
        assert!(has(&g, "data-gsap-repeat-delay", "0.1"));
        let forever = Gsap::fade().repeat(Repeat::Forever);
        assert!(has(&forever, "data-gsap-repeat", "-1"));
    }

    #[test]
    fn scroll_options_write_attributes() {
        let g = Gsap::fade_up()
            .trigger("#hero")
            .start(ScrollPos::new(Edge::Top, Edge::Center))
            .end(ScrollPos::Distance(400.0))
            .toggle_actions(ToggleActions::new(
                Action::Play,
                Action::None,
                Action::None,
                Action::Reverse,
            ))
            .replay()
            .scrub(Scrub::Smooth(Duration::from_millis(500)))
            .pin()
            .markers()
            .animate_on_reduced_motion();
        assert!(has(&g, "data-gsap-trigger", "#hero"));
        assert!(has(&g, "data-gsap-start", "top center"));
        assert!(has(&g, "data-gsap-end", "+=400"));
        assert!(has(
            &g,
            "data-gsap-toggle-actions",
            "play none none reverse"
        ));
        assert!(has(&g, "data-gsap-once", "false"));
        assert!(has(&g, "data-gsap-scrub", "0.5"));
        assert!(has(&g, "data-gsap-pin", ""));
        assert!(has(&g, "data-gsap-markers", ""));
        assert!(has(&g, "data-gsap-reduced", "animate"));
    }

    #[test]
    fn stagger_and_split_write_attributes() {
        let g = Gsap::fade_up()
            .stagger(Duration::from_millis(80))
            .stagger_from(StaggerFrom::Center)
            .stagger_ease(Ease::Power2(EaseDir::In))
            .split(Split::Words)
            .split_mask();
        assert!(has(&g, "data-gsap-stagger", "0.08"));
        assert!(has(&g, "data-gsap-stagger-from", "center"));
        assert!(has(&g, "data-gsap-stagger-ease", "power2.in"));
        assert!(has(&g, "data-gsap-split", "words"));
        assert!(has(&g, "data-gsap-split-mask", ""));
    }

    #[test]
    fn stagger_from_values() {
        let cases = [
            (StaggerFrom::Start, "start"),
            (StaggerFrom::Center, "center"),
            (StaggerFrom::End, "end"),
            (StaggerFrom::Edges, "edges"),
            (StaggerFrom::Random, "random"),
            (StaggerFrom::Index(3), "3"),
        ];
        for (from, want) in cases {
            assert!(has(
                &Gsap::fade().stagger_from(from),
                "data-gsap-stagger-from",
                want
            ));
        }
    }

    #[test]
    fn split_units() {
        for (unit, want) in [
            (Split::Chars, "chars"),
            (Split::Words, "words"),
            (Split::Lines, "lines"),
        ] {
            assert!(has(&Gsap::fade().split(unit), "data-gsap-split", want));
        }
    }

    #[test]
    fn custom_props_write_json() {
        let from = Gsap::from_props(Props::new().x(-40.0));
        assert!(has(&from, "data-gsap", "custom"));
        assert!(has(&from, "data-gsap-from", r#"{"x":-40}"#));
        assert!(!attrs(&from).iter().any(|(k, _)| k == "data-gsap-to"));

        let to = Gsap::to_props(Props::new().rotation(360.0));
        assert!(has(&to, "data-gsap-to", r#"{"rotation":360}"#));
        assert!(!attrs(&to).iter().any(|(k, _)| k == "data-gsap-from"));

        let both = Gsap::from_to(Props::new().scale(0.0), Props::new().scale(1.0));
        assert!(has(&both, "data-gsap-from", r#"{"scale":0}"#));
        assert!(has(&both, "data-gsap-to", r#"{"scale":1}"#));
    }

    #[test]
    fn parallax_writes_the_factor() {
        let p = Gsap::parallax(-0.25);
        assert_eq!(
            p.attributes(),
            vec![
                ("data-gsap", "parallax".to_owned()),
                ("data-gsap-parallax", "-0.25".to_owned()),
            ]
        );
        assert_eq!(Gsap::parallax(f32::NAN).attributes().len(), 1);
    }

    #[test]
    fn parallax_has_only_the_options_that_init_js_reads() {
        let html = Gsap::parallax(0.3)
            .id("art")
            .class("hero-art")
            .animate_on_reduced_motion()
            .wrap_in(crate::Tag::Figure, html! {})
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<figure id="art" class="hero-art" data-gsap="parallax" "#,
                r#"data-gsap-parallax="0.3" data-gsap-reduced="animate"></figure>"#
            )
        );
        assert!(
            Gsap::parallax(0.3)
                .wrap(html! {})
                .into_string()
                .starts_with("<div")
        );
    }

    #[test]
    fn position_writes_an_attribute() {
        let g = Gsap::fade().position(Position::WithPrevious);
        assert!(has(&g, "data-gsap-position", "<"));
        // `>` is not the GSAP default (the end of the timeline), so it is written.
        let after = Gsap::fade().position(Position::After);
        assert!(has(&after, "data-gsap-position", ">"));
        assert!(
            !attrs(&Gsap::fade())
                .iter()
                .any(|(k, _)| k == "data-gsap-position")
        );
    }

    #[test]
    fn scroll_progress_renders_the_bar() {
        let html = Gsap::scroll_progress().into_string();
        assert_eq!(
            html,
            r#"<div class="gsap-progress" data-gsap="scroll-progress" aria-hidden="true"></div>"#
        );
    }

    #[test]
    fn wrap_in_writes_tag_id_and_class() {
        let html = Gsap::fade_up()
            .id("t1")
            .class("title big")
            .wrap_in(crate::Tag::H1, html! { "Hi" })
            .into_string();
        assert_eq!(
            html,
            r#"<h1 id="t1" class="title big" data-gsap="fade-up">Hi</h1>"#
        );
        let attrs = Gsap::fade_up().id("t1").class("x").attributes();
        assert_eq!(attrs.len(), 1, "id and class are not data-gsap attributes");
    }

    #[test]
    fn wrap_keeps_the_inner_markup() {
        let html = Gsap::fade_up()
            .delay(Duration::from_millis(100))
            .wrap(html! { p { "inner" } })
            .into_string();
        assert_eq!(
            html,
            r#"<div data-gsap="fade-up" data-gsap-delay="0.1"><p>inner</p></div>"#
        );
    }
}
