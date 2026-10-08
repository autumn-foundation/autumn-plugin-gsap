# CLAUDE.md

Autumn plugin for GSAP. Rust crate plus one JS file (`assets/init.js`).

## Layout

- `src/assets.rs`: `GSAP_ASSETS` bundle, version and upstream hash pins.
- `src/plugin.rs`: `GsapPlugin` (installs the bundle).
- `src/script.rs`: `gsap_script()`, `gsap_stylesheet()`.
- `src/tween.rs`, `src/timeline.rs`, `src/options.rs`: the builders. `options.rs` has the shared setters.
- `src/ease.rs`, `src/scroll.rs`, `src/props.rs`, `src/fmt.rs`, `src/attrs.rs`: typed values and output.
- `src/grammar.rs` (tests only): the value regexes. They must equal the `RE_*` lines in `init.js`.
- `assets/init.js`: the scanner. `assets/gsap.css`: progress bar style. `assets/manifest.json`: provenance (not served).
- `tests/golden.rs` + `tests/fixtures/attributes.json`: the Rust/JS contract.
- `tests/js/`: `node:test` parser tests. `tests/e2e/`: Playwright tests against `examples/gsap_demo.rs`.

## Rules

- Write tests first (red, green, refactor).
- A new attribute needs: a builder method, a parser in `init.js` (`PARSERS`), a golden case, docs in README.
- After an attribute change, run `UPDATE_GOLDEN=1 cargo test --test golden`.
- Do not change the vendored GSAP files. To upgrade, replace them, then update the pins and `manifest.json`.
- No inline scripts or styles in the demo. The e2e tests run it with a strict CSP.
- Docs and comments: short, active voice, ASD-STE100 style.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
node --test tests/js/*.test.mjs
cargo build --example gsap_demo && npm --prefix tests/e2e ci && npm --prefix tests/e2e test
cargo llvm-cov --fail-under-lines 85
```
