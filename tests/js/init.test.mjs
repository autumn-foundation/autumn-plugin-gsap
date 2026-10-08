// Unit tests for the attribute parsers in assets/init.js.
// Run: node --test tests/js/*.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const root = new URL("../../", import.meta.url);
const source = readFileSync(new URL("assets/init.js", root), "utf8");
const fixture = JSON.parse(
  readFileSync(new URL("tests/fixtures/attributes.json", root), "utf8"),
);

// Loads init.js in a sandbox. `extra` adds globals (for example a fake gsap).
function load(extra = {}) {
  const listeners = {};
  const sandbox = {
    console,
    document: {
      readyState: "complete",
      documentElement: {},
      addEventListener(type, fn) {
        listeners[type] = fn;
      },
      querySelectorAll() {
        return [];
      },
    },
    ...extra,
  };
  sandbox.window = sandbox;
  vm.runInNewContext(source, sandbox);
  return { api: sandbox.AutumnGsap, listeners };
}

const { api } = load();
const P = api.parse;

test("init.js runs with no gsap and exposes the API", () => {
  assert.equal(typeof api.scan, "function");
  assert.equal(typeof api.revert, "function");
  assert.equal(api.version, "0.1.0");
  assert.doesNotThrow(() => api.scan());
});

test("init.js listens for htmx events", () => {
  const { listeners } = load();
  for (const type of ["htmx:load", "htmx:beforeCleanupElement", "htmx:afterSettle"]) {
    assert.equal(typeof listeners[type], "function", type);
  }
});

test("secs accepts non-negative decimals only", () => {
  assert.equal(P.secs("0.25"), 0.25);
  assert.equal(P.secs("2"), 2);
  for (const bad of ["", "-1", "abc", "1e3", "0.5s", " 1"]) {
    assert.equal(P.secs(bad), null, bad);
  }
});

test("ease accepts GSAP ease strings only", () => {
  for (const ok of [
    "none",
    "power1.in",
    "power4.inOut",
    "sine.out",
    "expo.out",
    "circ.in",
    "bounce.out",
    "back.out",
    "back.out(1.7)",
    "elastic.inOut(1,0.3)",
    "steps(5)",
  ]) {
    assert.equal(P.ease(ok), ok, ok);
  }
  for (const bad of ["", "power5.out", "linear", "ease-out", "power2.out;x", "back.out(a)"]) {
    assert.equal(P.ease(bad), null, bad);
  }
});

test("repeat accepts -1 and whole numbers", () => {
  assert.equal(P.repeat("-1"), -1);
  assert.equal(P.repeat("0"), 0);
  assert.equal(P.repeat("3"), 3);
  for (const bad of ["-2", "1.5", "x", ""]) {
    assert.equal(P.repeat(bad), null, bad);
  }
});

test("scrollPos accepts two edges or a distance", () => {
  for (const ok of ["top 85%", "center center", "bottom top", "-40px 75.5%", "+=500", "+=50%"]) {
    assert.equal(P.scrollPos(ok), ok, ok);
  }
  for (const bad of ["top", "left right", "top 85", "+=-5", "top  85%"]) {
    assert.equal(P.scrollPos(bad), null, bad);
  }
});

test("toggleActions needs four known actions", () => {
  assert.equal(P.toggleActions("play none none reverse"), "play none none reverse");
  for (const bad of ["play none", "play none none jump", "play  none none none"]) {
    assert.equal(P.toggleActions(bad), null, bad);
  }
});

test("scrub accepts true or seconds", () => {
  assert.equal(P.scrub("true"), true);
  assert.equal(P.scrub("0.5"), 0.5);
  assert.equal(P.scrub("yes"), null);
});

test("staggerFrom accepts words and indexes", () => {
  for (const w of ["start", "center", "end", "edges", "random"]) {
    assert.equal(P.staggerFrom(w), w);
  }
  assert.equal(P.staggerFrom("4"), 4);
  assert.equal(P.staggerFrom("middle"), null);
  assert.equal(P.staggerFrom("-1"), null);
});

test("split accepts one unit", () => {
  for (const u of ["chars", "words", "lines"]) {
    assert.equal(P.split(u), u);
  }
  assert.equal(P.split("letters"), null);
  assert.equal(P.split("chars,words"), null);
});

test("position accepts GSAP position values", () => {
  for (const ok of ["<", ">", "<0.25", "+=0.5", "-=0.2", "1.5"]) {
    assert.equal(P.position(ok), ok, ok);
  }
  for (const bad of ["", "label", "+=x", "<<"]) {
    assert.equal(P.position(bad), null, bad);
  }
});

test("factor accepts finite numbers", () => {
  assert.equal(P.factor("0.3"), 0.3);
  assert.equal(P.factor("-0.25"), -0.25);
  assert.equal(P.factor("NaN"), null);
  assert.equal(P.factor("Infinity"), null);
});

test("vars keeps listed properties with finite numbers only", () => {
  assert.deepEqual({ ...P.vars('{"x":-40,"opacity":0}') }, { x: -40, opacity: 0 });
  assert.deepEqual(
    { ...P.vars('{"x":1,"onComplete":"alert(1)","y":"10px","scale":2}') },
    { x: 1, scale: 2 },
  );
  for (const bad of ["", "nope", "[1]", "null", '{"onStart":1}', '{"__proto__":{"x":1}}']) {
    assert.equal(P.vars(bad), null, bad);
  }
});

test("on and once accept their values only", () => {
  assert.equal(P.on("load"), "load");
  assert.equal(P.on("scroll"), "scroll");
  assert.equal(P.on("click"), null);
  assert.equal(P.once("false"), false);
  assert.equal(P.once("true"), true);
  assert.equal(P.once("no"), null);
});

test("every attribute in the golden fixture parses", () => {
  let count = 0;
  for (const [name, attrs] of Object.entries(fixture)) {
    for (const [attr, value] of attrs) {
      assert.ok(P.known(attr), `${name}: unknown attribute ${attr}`);
      const parsed = P.attr(attr, value);
      assert.notEqual(parsed, null, `${name}: ${attr}="${value}"`);
      assert.notEqual(parsed, undefined, `${name}: ${attr}="${value}"`);
      count += 1;
    }
  }
  assert.ok(count > 100, `fixture has ${count} attributes`);
});

test("every Rust preset has keyframes in init.js", () => {
  const names = Object.values(fixture)
    .flat()
    .filter(([attr]) => attr === "data-gsap")
    .map(([, value]) => value);
  assert.ok(names.length > 15);
  for (const name of names) {
    assert.ok(api.presets.includes(name), name);
  }
});

test("scan does nothing without gsap and with reduced motion", () => {
  const { api: noGsap } = load();
  assert.equal(noGsap.scan(), 0);
});
