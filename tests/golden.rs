//! Golden fixture for the Rust/JS attribute contract.
//!
//! This test writes the attributes of many builders to JSON and compares them
//! with `tests/fixtures/attributes.json`. `tests/js/init.test.mjs` parses the same
//! file with `init.js`. Thus both sides agree on each value.
//!
//! To update the fixture after a change, run: `UPDATE_GOLDEN=1 cargo test --test golden`.

use std::time::Duration;

use autumn_plugin_gsap::{
    Action, Attr, Ease, EaseDir, Edge, Gsap, Play, Position, Preset, Props, Repeat, ScrollPos,
    Scrub, Split, StaggerFrom, Timeline, ToggleActions,
};
use serde_json::{Map, Value};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes.json"
);

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

type Case = (String, Vec<Attr>);

fn cases() -> Vec<Case> {
    let mut out = preset_cases();
    out.extend(ease_cases());
    out.extend(scroll_cases());
    out.extend(stagger_cases());
    out.extend(custom_cases());
    out.extend(timeline_cases());
    out
}

fn preset_cases() -> Vec<Case> {
    Preset::ALL
        .iter()
        .map(|p| {
            (
                format!("preset-{}", p.name()),
                Gsap::preset(*p).attributes(),
            )
        })
        .collect()
}

fn ease_cases() -> Vec<Case> {
    let mut out = Vec::new();
    let dirs = [EaseDir::In, EaseDir::Out, EaseDir::InOut];
    let mut eases = vec![Ease::None, Ease::Steps(6)];
    for d in dirs {
        eases.extend([
            Ease::Power1(d),
            Ease::Power2(d),
            Ease::Power3(d),
            Ease::Power4(d),
            Ease::Sine(d),
            Ease::Expo(d),
            Ease::Circ(d),
            Ease::Bounce(d),
            Ease::Back(d, 2.5),
            Ease::Elastic(d, 1.2, 0.4),
        ]);
    }
    for e in eases {
        out.push((
            format!("ease-{e}"),
            Gsap::fade().ease(e).stagger_ease(e).attributes(),
        ));
    }
    out
}

fn scroll_cases() -> Vec<Case> {
    let mut out = Vec::new();
    let actions = [
        Action::Play,
        Action::Pause,
        Action::Resume,
        Action::Reverse,
        Action::Restart,
        Action::Reset,
        Action::Complete,
        Action::None,
    ];
    for a in actions {
        out.push((
            format!(
                "toggle-{}",
                ToggleActions::new(a, Action::None, a, Action::Reverse)
            ),
            Gsap::fade()
                .toggle_actions(ToggleActions::new(a, Action::None, a, Action::Reverse))
                .attributes(),
        ));
    }
    let edges = [
        Edge::Top,
        Edge::Center,
        Edge::Bottom,
        Edge::Percent(75.5),
        Edge::Px(-40.0),
    ];
    for a in edges {
        for b in edges {
            out.push((
                format!("start-{}", ScrollPos::new(a, b)),
                Gsap::fade().start(ScrollPos::new(a, b)).attributes(),
            ));
        }
    }
    out.push((
        "scrub-linked".to_owned(),
        Gsap::fade()
            .scrub(Scrub::Linked)
            .repeat(Repeat::Forever)
            .attributes(),
    ));
    out
}

fn stagger_cases() -> Vec<Case> {
    let mut out = Vec::new();
    let froms = [
        StaggerFrom::Start,
        StaggerFrom::Center,
        StaggerFrom::End,
        StaggerFrom::Edges,
        StaggerFrom::Random,
        StaggerFrom::Index(4),
    ];
    for f in froms {
        out.push((
            format!("stagger-from-{f:?}"),
            Gsap::fade_up().stagger(ms(60)).stagger_from(f).attributes(),
        ));
    }
    for s in [Split::Chars, Split::Words, Split::Lines] {
        out.push((
            format!("split-{s:?}"),
            Gsap::fade_up().split(s).split_mask().attributes(),
        ));
    }
    out
}

fn custom_cases() -> Vec<Case> {
    vec![
        (
            "everything".to_owned(),
            Gsap::fade_up()
                .play(Play::Load)
                .delay(ms(250))
                .duration(ms(1250))
                .ease(Ease::Back(EaseDir::Out, 1.7))
                .repeat(Repeat::Times(3))
                .yoyo()
                .repeat_delay(ms(400))
                .trigger("#hero .title")
                .start(ScrollPos::new(Edge::Top, Edge::Percent(80.0)))
                .end(ScrollPos::Distance(600.0))
                .replay()
                .scrub(Scrub::Smooth(ms(800)))
                .pin()
                .markers()
                .animate_on_reduced_motion()
                .position(Position::Overlap(ms(300)))
                .attributes(),
        ),
        (
            "custom".to_owned(),
            Gsap::from_to(
                Props::new()
                    .x(-40.0)
                    .y(10.5)
                    .x_percent(-100.0)
                    .y_percent(50.0)
                    .scale(0.5)
                    .scale_x(1.5)
                    .scale_y(0.75)
                    .rotation(-90.0)
                    .rotation_x(45.0)
                    .rotation_y(30.0)
                    .skew_x(10.0)
                    .skew_y(-5.0)
                    .opacity(0.0)
                    .auto_alpha(0.0),
                Props::new().x(0.0).opacity(1.0),
            )
            .attributes(),
        ),
        ("parallax".to_owned(), Gsap::parallax(-0.3).attributes()),
        (
            "parallax-reduced".to_owned(),
            Gsap::parallax(0.2).animate_on_reduced_motion().attributes(),
        ),
        (
            "custom-to-only".to_owned(),
            Gsap::to_props(Props::new().rotation(360.0)).attributes(),
        ),
        (
            "zero-times".to_owned(),
            Gsap::fade()
                .duration(ms(0))
                .delay(ms(0))
                .repeat(Repeat::Times(0))
                .repeat_delay(ms(0))
                .stagger(ms(0))
                .attributes(),
        ),
    ]
}

fn timeline_cases() -> Vec<Case> {
    let mut out = Vec::new();
    let positions = [
        Position::After,
        Position::WithPrevious,
        Position::AfterPreviousStart(ms(150)),
        Position::Gap(ms(500)),
        Position::Overlap(ms(250)),
        Position::At(ms(2000)),
    ];
    for p in positions {
        out.push((
            format!("position-{p}"),
            Gsap::fade().position(p).attributes(),
        ));
    }
    out.push((
        "timeline-scroll".to_owned(),
        Timeline::new()
            .delay(ms(100))
            .repeat_delay(ms(200))
            .trigger("#stage")
            .toggle_actions(ToggleActions::new(
                Action::Play,
                Action::Pause,
                Action::Resume,
                Action::Reset,
            ))
            .replay()
            .markers()
            .animate_on_reduced_motion()
            .attributes(),
    ));
    out.push((
        "timeline".to_owned(),
        Timeline::new()
            .play(Play::Load)
            .duration(ms(500))
            .ease(Ease::Power2(EaseDir::InOut))
            .repeat(Repeat::Forever)
            .yoyo()
            .start(ScrollPos::new(Edge::Top, Edge::Top))
            .end(ScrollPos::Distance(1500.0))
            .scrub(Scrub::Linked)
            .pin()
            .attributes(),
    ));
    out
}

fn to_value(cases: &[Case]) -> Value {
    let mut map = Map::new();
    for (name, attrs) in cases {
        let list = attrs
            .iter()
            .map(|(k, v)| Value::Array(vec![Value::from(*k), Value::from(v.as_str())]))
            .collect();
        map.insert(name.clone(), Value::Array(list));
    }
    Value::Object(map)
}

#[test]
fn case_names_are_unique() {
    let cases = cases();
    let mut names: Vec<_> = cases.iter().map(|(n, _)| n.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), cases.len());
}

#[test]
fn fixture_matches_the_builders() {
    let mut want = serde_json::to_string_pretty(&to_value(&cases())).expect("json");
    want.push('\n');
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(FIXTURE, &want).expect("write fixture");
    }
    let have = std::fs::read_to_string(FIXTURE).unwrap_or_default();
    assert!(
        have == want,
        "tests/fixtures/attributes.json is stale. Run: UPDATE_GOLDEN=1 cargo test --test golden"
    );
}
