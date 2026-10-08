# autumn-plugin-gsap

[GSAP](https://gsap.com) animations for [Autumn](https://github.com/autumn-foundation/autumn) 0.8 apps.
You write Rust and Maud. You need no npm, no bundler and no JavaScript.

- The crate includes GSAP 3.15.0, ScrollTrigger and SplitText.
- The plugin serves them through the plugin asset seam of Autumn: hashed URLs, SRI, immutable cache.
- Typed builders: `Gsap`, `Timeline`, `Parallax`, `Ease`, `ScrollPos`, `Position`, `Props`, `Tag`.
- htmx: new content animates. `init.js` reverts the animations of removed content.
- Elements do not animate when the user prefers reduced motion.
- The plugin works with a strict CSP.

## Quickstart

1. Add the plugin.

   ```rust
   use autumn_plugin_gsap::GsapPlugin;

   autumn_web::app().plugin(GsapPlugin::new()).run().await;
   ```

2. Put the tags in the page `<head>`. The script tags have the `defer` attribute and SRI hashes.

   ```rust
   use autumn_plugin_gsap::{gsap_script, gsap_stylesheet};

   html! { head { (gsap_script()) (gsap_stylesheet()) } }
   ```

3. Animate.

   ```rust
   use std::time::Duration;
   use autumn_plugin_gsap::{Ease, EaseDir, Gsap, Split, Tag};

   html! {
       // A tween on a wrapper element.
       (Gsap::fade_up().delay(Duration::from_millis(100)).class("card").wrap(html! {
           h2 { "Hello" }
       }))
       // SplitText on the heading itself. The heading gets the aria-label.
       (Gsap::fade_up()
           .split(Split::Chars)
           .ease(Ease::Expo(EaseDir::Out))
           .wrap_in(Tag::H1, html! { "Big title" }))
       // Raw attributes on your own element.
       p data-gsap="fade" data-gsap-delay="0.2" { "World" }
   }
   ```

We recommend `gsap_stylesheet()` on each page. It shows all content in print,
keeps split masks from cutting letters, and styles `Gsap::scroll_progress()`.

## Features

### Tweens

```rust
use autumn_plugin_gsap::{Gsap, Props, StaggerFrom};

Gsap::zoom_in();                                                 // a preset
Gsap::scale().stagger(ms(80)).stagger_from(StaggerFrom::Center); // the children, one by one
Gsap::from_props(Props::new().x(-120.0).rotation(-8.0));         // gsap.from
Gsap::to_props(Props::new().x_percent(-60.0));                   // gsap.to
Gsap::from_to(Props::new().scale(0.0), Props::new().scale(1.0)); // gsap.fromTo
```

Presets: `fade`, `fade_up`, `fade_down`, `fade_left`, `fade_right`, `scale`, `zoom_in`, `zoom_out`,
`slide_up`, `slide_down`, `slide_left`, `slide_right`, `rotate_in`, `blur_in`, `flip_x`.

With `from_to`, put each property of `from` also in `to`. GSAP animates only the properties in `to`.

### ScrollTrigger

By default, a tween plays one time when its element top is at 85% of the viewport.
The start is never after the last scroll position, so content at the page end also plays.

```rust
use autumn_plugin_gsap::{Edge, Gsap, Play, ScrollPos, Scrub};

Gsap::fade().play(Play::Load);                                  // no ScrollTrigger
Gsap::fade().start(ScrollPos::new(Edge::Top, Edge::Center));    // start: "top center"
Gsap::fade().replay();                                          // forward on enter, backward on leave back
Gsap::fade().scrub(Scrub::Smooth(ms(500))).pin();               // link to the scroll, pin
Gsap::fade().trigger("#hero").markers();                        // another trigger, debug markers
Gsap::parallax(0.3);                                            // drift 30% of its height
Gsap::scroll_progress();                                        // a page progress bar
```

### Timelines

A `Timeline` plays its `data-gsap` children in document order, as one sequence.
`Position` sets where each step starts. With no position, a step starts at the end of the timeline.

```rust
use autumn_plugin_gsap::{Gsap, Position, Tag, Timeline};

Timeline::new().wrap_in(Tag::Section, html! {
    (Gsap::fade_up().wrap_in(Tag::H1, html! { "Title" }))
    (Gsap::fade().position(Position::Overlap(ms(300))).wrap(html! { p { "Text" } }))
})
```

- The timeline options (`play`, `scrub`, `pin`, `repeat`, ...) apply to the full sequence.
- The `duration` and `ease` of the timeline are defaults for the steps.
- On a step, `init.js` ignores the ScrollTrigger options and `data-gsap-reduced`.

### htmx

- `init.js` scans each `htmx:load` element. Thus server-rendered partials animate with no extra code.
- Before a swap, `init.js` reverts the animations in the swap target. After the swap, it scans the target again.
- On `htmx:beforeCleanupElement`, it reverts the tweens, ScrollTriggers and SplitText of removed content.
- Before htmx saves a page for the Back button, `init.js` reverts the animations. Back then plays them again.
- After `htmx:afterSettle`, it refreshes the ScrollTrigger positions when something changed.

```rust
#[get("/more")]
async fn more() -> Markup {
    Gsap::slide_up().stagger(ms(90)).wrap(rows())
}
```

### JavaScript API

`window.AutumnGsap` has these members:

- `scan(root)`: starts the animations in `root`. It returns the count.
- `revert(root)`: reverts the animations in `root`. It returns the count.
- `parse`: the attribute parsers (for tests and debug).
- `presets`, `version`.

You can also use `window.gsap`, `window.ScrollTrigger` and `window.SplitText` in your own code.

## Attribute reference

The builders write these attributes. You can also write them by hand.
For a bad value, `init.js` writes a console warning and uses the default.

| Attribute | Values | Default |
|---|---|---|
| `data-gsap` | a preset name, `custom`, `parallax`, `scroll-progress` | `fade-up` |
| `data-gsap-from`, `data-gsap-to` | JSON, for example `{"x":-40,"opacity":0}` | — |
| `data-gsap-on` | `scroll`, `load` | `scroll` |
| `data-gsap-delay`, `data-gsap-duration`, `data-gsap-repeat-delay` | seconds, for example `0.25` | `0`, `0.8`, `0` |
| `data-gsap-ease` | a GSAP ease, for example `power3.out`, `back.out(1.7)`, `steps(5)` | `power3.out` |
| `data-gsap-stagger-ease` | a GSAP ease | linear |
| `data-gsap-repeat` | a whole number, or `-1` (no end) | — |
| `data-gsap-yoyo` | bare | — |
| `data-gsap-trigger` | a CSS selector | the element |
| `data-gsap-start` | `top 85%`, `center center`, `-40px 50%` | `top 85%`; `top bottom` with scrub |
| `data-gsap-end` | `bottom top`, `+=500` | ScrollTrigger default; `bottom top` with scrub |
| `data-gsap-toggle-actions` | four of `play pause resume reverse restart reset complete none` | — |
| `data-gsap-once` | `false` | plays one time (not with a pin or scrub) |
| `data-gsap-scrub` | `true`, or seconds | — |
| `data-gsap-pin`, `data-gsap-markers` | bare | — |
| `data-gsap-stagger` | seconds between children | — |
| `data-gsap-stagger-from` | `start`, `center`, `end`, `edges`, `random`, an index | `start` |
| `data-gsap-split` | `chars`, `words`, `lines` | — |
| `data-gsap-split-mask` | bare | — |
| `data-gsap-parallax` | a factor, for example `-0.3` | `0.3` |
| `data-gsap-position` | `<`, `>`, `<0.2`, `+=0.5`, `-=0.2`, `1.5` (only in a timeline) | the end of the timeline |
| `data-gsap-reduced` | `animate` | no animation for reduced motion |
| `data-gsap-timeline` | bare, on the timeline element | — |
| `data-gsap-ignore` | bare, on a region that must not animate | — |

`init.js` sets `data-gsap-init` on each element that it starts. A second scan skips these elements.

`data-gsap-from` and `data-gsap-to` accept only these names with finite numbers:
`x`, `y`, `xPercent`, `yPercent`, `scale`, `scaleX`, `scaleY`, `rotation`, `rotationX`, `rotationY`,
`skewX`, `skewY`, `opacity`, `autoAlpha`.

## Rules for Autumn pages

- Do not use inline `<script>`. The default CSP is `script-src 'self'`.
- Do not use inline `<style>`. Nonce mode blocks it.
- htmx adds an inline indicator style. For a strict CSP, add
  `meta name="htmx-config" content=r#"{"includeIndicatorStyles":false}"#;`.
- Put a stagger on the container. Do not put `data-gsap` on its children too.
- Put SplitText on a heading (`Tag::H1` to `Tag::H6`). SplitText puts the `aria-label` on that element.
  Screen readers can ignore an `aria-label` on a `div` or `p`.
- SplitText hides the split parts from screen readers. Thus `init.js` does not split text with links or controls.
- `x` and `xPercent` start states can make the page wider. Put `overflow-x: clip` on a parent.
  Do not put `overflow: hidden` on `body`. It stops pins.
- `autoAlpha: 0` hides content from screen readers and the Tab key until the tween plays. Prefer `opacity`.

## Motion and accessibility

- Use `data-gsap-reduced="animate"` (`animate_on_reduced_motion()`) only for motion that is necessary.
- A change of the reduced-motion setting after load reverts or starts the animations.
- `Repeat::Forever` is motion with no end. WCAG 2.2.2 requires a pause control for it.

## Demo

```sh
cargo run --example gsap_demo
# open http://127.0.0.1:3000
```

The demo shows a hero timeline with split text, scroll reveals, a pinned scrub timeline,
parallax, a progress bar, and htmx append and replace.

## Tests

| Command | What it tests |
|---|---|
| `cargo test` | Builders, assets, plugin routes, conformance, property tests, golden fixture. |
| `cargo llvm-cov --fail-under-lines 85` | Rust line coverage. |
| `node --test tests/js/*.test.mjs` | `init.js` parsers and scan rules (fake GSAP), golden fixture values. |
| `cargo build --example gsap_demo && npm --prefix tests/e2e ci && npm --prefix tests/e2e test` | The demo in Chromium (Playwright). It also checks the `init.js` line coverage (85% or more). |

After a change to an attribute value, run `UPDATE_GOLDEN=1 cargo test --test golden`.

## Limits

- Each plugin release pins one GSAP version. See `assets/manifest.json`.
- The crate does not include `Flip`, `Draggable`, `ScrollSmoother`, `MorphSVG`, `DrawSVG`, `MotionPath`
  or other GSAP plugins.
- htmx content that you add into an existing stagger container or timeline does not animate.
  Send a new container in the partial.
- `lines` split occurs one time, at scan. A resize does not split the lines again.
- Until the deferred scripts run, the browser shows the end state. On a slow network, this can be
  about one second. Then the content goes to its start state and animates.
- Each element gets its own tween and ScrollTrigger. A scan of 200 elements takes about 80 ms;
  1000 elements take about 650 ms. For long lists, animate a container with a stagger.
- htmx Back plays the animations of the restored page again.

## Untrusted content

`init.js` acts on each `data-gsap*` attribute on the page. User HTML can contain these attributes.
For example, it can move an element over your own UI.

- Remove `data-gsap*` attributes from user HTML when you sanitize it. (DOMPurify keeps `data-*` by default.)
- Or put user HTML in an element with `data-gsap-ignore` or `hx-disable`. `init.js` does not animate in these regions.
- `data-gsap-pin` pins only the element or an element in it. `init.js` does not pin other elements.
- Do not use SplitText on untrusted rich HTML. A revert parses the saved HTML again.

## License

- The crate license expression is `Apache-2.0 AND LicenseRef-GSAP-Standard-License`.
  If you use `cargo-deny`, add `LicenseRef-GSAP-Standard-License` to your allow list after you read the terms.
- The plugin code: Apache-2.0 (see `LICENSE`).
- The GSAP files (`assets/gsap.min.js`, `assets/ScrollTrigger.min.js`, `assets/SplitText.min.js`):
  [GSAP Standard License](https://gsap.com/standard-license). GSAP is free, also for commercial use.
  If a no-code animation tool competes with Webflow, the license does not allow it in that tool.
  Read the full terms before you use GSAP. See `LICENSES/LicenseRef-GSAP-Standard-License.txt`.
