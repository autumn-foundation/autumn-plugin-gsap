# Changelog

## 0.1.0 (2026-10-08)

First release.

- `GsapPlugin` serves GSAP 3.15.0, ScrollTrigger and SplitText through the Autumn 0.8 plugin asset seam.
- `gsap_script()` and `gsap_stylesheet()` write tags with SRI hashes.
- The `Gsap` builder has 15 presets, custom `Props`, stagger and SplitText.
- `Parallax` and `Gsap::scroll_progress()` follow the scroll.
- The `Timeline` builder has typed `Position` values.
- Typed values: `Ease`, `ScrollPos`, `ToggleActions`, `Scrub`, `Repeat`, `Play`, `Tag`.
- `init.js` scans on load and on `htmx:load`.
- `init.js` reverts animations before htmx swaps, on cleanup and before history saves.
- `init.js` checks each attribute value. A bad value gets a warning and the default.
- Elements do not animate for reduced motion. A setting change after load also applies.
- `init.js` skips `data-gsap-ignore` and `hx-disable` regions.
- Tests: unit, property, golden fixture (Rust and JS), and Playwright e2e with an `init.js` coverage gate.
