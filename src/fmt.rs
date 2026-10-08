//! Number formats for attribute values.

use std::time::Duration;

/// Writes `d` as seconds with millisecond precision (`150ms` → `"0.15"`).
pub(crate) fn secs(d: Duration) -> String {
    let whole = d.as_secs();
    let millis = d.subsec_millis();
    if millis == 0 {
        return whole.to_string();
    }
    let frac = format!("{millis:03}");
    format!("{whole}.{}", frac.trim_end_matches('0'))
}

/// Writes a finite `f32` in plain decimal form. Returns `None` for NaN and infinity.
pub(crate) fn num(v: f32) -> Option<String> {
    // `+ 0.0` turns `-0.0` into `0.0`. `f32` `Display` never uses exponents.
    v.is_finite().then(|| format!("{}", v + 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn secs_uses_milliseconds() {
        assert_eq!(secs(Duration::ZERO), "0");
        assert_eq!(secs(Duration::from_millis(150)), "0.15");
        assert_eq!(secs(Duration::from_millis(1500)), "1.5");
        assert_eq!(secs(Duration::from_secs(2)), "2");
        assert_eq!(secs(Duration::from_millis(1001)), "1.001");
        assert_eq!(secs(Duration::from_micros(1999)), "0.001");
    }

    #[test]
    fn num_writes_plain_decimals() {
        assert_eq!(num(1.7).as_deref(), Some("1.7"));
        assert_eq!(num(-40.0).as_deref(), Some("-40"));
        assert_eq!(num(0.0).as_deref(), Some("0"));
        assert_eq!(num(-0.0).as_deref(), Some("0"));
        assert_eq!(num(f32::NAN), None);
        assert_eq!(num(f32::INFINITY), None);
        assert_eq!(num(f32::NEG_INFINITY), None);
    }

    proptest! {
        #[test]
        fn secs_matches_the_js_grammar(ms in 0u64..10_000_000) {
            let s = secs(Duration::from_millis(ms));
            prop_assert!(crate::grammar::is_secs(&s), "{s}");
            let back: f64 = s.parse().expect("number");
            #[allow(clippy::cast_precision_loss)]
            let want = ms as f64 / 1000.0;
            prop_assert!((back - want).abs() < 1e-9, "{s} != {want}");
        }

        #[test]
        fn num_matches_the_js_grammar(v in proptest::num::f32::ANY) {
            if let Some(s) = num(v) {
                prop_assert!(crate::grammar::is_num(&s), "{s}");
            } else {
                prop_assert!(!v.is_finite());
            }
        }
    }
}
