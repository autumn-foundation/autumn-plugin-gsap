//! Test-only copy of the value grammar that `assets/init.js` accepts.
//!
//! Keep each pattern equal to the `RE_*` constant with the same name in `init.js`.
//! The test `init_js_uses_the_same_patterns` checks this.

use regex::Regex;

/// A plain decimal number.
pub(crate) const NUM: &str = r"-?\d+(\.\d+)?";

/// Pattern sources, by `init.js` constant name. `init.js` anchors each pattern with `^…$`.
pub(crate) const PATTERNS: &[(&str, &str)] = &[
    ("RE_SECS", r"\d+(\.\d+)?"),
    ("RE_NUM", r"-?\d+(\.\d+)?"),
    ("RE_INDEX", r"\d+"),
    (
        "RE_EASE",
        r"none|(power[1-4]|sine|expo|circ|bounce)\.(in|out|inOut)|back\.(in|out|inOut)(\(-?\d+(\.\d+)?\))?|elastic\.(in|out|inOut)(\(-?\d+(\.\d+)?,-?\d+(\.\d+)?\))?|steps\([1-9]\d*\)",
    ),
    (
        "RE_SCROLL_POS",
        r"(top|center|bottom|-?\d+(\.\d+)?(px|%)) (top|center|bottom|-?\d+(\.\d+)?(px|%))|\+=\d+(\.\d+)?(px|%)?",
    ),
    (
        "RE_POSITION",
        r"<|>|[<>]-?\d+(\.\d+)?|[+-]=\d+(\.\d+)?|\d+(\.\d+)?",
    ),
];

/// The pattern source for `name`.
fn source(name: &str) -> &'static str {
    PATTERNS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
        .expect("known pattern")
}

/// `true` when `value` matches the whole `name` pattern.
fn full(name: &str, value: &str) -> bool {
    Regex::new(&format!("^({})$", source(name)))
        .expect("valid regex")
        .is_match(value)
}

/// Seconds: a non-negative decimal.
pub(crate) fn is_secs(v: &str) -> bool {
    full("RE_SECS", v)
}

/// A finite decimal.
pub(crate) fn is_num(v: &str) -> bool {
    full("RE_NUM", v)
}

/// A GSAP ease string.
pub(crate) fn is_ease(v: &str) -> bool {
    full("RE_EASE", v)
}

/// A ScrollTrigger `start` or `end` value.
pub(crate) fn is_scroll_pos(v: &str) -> bool {
    full("RE_SCROLL_POS", v)
}

/// A timeline position value.
pub(crate) fn is_position(v: &str) -> bool {
    full("RE_POSITION", v)
}

#[test]
fn num_constant_matches_the_pattern_table() {
    assert_eq!(source("RE_NUM"), NUM);
}

#[test]
fn init_js_uses_the_same_patterns() {
    let js = include_str!("../assets/init.js");
    for (name, src) in PATTERNS {
        let line = format!("var {name} = /^({src})$/;");
        assert!(js.contains(&line), "init.js must contain: {line}");
    }
}
