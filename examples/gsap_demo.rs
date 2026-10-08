//! GSAP demo: a small Autumn app that shows `autumn-plugin-gsap`.
//!
//! Run it, then open <http://127.0.0.1:3000>:
//!
//! ```sh
//! cargo run --example gsap_demo
//! ```
//!
//! The page shows these items:
//!
//! - A hero timeline plays on load. It splits the heading, then staggers the text.
//! - Cards appear on scroll. They use presets, custom props and a 3D flip.
//! - A pinned section plays a scrubbed timeline.
//! - Parallax art and a scroll progress bar follow the scroll.
//! - htmx buttons add ("Load more") and replace ("Replace") content.
//!   New content animates. init.js reverts the animations of removed content.
//!
//! All JS and CSS are files, so the page works with the default Autumn CSP.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use autumn_plugin_gsap::{
    Ease, EaseDir, Edge, Gsap, GsapPlugin, Play, Position, Props, ScrollPos, Scrub, Split,
    StaggerFrom, Tag, Timeline, gsap_script, gsap_stylesheet,
};
use autumn_web::assets::asset_url;
use autumn_web::{Markup, html};

/// The crate `static/` directory, embedded in the binary.
static STATIC: autumn_web::include_dir::Dir = autumn_web::embed_static!();

/// Counts the htmx batches.
static BATCH: AtomicU32 = AtomicU32::new(1);

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(GsapPlugin::new())
        .embedded_static(&STATIC)
        .routes(autumn_web::routes![index, about, more, swap, plain])
        .run()
        .await;
}

fn layout(content: &Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "GSAP demo" }
                // htmx adds an inline <style> for indicators. A strict CSP blocks it.
                meta name="htmx-config" content=r#"{"includeIndicatorStyles":false}"#;
                link rel="stylesheet" href=(asset_url("css/demo.css"));
                (gsap_stylesheet())
                (gsap_script())
                script src=(asset_url("js/htmx.min.js")) defer {}
            }
            body {
                (Gsap::scroll_progress())
                main class="wrap" { (content) }
                footer class="wrap" {
                    // A boosted link: htmx keeps this page in its history cache. Back restores it.
                    a href="/about" hx-boost="true" id="about-link" { "About" }
                }
            }
        }
    }
}

fn hero() -> Markup {
    Timeline::new()
        .play(Play::Load)
        .id("hero")
        .class("hero")
        .wrap_in(
            Tag::Section,
            html! {
                (Gsap::fade().class("kicker").wrap(html! { "autumn-plugin-gsap" }))
                // SplitText on the heading itself: the heading gets the `aria-label`.
                (Gsap::fade_up()
                    .split(Split::Chars)
                    .split_mask()
                    .ease(Ease::Expo(EaseDir::Out))
                    .id("split")
                    .wrap_in(Tag::H1, html! { "Server HTML, GSAP motion." }))
                (Gsap::fade_up()
                    .stagger(ms(120))
                    .position(Position::Overlap(ms(400)))
                    .id("lede")
                    .wrap(html! {
                        p { "Maud renders it. htmx swaps it. GSAP moves it." }
                        p { "You write no JavaScript." }
                    }))
            },
        )
}

fn cards() -> Markup {
    html! {
        section class="cards" {
            (Gsap::fade_up().id("card-fade-up").class("card").wrap(html! {
                h2 { "Presets" }
                p { "Gsap::fade_up() writes data-gsap=\"fade-up\". init.js does the rest." }
            }))
            (Gsap::flip_x().duration(ms(1000)).class("card").wrap(html! {
                h2 { "Typed in Rust" }
                p { "Ease, ScrollPos, Position and Props are types. The compiler checks them." }
            }))
            (Gsap::from_props(Props::new().x(-120.0).rotation(-8.0).opacity(0.0))
                .ease(Ease::Back(EaseDir::Out, 1.7))
                .id("card-custom")
                .class("card")
                .wrap(html! {
                    h2 { "Custom props" }
                    p { "Gsap::from_props(Props::new().x(-120.0).rotation(-8.0))" }
                }))
            (Gsap::scale()
                .stagger(ms(80))
                .stagger_from(StaggerFrom::Center)
                .id("chips")
                .class("chips")
                .wrap_in(Tag::Ul, html! {
                    @for n in 1..=6 { li class="chip" { "chip " (n) } }
                }))
            div id="card-raw" class="card" data-gsap="zoom-in"
                data-gsap-ease="elastic.out(1,0.4)" data-gsap-duration="1.4" {
                h2 { "Raw attributes" }
                p { "data-gsap=\"zoom-in\" data-gsap-ease=\"elastic.out(1,0.4)\"" }
            }
        }
    }
}

fn pinned() -> Markup {
    Timeline::new()
        .start(ScrollPos::new(Edge::Top, Edge::Top))
        .end(ScrollPos::Distance(1200.0))
        .scrub(Scrub::Smooth(ms(500)))
        .pin()
        .id("pinned")
        .class("pinned")
        .wrap_in(
            Tag::Section,
            html! {
                (Gsap::to_props(Props::new().x_percent(-60.0).rotation(-4.0))
                    .class("band")
                    .wrap(html! { "Pinned. Scrubbed. Typed." }))
                (Gsap::to_props(Props::new().scale(1.4).opacity(0.1))
                    .position(Position::WithPrevious)
                    .class("orb")
                    .wrap(html! {}))
            },
        )
}

#[autumn_web::get("/")]
async fn index() -> Markup {
    layout(&html! {
        (hero())
        (Gsap::parallax(-0.4).id("parallax").class("art").wrap(html! {}))
        (cards())
        (pinned())
        section id="htmx" class="htmx-zone" {
            h2 { "htmx" }
            p { "New content animates on htmx:load. init.js reverts removed content." }
            div class="buttons" {
                button hx-get="/more" hx-target="#more" hx-swap="beforeend" id="load-more" {
                    "Load more"
                }
                button hx-get="/swap" hx-target="#swap" hx-swap="innerHTML" id="replace" {
                    "Replace"
                }
            }
            // aria-live: screen readers announce the new content.
            div id="more" aria-live="polite" {}
            div id="swap" aria-live="polite" { (swap_panel(0)) }
        }
        p class="end" { "End." }
    })
}

/// A stagger panel. `replay` keeps its ScrollTrigger alive, so a swap must revert it.
fn swap_panel(n: u32) -> Markup {
    Gsap::fade_up()
        .stagger(ms(60))
        .replay()
        .start(ScrollPos::new(Edge::Top, Edge::Bottom))
        .wrap(html! {
            @for i in 1..=3 {
                div class="item swap-item" { "Panel " (n) " · item " (i) }
            }
        })
}

/// A second page for the htmx history test.
#[autumn_web::get("/about")]
async fn about() -> Markup {
    layout(&html! {
        (Gsap::fade_up().play(Play::Load).id("about").wrap_in(Tag::H1, html! { "About" }))
        a href="/" hx-boost="true" { "Back to the demo" }
    })
}

/// htmx partial with no animation. The e2e tests use it to change the page height.
#[autumn_web::get("/plain")]
async fn plain() -> Markup {
    html! { p class="plain" { "Plain content." } }
}

/// htmx partial: one more batch. It is a stagger container.
#[autumn_web::get("/more")]
async fn more() -> Markup {
    let n = BATCH.fetch_add(1, Ordering::Relaxed);
    Gsap::slide_up()
        .stagger(ms(90))
        .start(ScrollPos::new(Edge::Top, Edge::Bottom))
        .wrap(html! {
            @for i in 1..=3 {
                div class="item more-item" { "Batch " (n) " · item " (i) }
            }
        })
}

/// htmx partial: a new panel that replaces the old one.
#[autumn_web::get("/swap")]
async fn swap() -> Markup {
    swap_panel(BATCH.fetch_add(1, Ordering::Relaxed))
}
