//! Typed tween properties for custom animations.

use crate::fmt::num;

/// Tween properties for [`Gsap::from_props`](crate::Gsap::from_props),
/// [`Gsap::to_props`](crate::Gsap::to_props) and [`Gsap::from_to`](crate::Gsap::from_to).
///
/// Each setter sets one GSAP property. Lengths are pixels, angles are degrees.
/// A value that is not finite is not written.
///
/// ```rust
/// use autumn_plugin_gsap::Props;
///
/// let p = Props::new().x(-40.0).opacity(0.0);
/// assert_eq!(p.to_json(), r#"{"x":-40,"opacity":0}"#);
/// ```
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct Props {
    values: Vec<(&'static str, f32)>,
}

/// Property names that `init.js` accepts, in GSAP spelling.
#[cfg(test)]
pub(crate) const PROP_NAMES: &[&str] = &[
    "x",
    "y",
    "xPercent",
    "yPercent",
    "scale",
    "scaleX",
    "scaleY",
    "rotation",
    "rotationX",
    "rotationY",
    "skewX",
    "skewY",
    "opacity",
    "autoAlpha",
];

macro_rules! prop_setters {
    ($($(#[$doc:meta])* $fn_name:ident => $gsap:literal;)*) => {
        $(
            $(#[$doc])*
            pub fn $fn_name(self, value: f32) -> Self {
                self.set($gsap, value)
            }
        )*
    };
}

impl Props {
    /// Makes an empty property set.
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Sets `name` to `value`. A later call for the same name replaces the value.
    fn set(mut self, name: &'static str, value: f32) -> Self {
        if !value.is_finite() {
            self.values.retain(|(n, _)| *n != name);
        } else if let Some(slot) = self.values.iter_mut().find(|(n, _)| *n == name) {
            slot.1 = value;
        } else {
            self.values.push((name, value));
        }
        self
    }

    prop_setters! {
        /// Horizontal move in pixels (`x`).
        x => "x";
        /// Vertical move in pixels (`y`).
        y => "y";
        /// Horizontal move in percent of the element width (`xPercent`).
        x_percent => "xPercent";
        /// Vertical move in percent of the element height (`yPercent`).
        y_percent => "yPercent";
        /// Scale on both axes (`scale`).
        scale => "scale";
        /// Horizontal scale (`scaleX`).
        scale_x => "scaleX";
        /// Vertical scale (`scaleY`).
        scale_y => "scaleY";
        /// Rotation in degrees (`rotation`).
        rotation => "rotation";
        /// 3D rotation around the x axis in degrees (`rotationX`).
        rotation_x => "rotationX";
        /// 3D rotation around the y axis in degrees (`rotationY`).
        rotation_y => "rotationY";
        /// Horizontal skew in degrees (`skewX`).
        skew_x => "skewX";
        /// Vertical skew in degrees (`skewY`).
        skew_y => "skewY";
        /// Opacity, 0 to 1 (`opacity`).
        opacity => "opacity";
        /// Opacity that also sets `visibility: hidden` at 0 (`autoAlpha`).
        auto_alpha => "autoAlpha";
    }

    /// `true` when no property is set.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The JSON object that `init.js` reads, for example `{"x":-40}`.
    #[must_use]
    pub fn to_json(&self) -> String {
        let body: Vec<String> = self
            .values
            .iter()
            .filter_map(|(name, value)| num(*value).map(|v| format!(r#""{name}":{v}"#)))
            .collect();
        format!("{{{}}}", body.join(","))
    }
}

/// Two sets are equal when they hold the same values. The order does not matter.
impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.values.len() == other.values.len()
            && self.values.iter().all(|v| other.values.contains(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn every_setter_writes_its_gsap_name() {
        let p = Props::new()
            .x(1.0)
            .y(2.0)
            .x_percent(3.0)
            .y_percent(4.0)
            .scale(5.0)
            .scale_x(6.0)
            .scale_y(7.0)
            .rotation(8.0)
            .rotation_x(9.0)
            .rotation_y(10.0)
            .skew_x(11.0)
            .skew_y(12.0)
            .opacity(0.5)
            .auto_alpha(0.25);
        assert_eq!(
            p.to_json(),
            concat!(
                r#"{"x":1,"y":2,"xPercent":3,"yPercent":4,"scale":5,"scaleX":6,"#,
                r#""scaleY":7,"rotation":8,"rotationX":9,"rotationY":10,"skewX":11,"#,
                r#""skewY":12,"opacity":0.5,"autoAlpha":0.25}"#
            )
        );
    }

    #[test]
    fn init_js_accepts_the_same_names() {
        let js = include_str!("../assets/init.js");
        let start = js.find("var PROPS = [").expect("PROPS list");
        let list = &js[start..start + js[start..].find("];").expect("end")];
        let names: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
        assert_eq!(names, PROP_NAMES);
    }

    #[test]
    fn equality_ignores_the_order() {
        assert_eq!(Props::new().x(1.0).y(2.0), Props::new().y(2.0).x(1.0));
        assert_ne!(Props::new().x(1.0), Props::new().x(2.0));
        assert_ne!(Props::new().x(1.0), Props::new().x(1.0).y(0.0));
    }

    #[test]
    fn a_later_value_replaces_an_earlier_one() {
        let p = Props::new().x(1.0).opacity(0.0).x(2.0);
        assert_eq!(p.to_json(), r#"{"x":2,"opacity":0}"#);
    }

    #[test]
    fn values_that_are_not_finite_are_not_written() {
        let p = Props::new().x(f32::NAN).y(f32::INFINITY);
        assert!(p.is_empty());
        assert_eq!(p.to_json(), "{}");
        let q = Props::new().x(1.0).x(f32::NAN);
        assert_eq!(q.to_json(), "{}", "a bad value also clears the old one");
    }

    #[test]
    fn json_is_valid_and_uses_only_known_names() {
        let p = Props::new().rotation(-12.5).auto_alpha(0.0);
        let v: serde_json::Value = serde_json::from_str(&p.to_json()).expect("json");
        let obj = v.as_object().expect("object");
        assert_eq!(obj.len(), 2);
        for key in obj.keys() {
            assert!(PROP_NAMES.contains(&key.as_str()), "{key}");
        }
    }

    proptest! {
        #[test]
        fn json_values_match_the_js_grammar(v in proptest::num::f32::ANY) {
            let json = Props::new().x(v).to_json();
            let parsed: serde_json::Value = serde_json::from_str(&json).expect("json");
            if v.is_finite() {
                let raw = json.trim_start_matches(r#"{"x":"#).trim_end_matches('}');
                prop_assert!(crate::grammar::is_num(raw), "{json}");
                prop_assert!(parsed["x"].is_number());
            } else {
                prop_assert_eq!(json, "{}");
            }
        }
    }
}
