# Changelog

## 0.1.0 (2026-10-08)

First release.

- `GsapPlugin` serves GSAP 3.15.0, ScrollTrigger and SplitText through the Autumn 0.8 plugin asset seam.
- `gsap_script()` and `gsap_stylesheet()` write deferred, SRI-hashed tags.
- `Gsap` builder: 15 presets, custom `Props`, parallax, scroll progress bar, stagger, SplitText.
- `Timeline` builder with typed `Position` values.
- Typed `Ease`, `ScrollPos`, `ToggleActions`, `Scrub`, `Repeat`, `Play`, `Tag`.
- `init.js`: scans on load and `htmx:load`, reverts on `htmx:beforeCleanupElement`, validates all values,
  respects reduced motion, exposes `window.AutumnGsap`.
- Tests: unit, property, golden fixture (Rust and JS), Playwright e2e (default and strict CSP).
