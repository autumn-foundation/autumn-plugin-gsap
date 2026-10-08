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
  for (const bad of ["", "power5.out", "linear", "ease-out", "power2.out;x", "back.out(a)", "steps(0)"]) {
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

// The value that init.js must read for each attribute in the fixture.
const NUMBERS = new Set([
  "data-gsap-delay",
  "data-gsap-duration",
  "data-gsap-repeat-delay",
  "data-gsap-stagger",
  "data-gsap-parallax",
  "data-gsap-repeat",
]);
const FLAGS = new Set([
  "data-gsap-yoyo",
  "data-gsap-pin",
  "data-gsap-markers",
  "data-gsap-split-mask",
  "data-gsap-timeline",
]);

function expected(attr, value) {
  if (NUMBERS.has(attr)) return Number(value);
  if (FLAGS.has(attr)) return true;
  if (attr === "data-gsap-from" || attr === "data-gsap-to") return JSON.parse(value);
  if (attr === "data-gsap-once") return value === "true";
  if (attr === "data-gsap-scrub") return value === "true" ? true : Number(value);
  if (attr === "data-gsap-stagger-from" && /^\d+$/.test(value)) return Number(value);
  return value;
}

// Copies a sandbox object into this realm, so deepEqual compares only values.
const norm = (v) => (v && typeof v === "object" ? { ...v } : v);

test("every attribute in the golden fixture parses to its value", () => {
  let count = 0;
  for (const [name, attrs] of Object.entries(fixture)) {
    for (const [attr, value] of attrs) {
      assert.ok(P.known(attr), `${name}: unknown attribute ${attr}`);
      const parsed = P.attr(attr, value);
      assert.deepEqual(norm(parsed), expected(attr, value), `${name}: ${attr}="${value}"`);
      count += 1;
    }
  }
  assert.ok(count > 100, `fixture has ${count} attributes`);
});

test("init.js has no preset that Rust does not know", () => {
  const rust = Object.keys(fixture)
    .filter((name) => name.startsWith("preset-"))
    .map((name) => name.slice("preset-".length));
  const special = ["custom", "parallax", "scroll-progress"];
  const js = api.presets.filter((p) => !special.includes(p));
  assert.deepEqual([...js].sort(), [...rust].sort());
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

// A small fake element: attributes, children and the selector calls that init.js uses.
function fakeEl(attrs) {
  const map = new Map(Object.entries(attrs));
  return {
    nodeType: 1,
    children: [],
    getAttribute: (n) => (map.has(n) ? map.get(n) : null),
    hasAttribute: (n) => map.has(n),
    setAttribute: (n, v) => map.set(n, String(v)),
    removeAttribute: (n) => map.delete(n),
    closest: () => null,
    matches: () => false,
    querySelector: () => null,
    querySelectorAll: () => [],
    contains: () => false,
  };
}

// A fake gsap that records each call.
function fakeGsap() {
  const calls = [];
  const gsap = {
    calls,
    registerPlugin() {},
    utils: { toArray: (t) => (Array.isArray(t) ? t : [t]) },
    getProperty: () => 1,
    context(fn) {
      fn();
      return { revert: () => calls.push(["revert"]) };
    },
    from: (t, v) => calls.push(["from", v]),
    to: (t, v) => calls.push(["to", v]),
    fromTo: (t, a, b) => calls.push(["fromTo", a, b]),
  };
  return gsap;
}

function loadWith(el, { reduce }) {
  const gsap = fakeGsap();
  const { api } = load({
    gsap,
    ScrollTrigger: function ScrollTrigger() {},
    matchMedia: () => ({ matches: reduce }),
  });
  const root = {
    querySelectorAll: (sel) => (sel === "[data-gsap]" ? [el] : []),
    matches: () => false,
  };
  return { api, gsap, root };
}

test("scan does nothing without gsap", () => {
  const { api: noGsap } = load();
  assert.equal(noGsap.scan(), 0);
});

test("scan skips elements under reduced motion, unless they opt in", () => {
  const el = fakeEl({ "data-gsap": "fade", "data-gsap-on": "load" });
  const { api, root } = loadWith(el, { reduce: true });
  assert.equal(api.scan(root), 0);
  assert.equal(el.hasAttribute("data-gsap-init"), false);
  el.setAttribute("data-gsap-reduced", "animate");
  assert.equal(api.scan(root), 1);
});

test("scan writes the default ScrollTrigger vars", () => {
  const el = fakeEl({ "data-gsap": "fade-up" });
  el.getBoundingClientRect = () => ({ top: 2000 });
  const { api, gsap, root } = loadWith(el, { reduce: false });
  assert.equal(api.scan(root), 1);
  const [kind, vars] = gsap.calls[0];
  assert.equal(kind, "from");
  assert.equal(vars.opacity, 0);
  assert.equal(vars.y, 32);
  assert.equal(vars.duration, 0.8);
  assert.equal(vars.ease, "power3.out");
  assert.equal(vars.scrollTrigger.once, true);
  assert.equal(typeof vars.scrollTrigger.start, "function");
  assert.equal(api.revert(root), 0, "revert needs a marked element in the root");
});

test("scrub, pin and toggle actions change the ScrollTrigger vars", () => {
  const el = fakeEl({
    "data-gsap": "fade",
    "data-gsap-scrub": "0.5",
    "data-gsap-pin": "",
    "data-gsap-toggle-actions": "play none none reverse",
  });
  const { api, gsap, root } = loadWith(el, { reduce: false });
  api.scan(root);
  const st = gsap.calls[0][1].scrollTrigger;
  assert.equal(st.scrub, 0.5);
  assert.equal(st.start, "top bottom");
  assert.equal(st.end, "bottom top");
  assert.equal(st.pin, true);
  assert.equal(st.once, undefined);
  assert.equal(st.toggleActions, undefined, "scrub ignores toggle actions");
});

test("blur-in and flip-x use fromTo with full end states", () => {
  for (const kind of ["blur-in", "flip-x"]) {
    const el = fakeEl({ "data-gsap": kind, "data-gsap-on": "load" });
    const { api, gsap, root } = loadWith(el, { reduce: false });
    api.scan(root);
    const [call, from, to] = gsap.calls[0];
    assert.equal(call, "fromTo", kind);
    for (const key of Object.keys(from)) {
      assert.ok(key in to, `${kind}: the end state sets ${key}`);
    }
    // The end opacity comes from the element (here 1, from the fake getProperty).
    assert.equal(typeof to.opacity, "function", kind);
    assert.equal(to.opacity(0), 1, kind);
  }
});

test("data-gsap-ignore is a known flag", () => {
  assert.equal(P.attr("data-gsap-ignore", ""), true);
});
