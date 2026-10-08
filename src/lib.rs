//! GSAP animations for Autumn, with typed Rust builders and htmx support.
//!
//! 1. Add [`GsapPlugin`] to the app.
//! 2. Put [`gsap_script()`] in the page `<head>`.
//! 3. Animate with [`Gsap`], [`Timeline`] or raw `data-gsap` attributes.
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use autumn_plugin_gsap::{Gsap, GsapPlugin, Split, gsap_script};
//! use autumn_web::prelude::*;
//!
//! #[get("/")]
//! async fn index() -> Markup {
//!     html! {
//!         html {
//!             head { (gsap_script()) }
//!             body {
//!                 (Gsap::fade_up().split(Split::Chars).wrap(html! { h1 { "Hello" } }))
//!                 p data-gsap="fade" data-gsap-delay="0.2" { "World" }
//!             }
//!         }
//!     }
//! }
//!
//! # async fn run() {
//! autumn_web::app()
//!     .plugin(GsapPlugin::new())
//!     .routes(routes![index])
//!     .run()
//!     .await;
//! # }
//! ```
//!
//! The crate vendors GSAP 3.15.0 with ScrollTrigger and SplitText. You need no npm and no bundler.
//! `init.js` reads the attributes on page load and on each `htmx:load`.
//! It reverts the animations of content that htmx removes.
//! When the user prefers reduced motion, elements do not animate.
//!
//! The GSAP files use the [GSAP Standard License](https://gsap.com/standard-license).
//! The plugin code uses Apache-2.0.

mod assets;
mod attrs;
mod ease;
mod fmt;
#[cfg(test)]
mod grammar;
mod options;
mod plugin;
mod props;
mod script;
mod scroll;
mod timeline;
mod tween;

pub use assets::{ASSETS_NAMESPACE, GSAP_ASSETS, GSAP_LICENSE, GSAP_UPSTREAM, GSAP_VERSION};
pub use attrs::{Attr, Tag};
pub use ease::{Ease, EaseDir};
pub use options::Repeat;
pub use plugin::{GsapPlugin, PLUGIN_NAME};
pub use props::Props;
pub use script::{gsap_script, gsap_stylesheet};
pub use scroll::{Action, Edge, Play, ScrollPos, Scrub, ToggleActions};
pub use timeline::{Position, Timeline};
pub use tween::{Gsap, Preset, Split, StaggerFrom};
