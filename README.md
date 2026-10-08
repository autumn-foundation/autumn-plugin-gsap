# autumn-plugin-gsap

[GSAP](https://gsap.com) animations for [Autumn](https://github.com/autumn-foundation/autumn) 0.8 apps.
You write Rust and Maud. You need no npm, no bundler and no JavaScript.

- Vendors GSAP 3.15.0, ScrollTrigger and SplitText.
- Serves them through the Autumn plugin asset seam: hashed URLs, SRI, immutable cache.
- Typed builders: `Gsap`, `Timeline`, `Ease`, `ScrollPos`, `Position`, `Props`.
- Works with htmx: new content animates, removed content is reverted.
- Respects `prefers-reduced-motion`. Works with a strict CSP.

## Quickstart

1. Add the plugin.

   ```rust
   use autumn_plugin_gsap::GsapPlugin;

   autumn_web::app().plugin(GsapPlugin::new()).run().await;
   ```

2. Put the scripts in the page `<head>`. The tags are deferred and have SRI hashes.

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

`gsap_stylesheet()` is necessary only for `Gsap::scroll_progress()`.

## Features

### Tweens

```rust
use autumn_plugin_gsap::{Gsap, Props, Repeat, StaggerFrom};

Gsap::zoom_in().repeat(Repeat::Forever).yoyo();                 // a preset
Gsap::scale().stagger(ms(80)).stagger_from(StaggerFrom::Center); // the children, one by one
Gsap::from_props(Props::new().x(-120.0).rotation(-8.0));         // gsap.from
Gsap::to_props(Props::new().x_percent(-60.0));                   // gsap.to
Gsap::from_to(Props::new().scale(0.0), Props::new().scale(1.0)); // gsap.fromTo
```

Presets: `fade`, `fade_up`, `fade_down`, `fade_left`, `fade_right`, `scale`, `zoom_in`, `zoom_out`,
`slide_up`, `slide_down`, `slide_left`, `slide_right`, `rotate_in`, `blur_in`, `flip_x`.

### ScrollTrigger

By default, a tween plays one time when its element top is at 85% of the viewport.

```rust
use autumn_plugin_gsap::{Edge, Gsap, Play, ScrollPos, Scrub};

Gsap::fade().play(Play::Load);                                  // no ScrollTrigger
Gsap::fade().start(ScrollPos::new(Edge::Top, Edge::Center));    // start: "top center"
Gsap::fade().replay();                                          // play again on each entry
Gsap::fade().scrub(Scrub::Smooth(ms(500))).pin();               // link to the scroll, pin
Gsap::fade().trigger("#hero").markers();                        // another trigger, debug markers
Gsap::parallax(0.3);                                            // drift 30% of its height
Gsap::scroll_progress();                                        // a page progress bar
```

### Timelines

A `Timeline` plays its `data-gsap` children in document order, as one sequence.
`Position` sets where each step starts.

```rust
use autumn_plugin_gsap::{Gsap, Position, Tag, Timeline};

Timeline::new().wrap_in(Tag::Section, html! {
    (Gsap::fade_up().wrap_in(Tag::H1, html! { "Title" }))
    (Gsap::fade().position(Position::Overlap(ms(300))).wrap(html! { p { "Text" } }))
})
```

The timeline options (`play`, `scrub`, `pin`, `repeat`, ...) apply to the full sequence.
Its `duration` and `ease` are defaults for the steps.

### htmx

`init.js` scans each `htmx:load` element. Thus server-rendered partials animate with no extra code.
On `htmx:beforeCleanupElement`, it reverts the tweens, ScrollTriggers and SplitText of the removed content.
After `htmx:afterSettle`, it refreshes ScrollTrigger positions.

```rust
#[get("/more")]
async fn more() -> Markup {
    Gsap::slide_up().stagger(ms(90)).wrap(rows())
}
```

### JavaScript API

`window.AutumnGsap` has:

- `scan(root)`: starts the animations in `root`. Returns the count.
- `revert(root)`: reverts the animations in `root`. Returns the count.
- `parse`: the attribute parsers (for tests and debug).
- `presets`, `version`.

`window.gsap`, `window.ScrollTrigger` and `window.SplitText` are also there for custom code.

## Attribute reference

The builders write these attributes. You can also write them by hand.
`init.js` ignores a bad value, writes a console warning, and uses the default.

| Attribute | Values | Default |
|---|---|---|
| `data-gsap` | a preset name, `custom`, `parallax`, `scroll-progress` | `fade-up` |
| `data-gsap-from`, `data-gsap-to` | JSON, for example `{"x":-40,"opacity":0}` | — |
| `data-gsap-on` | `scroll`, `load` | `scroll` |
| `data-gsap-delay`, `data-gsap-duration`, `data-gsap-repeat-delay` | seconds, for example `0.25` | `0`, `0.8`, `0` |
| `data-gsap-ease`, `data-gsap-stagger-ease` | a GSAP ease, for example `power3.out`, `back.out(1.7)`, `steps(5)` | `power3.out` |
| `data-gsap-repeat` | a whole number, or `-1` (no end) | — |
| `data-gsap-yoyo` | bare | — |
| `data-gsap-trigger` | a CSS selector | the element |
| `data-gsap-start`, `data-gsap-end` | `top 85%`, `bottom top`, `-40px 50%`, `+=500` | `top 85%` |
| `data-gsap-toggle-actions` | four of `play pause resume reverse restart reset complete none` | — |
| `data-gsap-once` | `false` | plays one time |
| `data-gsap-scrub` | `true`, or seconds | — |
| `data-gsap-pin`, `data-gsap-markers` | bare | — |
| `data-gsap-stagger` | seconds between children | — |
| `data-gsap-stagger-from` | `start`, `center`, `end`, `edges`, `random`, an index | `start` |
| `data-gsap-split` | `chars`, `words`, `lines` | — |
| `data-gsap-split-mask` | bare | — |
| `data-gsap-parallax` | a factor, for example `-0.3` | `0.3` |
| `data-gsap-position` | `<`, `>`, `<0.2`, `+=0.5`, `-=0.2`, `1.5` | `>` |
| `data-gsap-reduced` | `animate` | skip for reduced motion |
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
- Put SplitText on the text element (for example `Tag::H1`). SplitText puts the `aria-label` on that element.

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
| `node --test tests/js/*.test.mjs` | The `init.js` parsers and the golden fixture. |
| `cargo build --example gsap_demo && npm --prefix tests/e2e ci && npm --prefix tests/e2e test` | The demo in Chromium (Playwright). |

After a change to an attribute value, run `UPDATE_GOLDEN=1 cargo test --test golden`.

## Limits

- The GSAP version is pinned per plugin release. See `assets/manifest.json`.
- `Flip`, `Draggable`, `ScrollSmoother`, `MorphSVG`, `DrawSVG`, `MotionPath` and other GSAP plugins are not included.
- htmx content that you append into an existing stagger container does not animate.
  Send a new stagger container in the partial.
- `lines` split happens one time, at scan. A resize does not split the lines again.
- Until the deferred scripts run, the browser can show the end state for a short time.

## Untrusted content

`init.js` acts on each `data-gsap*` attribute on the page. User HTML can contain these attributes.
For example, it can move an element over your own UI.

- Remove `data-gsap*` attributes from user HTML when you sanitize it. (DOMPurify keeps `data-*` by default.)
- Or put user HTML in an element with `data-gsap-ignore` or `hx-disable`. `init.js` does not animate in these regions.
- `data-gsap-pin` pins only the element or an element in it. A pin of another element is refused.
- Do not use SplitText on untrusted rich HTML. A revert parses the saved HTML again.

## License

- The crate license expression is `Apache-2.0 AND LicenseRef-GSAP-Standard-License`.
  If you use `cargo-deny`, add `LicenseRef-GSAP-Standard-License` to your allow list after you read the terms.
- The plugin code: Apache-2.0 (see `LICENSE`).
- The vendored GSAP files (`assets/gsap.min.js`, `assets/ScrollTrigger.min.js`, `assets/SplitText.min.js`):
  [GSAP Standard License](https://gsap.com/standard-license). GSAP is free, also for commercial use.
  The license does not permit tools that let users build visual animations with no code,
  if they compete with Webflow. Read the full terms before you use GSAP.
  See `LICENSES/LicenseRef-GSAP-Standard-License.txt`.
