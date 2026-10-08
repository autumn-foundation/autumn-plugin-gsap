# Plan: autumn-plugin-gsap 0.1.0

Status: done. Target: autumn-web 0.8.0, GSAP 3.15.0.
The AC evidence table is in the PR description.
Prior art: [autumn-plugin-motion](https://github.com/madmax983/autumn-plugin-motion).

## 1. Goal

Give Autumn apps GSAP animations with no npm and no bundler.
The Rust code describes the animation. The page gets `data-gsap*` attributes.
One script (`init.js`) reads the attributes and calls GSAP.

## 2. Brainstorm (what can the plugin do?)

- Vendor `gsap.min.js`, `ScrollTrigger.min.js` and `SplitText.min.js`.
  All GSAP plugins are free since 3.13.
- Serve the files through the Autumn 0.8 `PluginAssets` seam.
  The framework gives hashed URLs, SRI, ETag and immutable cache.
- Typed Rust builder `Gsap` with presets (`fade_up`, `zoom_in`, ...).
- Typed `Ease` that writes GSAP ease strings (`power3.out`, `back.out(1.7)`).
- Use `std::time::Duration` for all times. No raw milliseconds.
- ScrollTrigger: `start`, `end`, `scrub`, `pin`, `markers`, `toggleActions`.
- Stagger: `each`, `from`, `ease`.
- SplitText: animate chars, words or lines.
- Timeline: a container plays its children in sequence, with GSAP position values.
- Custom tweens: typed `Props` for `from` and `to` values.
- Parallax and a scroll progress bar.
- htmx: scan new content on `htmx:load`.
- htmx: kill tweens and ScrollTriggers when htmx removes content.
- `window.AutumnGsap` with `scan`, `revert` and the attribute parsers.
- Browser tests with Playwright against a real demo app.

## 3. Reverse brainstorm (how can the plugin fail?)

| Way to fail | Prevention |
|---|---|
| Inline `<script>` or `<style>` breaks the Autumn CSP. | Serve all JS and CSS as files. The e2e test fails on a CSP error. |
| htmx removes content but ScrollTriggers stay. Memory leaks. Old triggers fire. | Keep one `gsap.context` per element. Revert it on `htmx:beforeCleanupElement`. The e2e test counts triggers. |
| ScrollTrigger positions go stale after a swap. | Call `ScrollTrigger.refresh()` after `htmx:afterSettle`. |
| A typo in an attribute throws and stops the scan. | Validate each value. Ignore a bad value and use the default. |
| A JSON attribute sets `onComplete` or other callbacks. | Accept only listed property names with finite numbers. |
| Rust writes a value that `init.js` does not accept. | One golden fixture. Rust makes it. The JS test parses it. |
| Content stays hidden when GSAP does not load. | `init.js` stops when `window.gsap` is missing. Nothing gets hidden before GSAP runs. |
| Users with reduced motion get animations. | Skip each element unless it has `data-gsap-reduced="animate"`. |
| A second scan animates an element again. | Mark each element with `data-gsap-init`. |
| A timeline child also animates alone. | Scan timelines first. They claim their children. |
| SplitText removes the text for screen readers. | Use SplitText `aria: "auto"`. The e2e test checks `aria-label`. |
| The vendored bytes change by accident. | Pin the `sha384` of each upstream file. A test checks it. |
| The GSAP license is ignored. | Keep the upstream headers. Record the license in the manifest and README. |
| The plugin forces features on the host app. | Use `PluginAssets::from_files`. Do not enable `embed-assets`. |

## 4. Six thinking hats

- **White (facts):** GSAP 3.15.0 is on npm. It uses the GSAP Standard License, not MIT.
  Autumn 0.8 ships htmx 2 and the `PluginAssets` seam. The default CSP is `script-src 'self'`.
- **Red (feelings):** Users want "add one line, it moves". The API must feel like GSAP, not like a new tool.
  So attribute names follow GSAP names (`scrub`, `pin`, `start`, `toggle-actions`).
- **Black (risks):** The license does not allow no-code visual builders. This crate is a code API. The license allows it.
  The README tells users about the terms. Bundle size is about 125 KB of JS.
  `gsap_script()` loads ScrollTrigger and SplitText too. That cost is small and keeps setup to one call.
- **Yellow (benefits):** Timelines and ScrollTrigger are stronger than the Motion plugin.
  The htmx cleanup removes a leak that the Motion plugin has.
  Browser tests close the "no live-browser verification" gap of the prior art.
- **Green (ideas):** Golden fixture for Rust/JS lockstep. `AutumnGsap.parse` for tests and debug.
  Typed `Position` for timelines. Later: `Flip` for htmx swaps, `Draggable`, `ScrollSmoother`.
- **Blue (process):** Red, green, refactor for each module. Then docs, CI, e2e.
  Then a multi-angle agent review. Then an AC evidence table.

## 5. Scope

In scope: section 2.
Out of scope for 0.1.0: `Flip`, `Draggable`, `ScrollSmoother`, `MorphSVG`, `DrawSVG`,
`MotionPath`, `Observer`, `CustomEase`. Users can call `window.gsap` for these after they load the files.

## 6. Acceptance criteria

1. `GsapPlugin` installs the bundle. All files serve under `/static/_plugins/gsap/` with hashed, immutable URLs, ETag/304 and SRI.
2. The plugin does not serve `manifest.json`. The bundle does not force `embed-assets` on the host.
3. A test pins the `sha384` of each vendored upstream file. The manifest records version, source URL and license.
4. `gsap_script()` emits deferred, SRI tags in this order: gsap, ScrollTrigger, SplitText, init. `gsap_stylesheet()` emits the CSS link.
5. The `Gsap` builder covers presets, `Duration` times, typed `Ease`, repeat/yoyo, ScrollTrigger options, stagger, SplitText, custom `Props`, parallax and the progress bar.
6. The `Timeline` builder plays children in sequence. It supports typed `Position` values.
7. Each builder option maps to one `data-gsap-*` attribute. Defaults write no attribute.
8. `init.js` scans on load and on `htmx:load`. It reverts tweens and ScrollTriggers on `htmx:beforeCleanupElement`.
9. `init.js` validates every attribute. A bad value does not stop the scan.
10. Reduced motion skips animation. `data-gsap-reduced="animate"` opts in.
11. Rust and JS agree on attribute values (golden fixture, tested on both sides).
12. The plugin passes the Autumn plugin conformance check. Asset routes are public plugin routes.
13. A runnable demo shows each feature. It works under the default CSP with no console errors.
14. Playwright e2e tests prove the browser behavior (load, scroll, split, timeline, htmx, cleanup, reduced motion).
15. CI runs fmt, clippy (pedantic, nursery), tests, docs, coverage (>= 85% lines), JS tests and e2e.
16. Docs (README, ADR, rustdoc, CLAUDE.md) are short and use ASD-STE100 style.

## 7. Test plan

| Layer | Tool | What |
|---|---|---|
| Rust unit | `cargo test` | Builders, assets, plugin routes, conformance, script tags. |
| Rust property | `proptest` | Each `Ease` and time value matches the grammar that `init.js` accepts. |
| Golden | Rust + Node | `tests/fixtures/attributes.json`: Rust makes it, JS parses it. |
| JS unit | `node --test` | Parsers in `init.js`, loaded in a `vm` sandbox. |
| Browser e2e | Playwright | Real demo app in Chromium. |
