//! Typed GSAP eases.

use std::fmt;

use crate::fmt::num;

/// The direction of an [`Ease`] curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EaseDir {
    /// Slow start.
    In,
    /// Slow end.
    Out,
    /// Slow start and slow end.
    InOut,
}

impl EaseDir {
    /// The GSAP name (`in`, `out`, `inOut`).
    const fn gsap(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
            Self::InOut => "inOut",
        }
    }
}

/// A GSAP ease. It writes the GSAP ease string, for example `power3.out`.
///
/// ```rust
/// use autumn_plugin_gsap::{Ease, EaseDir};
///
/// assert_eq!(Ease::Power3(EaseDir::Out).to_string(), "power3.out");
/// assert_eq!(Ease::Back(EaseDir::Out, 1.7).to_string(), "back.out(1.7)");
/// assert_eq!(Ease::Steps(5).to_string(), "steps(5)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Ease {
    /// Constant speed (`none`).
    None,
    /// `power1` (quad).
    Power1(EaseDir),
    /// `power2` (cubic).
    Power2(EaseDir),
    /// `power3` (quart).
    Power3(EaseDir),
    /// `power4` (quint).
    Power4(EaseDir),
    /// `sine`.
    Sine(EaseDir),
    /// `expo`.
    Expo(EaseDir),
    /// `circ`.
    Circ(EaseDir),
    /// `back` with an overshoot value (GSAP default `1.7`).
    /// A value that is not finite writes no parameter.
    Back(EaseDir, f32),
    /// `elastic` with amplitude and period (GSAP default `1`, `0.3`).
    /// A value that is not finite writes no parameters.
    Elastic(EaseDir, f32, f32),
    /// `bounce`.
    Bounce(EaseDir),
    /// `steps(n)`. The minimum is 1.
    Steps(u32),
}

impl fmt::Display for Ease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, dir) = match *self {
            Self::None => return f.write_str("none"),
            Self::Steps(n) => return write!(f, "steps({})", n.max(1)),
            Self::Power1(d) => ("power1", d),
            Self::Power2(d) => ("power2", d),
            Self::Power3(d) => ("power3", d),
            Self::Power4(d) => ("power4", d),
            Self::Sine(d) => ("sine", d),
            Self::Expo(d) => ("expo", d),
            Self::Circ(d) => ("circ", d),
            Self::Bounce(d) => ("bounce", d),
            Self::Back(d, overshoot) => {
                write!(f, "back.{}", d.gsap())?;
                return num(overshoot).map_or(Ok(()), |o| write!(f, "({o})"));
            }
            Self::Elastic(d, amplitude, period) => {
                write!(f, "elastic.{}", d.gsap())?;
                return match (num(amplitude), num(period)) {
                    (Some(a), Some(p)) => write!(f, "({a},{p})"),
                    _ => Ok(()),
                };
            }
        };
        write!(f, "{name}.{}", dir.gsap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn named_eases_write_gsap_strings() {
        let cases = [
            (Ease::None, "none"),
            (Ease::Power1(EaseDir::In), "power1.in"),
            (Ease::Power2(EaseDir::Out), "power2.out"),
            (Ease::Power3(EaseDir::InOut), "power3.inOut"),
            (Ease::Power4(EaseDir::Out), "power4.out"),
            (Ease::Sine(EaseDir::InOut), "sine.inOut"),
            (Ease::Expo(EaseDir::Out), "expo.out"),
            (Ease::Circ(EaseDir::In), "circ.in"),
            (Ease::Bounce(EaseDir::Out), "bounce.out"),
        ];
        for (ease, want) in cases {
            assert_eq!(ease.to_string(), want);
        }
    }

    #[test]
    fn parametric_eases_write_parameters() {
        assert_eq!(Ease::Back(EaseDir::Out, 1.7).to_string(), "back.out(1.7)");
        assert_eq!(
            Ease::Elastic(EaseDir::Out, 1.0, 0.3).to_string(),
            "elastic.out(1,0.3)"
        );
        assert_eq!(Ease::Steps(12).to_string(), "steps(12)");
    }

    #[test]
    fn bad_parameters_fall_back_to_gsap_defaults() {
        assert_eq!(Ease::Back(EaseDir::In, f32::NAN).to_string(), "back.in");
        assert_eq!(
            Ease::Elastic(EaseDir::InOut, 1.0, f32::INFINITY).to_string(),
            "elastic.inOut"
        );
        assert_eq!(Ease::Steps(0).to_string(), "steps(1)");
    }

    fn dir() -> impl Strategy<Value = EaseDir> {
        prop_oneof![Just(EaseDir::In), Just(EaseDir::Out), Just(EaseDir::InOut)]
    }

    fn ease() -> impl Strategy<Value = Ease> {
        let f = proptest::num::f32::ANY;
        prop_oneof![
            Just(Ease::None),
            dir().prop_map(Ease::Power1),
            dir().prop_map(Ease::Power2),
            dir().prop_map(Ease::Power3),
            dir().prop_map(Ease::Power4),
            dir().prop_map(Ease::Sine),
            dir().prop_map(Ease::Expo),
            dir().prop_map(Ease::Circ),
            dir().prop_map(Ease::Bounce),
            (dir(), f).prop_map(|(d, o)| Ease::Back(d, o)),
            (dir(), f, f).prop_map(|(d, a, p)| Ease::Elastic(d, a, p)),
            any::<u32>().prop_map(Ease::Steps),
        ]
    }

    proptest! {
        #[test]
        fn every_ease_matches_the_js_grammar(e in ease()) {
            let s = e.to_string();
            prop_assert!(crate::grammar::is_ease(&s), "{s}");
        }
    }
}
