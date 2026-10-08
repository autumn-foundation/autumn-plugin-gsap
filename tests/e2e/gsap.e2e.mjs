// Browser tests: run the demo app and check the animations in Chromium.
//
// Build the demo first: `cargo build --example gsap_demo`.
// Then run: `npm --prefix tests/e2e test`.
// DEMO_BIN sets another demo binary. CHROMIUM sets another browser binary.
import { after, before, describe, test } from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { chromium } from "playwright";

const ROOT = new URL("../../", import.meta.url).pathname;
const BIN = process.env.DEMO_BIN || `${ROOT}target/debug/examples/gsap_demo`;
const STRICT_CSP = "default-src 'self'; style-src 'self'; script-src 'self'";

const servers = [];
let browser;

async function startServer(port, env = {}) {
  const child = spawn(BIN, [], {
    cwd: ROOT,
    env: { ...process.env, AUTUMN_SERVER__PORT: String(port), ...env },
    stdio: "ignore",
  });
  servers.push(child);
  const url = `http://127.0.0.1:${port}`;
  for (let i = 0; i < 100; i++) {
    try {
      const res = await fetch(url);
      if (res.ok) {
        return url;
      }
    } catch {
      // The server is not ready.
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`demo did not start on ${url}`);
}

// Opens the page and records console errors, page errors and CSP violations.
async function open(url, options = {}) {
  const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, ...options });
  const page = await context.newPage();
  const errors = [];
  const warnings = [];
  // Playwright gives the raw format string and arguments, joined by spaces.
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
    if (m.type() === "warning") warnings.push(m.text());
  });
  page.on("pageerror", (e) => errors.push(String(e)));
  await page.addInitScript(() => {
    window.__csp = [];
    document.addEventListener("securitypolicyviolation", (e) => {
      window.__csp.push(`${e.violatedDirective} ${e.blockedURI} ${e.sourceFile}:${e.lineNumber}`);
    });
  });
  await page.goto(url);
  await page.waitForFunction(() => window.AutumnGsap && window.htmx);
  return { page, context, errors, warnings };
}

const opacity = (page, sel) =>
  page.$eval(sel, (el) => Number(getComputedStyle(el).opacity));

async function waitOpaque(page, sel) {
  await page.waitForFunction(
    (s) => {
      const el = document.querySelector(s);
      return el && Number(getComputedStyle(el).opacity) === 1;
    },
    sel,
    { timeout: 5000 },
  );
}

// Scrolls so the element top is at the middle of the viewport.
async function scrollTo(page, sel) {
  await page.$eval(sel, (el) => {
    const top = el.getBoundingClientRect().top + window.scrollY;
    window.scrollTo(0, top - window.innerHeight / 2);
  });
}

// ScrollTriggers whose trigger is not in the document (a leak).
const detached = (page) =>
  page.evaluate(
    () =>
      window.ScrollTrigger.getAll().filter((st) => st.trigger && !st.trigger.isConnected).length,
  );

let url;
let strictUrl;

before(async () => {
  assert.ok(existsSync(BIN), `build the demo first: cargo build --example gsap_demo (${BIN})`);
  const base = 41000 + (process.pid % 1000) * 2;
  url = await startServer(base);
  strictUrl = await startServer(base + 1, {
    AUTUMN_SECURITY__HEADERS__CONTENT_SECURITY_POLICY: STRICT_CSP,
  });
  browser = await chromium.launch(
    process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {},
  );
});

after(async () => {
  await browser?.close();
  for (const s of servers) {
    s.kill();
  }
});

describe("default CSP", () => {
  test("loads GSAP, plugins and init.js with no errors", async () => {
    const { page, context, errors } = await open(url);
    const info = await page.evaluate(() => ({
      gsap: window.gsap.version,
      st: typeof window.ScrollTrigger,
      split: typeof window.SplitText,
      api: window.AutumnGsap.version,
      csp: window.__csp,
    }));
    assert.deepEqual(info, {
      gsap: "3.15.0",
      st: "function",
      split: "function",
      api: "0.1.0",
      csp: [],
    });
    assert.deepEqual(errors, []);
    await context.close();
  });

  test("the hero timeline plays on load", async () => {
    const { page, context } = await open(url);
    const marked = await page.$$eval(
      "#hero, #hero [data-gsap]",
      (els) => els.every((el) => el.hasAttribute("data-gsap-init")),
    );
    assert.ok(marked, "init.js marks the timeline and steps");
    // The lede paragraphs start hidden and play after the heading (a timeline, not in parallel).
    const early = await page.evaluate(() => ({
      lede: Number(getComputedStyle(document.querySelector("#lede p")).opacity),
      start: window.gsap.getTweensOf(document.querySelectorAll("#lede p"))[0].startTime(),
    }));
    assert.ok(early.lede < 0.05, `lede starts hidden: ${early.lede}`);
    assert.ok(early.start > 0.3, `lede starts after the heading: ${early.start}`);
    await waitOpaque(page, "#lede p:last-child");
    await context.close();
  });

  test("split text keeps an aria-label and hides the parts", async () => {
    const { page, context } = await open(url);
    const split = await page.$eval("#split", (el) => ({
      label: el.getAttribute("aria-label"),
      tag: el.tagName,
      parts: el.querySelectorAll("[aria-hidden=true]").length,
    }));
    assert.equal(split.tag, "H1");
    const charOpacity = await page.$eval(
      "#split .gsap-char",
      (el) => Number(getComputedStyle(el).opacity) + Number(window.gsap.getProperty(el, "yPercent")),
    );
    assert.ok(charOpacity !== 1, "the chars animate");
    assert.equal(split.label, "Server HTML, GSAP motion.");
    assert.ok(split.parts >= 20, `chars are split: ${split.parts}`);
    await context.close();
  });

  test("a card is hidden below the fold and reveals on scroll", async () => {
    const { page, context } = await open(url);
    for (const sel of ["#card-fade-up", "#card-custom", "#card-raw"]) {
      assert.ok((await opacity(page, sel)) < 0.05, `${sel} starts hidden`);
    }
    const vars = await page.evaluate(() => {
      const st = window.ScrollTrigger.getAll().find((t) => t.trigger.id === "card-fade-up");
      const chips = window.gsap.getTweensOf(document.querySelectorAll("#chips li"))[0];
      return {
        once: st.vars.once,
        clamped: st.start < window.ScrollTrigger.maxScroll(window),
        each: chips.vars.stagger.each,
      };
    });
    assert.equal(vars.once, true, "plays one time by default");
    assert.equal(vars.clamped, true, "the default start is before the page end");
    assert.equal(vars.each, 0.08);
    await scrollTo(page, "#card-fade-up");
    await waitOpaque(page, "#card-fade-up");
    await scrollTo(page, "#card-custom");
    await waitOpaque(page, "#card-custom");
    await page.waitForFunction(
      () => getComputedStyle(document.querySelector("#card-custom")).transform
        .match(/^(none|matrix\(1, 0, 0, 1, 0, 0\))$/),
    );
    await scrollTo(page, "#card-raw");
    await waitOpaque(page, "#card-raw");
    await context.close();
  });

  test("the pinned timeline pins and scrubs", async () => {
    const { page, context } = await open(url);
    const pinned = await page.evaluate(() =>
      window.ScrollTrigger.getAll().some((st) => st.pin && st.trigger.dataset.gsapTimeline !== undefined),
    );
    assert.ok(pinned, "a pinned ScrollTrigger exists");
    assert.equal(await page.locator(".pin-spacer").count(), 1);
    await page.$eval(".pin-spacer", (el) => window.scrollTo(0, el.offsetTop + 600));
    await page.waitForFunction(() => {
      const t = getComputedStyle(document.querySelector(".band")).transform;
      return t !== "none" && t !== "matrix(1, 0, 0, 1, 0, 0)";
    });
    await context.close();
  });

  test("parallax and the progress bar follow the scroll", async () => {
    const { page, context } = await open(url);
    const bar = () =>
      page.$eval(".gsap-progress", (el) => new DOMMatrix(getComputedStyle(el).transform).a);
    assert.ok((await bar()) < 0.01);
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await page.waitForFunction(
      () => new DOMMatrix(getComputedStyle(document.querySelector(".gsap-progress")).transform).a > 0.95,
    );
    const art = await page.$eval("#parallax", (el) => window.gsap.getProperty(el, "yPercent"));
    assert.ok(art < -1, `parallax moved: ${art}`);
    await context.close();
  });

  test("htmx content animates and removed content is reverted", async () => {
    const { page, context, errors } = await open(url);
    await scrollTo(page, "#htmx");
    await page.evaluate(() => {
      window.__refresh = 0;
      const real = window.ScrollTrigger.refresh;
      window.ScrollTrigger.refresh = function (...args) {
        window.__refresh += 1;
        return real.apply(this, args);
      };
    });
    await page.click("#load-more");
    // htmx fires htmx:load after its settle delay, so wait for the marks first.
    await page.waitForSelector(".more-item:nth-child(3)[data-gsap-init]");
    await waitOpaque(page, ".more-item:last-child");
    await page.waitForFunction(() => window.__refresh > 0);

    for (let i = 0; i < 3; i++) {
      await scrollTo(page, "#swap");
      const old = await page.$("#swap > div");
      await page.click("#replace");
      await page.waitForFunction((o) => !o.isConnected, old);
      assert.equal(await old.evaluate((el) => el.hasAttribute("data-gsap-init")), false);
      await page.waitForSelector("#swap .swap-item:nth-child(3)[data-gsap-init]");
      await waitOpaque(page, "#swap .swap-item:last-child");
    }
    await page.waitForTimeout(150);
    assert.equal(await detached(page), 0, "no ScrollTrigger keeps a removed trigger");
    assert.deepEqual(errors, []);
    await context.close();
  });

  test("scan and revert work by hand and skip bad values", async () => {
    const { page, context, warnings } = await open(url);
    const result = await page.evaluate(() => {
      const box = document.createElement("div");
      box.innerHTML =
        '<p data-gsap="fade" data-gsap-on="load" data-gsap-ease="nope">a</p>' +
        '<p data-gsap="bogus" data-gsap-on="load">b</p>' +
        '<p data-gsap="custom" data-gsap-on="load" data-gsap-from=\'{"onStart":1}\'>c</p>';
      document.body.appendChild(box);
      const scanned = window.AutumnGsap.scan(box);
      const again = window.AutumnGsap.scan(box);
      const reverted = window.AutumnGsap.revert(box);
      return {
        scanned,
        again,
        reverted,
        marked: box.querySelectorAll("[data-gsap-init]").length,
      };
    });
    assert.deepEqual(result, { scanned: 3, again: 0, reverted: 3, marked: 0 });
    assert.ok(warnings.some((w) => w.includes("data-gsap-ease nope")), warnings.join("\n"));
    assert.ok(warnings.some((w) => w.includes("data-gsap bogus")), warnings.join("\n"));
    await context.close();
  });
});

// Adds `html` to the page, scans it, and waits `ms`.
async function inject(page, html, ms = 0) {
  await page.evaluate((h) => {
    const box = document.createElement("div");
    box.id = "probe";
    box.innerHTML = h;
    document.body.prepend(box);
    window.AutumnGsap.scan(box);
  }, html);
  if (ms) await page.waitForTimeout(ms);
}

describe("runtime edge cases", () => {
  test("blur-in and flip-x end at their CSS state", async () => {
    const { page, context } = await open(url);
    await inject(
      page,
      '<p id="blur" data-gsap="blur-in" data-gsap-on="load" data-gsap-duration="0.2">a</p>' +
        '<div id="flip" style="height:400px" data-gsap="flip-x" data-gsap-on="load" ' +
        'data-gsap-duration="0.4">b</div>',
    );
    // In the middle of the flip, the perspective stays 600px.
    await page.waitForTimeout(200);
    const mid = await page.$eval("#flip", (el) => getComputedStyle(el).transform);
    await page.waitForTimeout(400);
    const end = await page.evaluate(() => ({
      blur: getComputedStyle(document.querySelector("#blur")).opacity,
      flip: getComputedStyle(document.querySelector("#flip")).opacity,
      height: Math.round(document.querySelector("#flip").getBoundingClientRect().height),
    }));
    assert.deepEqual(end, { blur: "1", flip: "1", height: 400 });
    assert.ok(!/perspective\((?!600px)/.test(mid), mid);
    await context.close();
  });

  test("split chars do not break a word at a line wrap", async () => {
    const { page, context } = await open(url, { viewport: { width: 360, height: 800 } });
    await page.waitForTimeout(100);
    const broken = await page.$eval("#split", (el) => {
      // Walk the text nodes in order. Two chars with no space between them are one word.
      const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
      let prev = null;
      let count = 0;
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!/\S/.test(n.textContent)) {
          prev = null;
          continue;
        }
        const top = n.parentElement.getBoundingClientRect().top;
        if (prev !== null && top > prev + 5) count += 1;
        prev = top;
      }
      return count;
    });
    assert.equal(broken, 0);
    await context.close();
  });

  test("a pin without scrub pins again after the first pass", async () => {
    const { page, context } = await open(url);
    await page.evaluate(() => {
      const box = document.createElement("section");
      box.id = "pinbox";
      box.style.height = "300px";
      box.setAttribute("data-gsap", "fade");
      box.setAttribute("data-gsap-pin", "");
      box.setAttribute("data-gsap-start", "top top");
      box.setAttribute("data-gsap-end", "+=600");
      document.querySelector("#htmx").before(box);
      window.AutumnGsap.scan(box);
      window.ScrollTrigger.refresh();
    });
    const top = () => page.$eval("#pinbox", (el) => Math.round(el.getBoundingClientRect().top));
    const start = await page.evaluate(
      () => window.ScrollTrigger.getAll().find((s) => s.trigger.id === "pinbox").start,
    );
    for (const y of [start + 300, start + 1200, start + 300]) {
      await page.evaluate((v) => window.scrollTo(0, v), y);
      await page.waitForTimeout(150);
    }
    assert.equal(await top(), 0, "pinned on the second pass");
    await context.close();
  });

  test("htmx swaps that remove a pinned element work", async () => {
    const { page, context, errors } = await open(url);
    const swapErrors = [];
    await page.exposeFunction("swapError", (m) => swapErrors.push(m));
    await page.evaluate(() => {
      document.body.addEventListener("htmx:swapError", (e) => window.swapError(String(e.detail.error)));
      const box = document.createElement("div");
      box.id = "pinhost";
      box.innerHTML =
        '<section id="pinned-a" style="height:200px" data-gsap="fade" data-gsap-pin ' +
        'data-gsap-on="scroll" data-gsap-start="top top" data-gsap-end="+=300">a</section>';
      document.querySelector("#htmx").before(box);
      const outer = document.createElement("section");
      outer.id = "pinned-b";
      outer.style.height = "200px";
      outer.setAttribute("data-gsap", "fade");
      outer.setAttribute("data-gsap-pin", "");
      outer.setAttribute("data-gsap-start", "top top");
      outer.setAttribute("data-gsap-end", "+=300");
      document.querySelector("#htmx").before(outer);
      window.AutumnGsap.scan(box);
      window.AutumnGsap.scan(outer);
      const b1 = document.createElement("button");
      b1.id = "swap-inner";
      b1.setAttribute("hx-get", "/swap");
      b1.setAttribute("hx-target", "#pinhost");
      b1.setAttribute("hx-swap", "innerHTML");
      const b2 = document.createElement("button");
      b2.id = "swap-outer";
      b2.setAttribute("hx-get", "/swap");
      b2.setAttribute("hx-target", "#pinned-b");
      b2.setAttribute("hx-swap", "outerHTML");
      document.body.prepend(b1, b2);
      window.htmx.process(b1);
      window.htmx.process(b2);
    });
    assert.equal(await page.locator(".pin-spacer").count(), 3);
    await page.click("#swap-inner");
    await page.waitForSelector("#pinhost .swap-item");
    await page.click("#swap-outer");
    await page.waitForFunction(() => !document.querySelector("#pinned-b"));
    await page.waitForTimeout(200);
    const state = await page.evaluate(() => ({
      oldA: !!document.querySelector("#pinned-a"),
      hostItems: document.querySelectorAll("#pinhost .swap-item").length,
      outerItems: document.querySelectorAll(".swap-item").length,
      spacers: document.querySelectorAll(".pin-spacer").length,
    }));
    assert.deepEqual(swapErrors, []);
    assert.deepEqual(state, { oldA: false, hostItems: 3, outerItems: 9, spacers: 1 });
    assert.equal(await detached(page), 0);
    assert.deepEqual(errors, []);
    await context.close();
  });

  test("htmx history restore animates the content again", async () => {
    const { page, context, errors } = await open(url);
    await page.click("#about-link");
    await page.waitForSelector("#about[data-gsap-init]");
    await page.goBack();
    await page.waitForSelector("#card-fade-up");
    await page.waitForTimeout(200);
    await scrollTo(page, "#card-fade-up");
    await waitOpaque(page, "#card-fade-up");
    const state = await page.evaluate(() => ({
      spacers: document.querySelectorAll(".pin-spacer").length,
      chars: document.querySelector("#split").querySelectorAll("[aria-hidden=true]").length > 0,
    }));
    assert.deepEqual(state, { spacers: 1, chars: true });
    assert.equal(await detached(page), 0);
    assert.deepEqual(errors, []);
    await context.close();
  });

  test("an error in one element does not leave the GSAP context open", async () => {
    const { page, context } = await open(url);
    const state = await page.evaluate(() => {
      const box = document.createElement("div");
      box.innerHTML = '<p id="bad" data-gsap="fade" data-gsap-on="load">x</p>';
      document.body.appendChild(box);
      const real = window.gsap.from;
      window.gsap.from = () => {
        throw new Error("boom");
      };
      const n = window.AutumnGsap.scan(box);
      window.gsap.from = real;
      return {
        n,
        marked: document.querySelector("#bad").hasAttribute("data-gsap-init"),
        context: !!window.gsap.core.context(),
      };
    });
    assert.deepEqual(state, { n: 0, marked: false, context: false });
    await context.close();
  });
});

describe("accessibility and layout", () => {
  test("an element below the reveal line at the page end appears", async () => {
    const { page, context } = await open(url);
    await page.evaluate(() => {
      const p = document.createElement("p");
      p.id = "last";
      p.textContent = "The end.";
      p.setAttribute("data-gsap", "fade");
      document.body.appendChild(p);
      window.AutumnGsap.scan(p);
      window.ScrollTrigger.refresh();
      window.scrollTo(0, document.documentElement.scrollHeight);
    });
    await waitOpaque(page, "#last");
    await context.close();
  });

  test("print shows all content", async () => {
    const { page, context } = await open(url);
    await page.emulateMedia({ media: "print" });
    const hidden = await page.$$eval("[data-gsap-init], [data-gsap-init] *", (els) =>
      els.filter((el) => Number(getComputedStyle(el).opacity) < 1).map((el) => el.id || el.className),
    );
    assert.deepEqual(hidden, []);
    await context.close();
  });

  test("split text with a link keeps the link readable", async () => {
    const { page, context, warnings } = await open(url);
    await inject(
      page,
      '<p id="lnk" data-gsap="fade" data-gsap-split="words" data-gsap-on="load">' +
        'Read the <a href="#x">install guide</a> now.</p>',
    );
    const state = await page.$eval("#lnk", (el) => ({
      hidden: el.querySelectorAll("[aria-hidden]").length,
      link: el.querySelector("a").textContent,
      marked: el.hasAttribute("data-gsap-init"),
    }));
    assert.deepEqual(state, { hidden: 0, link: "install guide", marked: true });
    assert.ok(warnings.some((w) => w.includes("data-gsap-split")), warnings.join("\n"));
    await context.close();
  });

  test("the split mask does not clip descenders", async () => {
    const { page, context } = await open(url);
    const pad = await page.$eval("#split .gsap-char-mask", (el) => parseFloat(getComputedStyle(el).paddingBottom));
    assert.ok(pad > 0, `mask padding: ${pad}`);
    await context.close();
  });

  test("a reduced-motion change after load reverts, then starts again", async () => {
    const { page, context, errors } = await open(url);
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.waitForFunction(() => document.querySelectorAll("[data-gsap-init]").length === 0);
    assert.equal(await opacity(page, "#card-fade-up"), 1);
    assert.equal(await page.locator(".pin-spacer").count(), 0);
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await page.waitForSelector("#card-fade-up[data-gsap-init]");
    assert.equal(await page.locator(".pin-spacer").count(), 1);
    assert.deepEqual(errors, []);
    await context.close();
  });

  test("a reload keeps the scroll position below a pin", async () => {
    const { page, context } = await open(url);
    await scrollTo(page, "#htmx");
    await page.waitForTimeout(100);
    const before = await page.$eval("#htmx", (el) => Math.round(el.getBoundingClientRect().top));
    await page.reload();
    await page.waitForFunction(() => window.AutumnGsap && window.htmx);
    await page.waitForTimeout(300);
    const after = await page.$eval("#htmx", (el) => Math.round(el.getBoundingClientRect().top));
    assert.ok(Math.abs(after - before) < 50, `before ${before}, after ${after}`);
    await context.close();
  });
});

describe("untrusted content", () => {
  test("ignored regions do not animate and a foreign pin is refused", async () => {
    const { page, context, warnings } = await open(url);
    const result = await page.evaluate(() => {
      const victim = document.createElement("header");
      victim.id = "victim";
      document.body.prepend(victim);
      const box = document.createElement("div");
      box.innerHTML =
        '<div data-gsap-ignore><p data-gsap="fade" data-gsap-on="load">a</p></div>' +
        '<div hx-disable><p data-gsap="fade" data-gsap-on="load">b</p></div>' +
        '<div data-gsap-ignore><div data-gsap-timeline data-gsap-on="load">' +
        '<p data-gsap="fade">c</p></div></div>' +
        '<p id="evil" data-gsap="fade" data-gsap-trigger="#victim" data-gsap-pin ' +
        'data-gsap-end="+=9999">d</p>';
      document.body.appendChild(box);
      const scanned = window.AutumnGsap.scan(box);
      return {
        scanned,
        marked: box.querySelectorAll("[data-gsap-init]").length,
        victimParent: victim.parentElement.className,
        victimPins: window.ScrollTrigger.getAll().filter((s) => s.pin === victim).length,
      };
    });
    assert.deepEqual(result, { scanned: 1, marked: 1, victimParent: "", victimPins: 0 });
    assert.ok(warnings.some((w) => w.includes("data-gsap-pin")), warnings.join("\n"));
    await context.close();
  });
});

describe("reduced motion", () => {
  test("elements do not animate and stay visible", async () => {
    const { page, context, errors } = await open(url, { reducedMotion: "reduce" });
    assert.equal(await page.locator("[data-gsap-init]").count(), 0);
    assert.equal(await opacity(page, "#card-fade-up"), 1);
    assert.equal(await page.$eval("#split", (el) => el.children.length), 0);
    assert.equal(await page.$eval(".gsap-progress", (el) => getComputedStyle(el).display), "none");
    const optIn = await page.evaluate(() => {
      const p = document.createElement("p");
      p.setAttribute("data-gsap", "fade");
      p.setAttribute("data-gsap-on", "load");
      p.setAttribute("data-gsap-reduced", "animate");
      document.body.appendChild(p);
      return window.AutumnGsap.scan(p);
    });
    assert.equal(optIn, 1, "data-gsap-reduced=animate opts in");
    assert.deepEqual(errors, []);
    await context.close();
  });
});

describe("strict CSP (no inline styles or scripts)", () => {
  test("animations work with no CSP violations", async () => {
    const { page, context, errors } = await open(strictUrl);
    await waitOpaque(page, "#lede");
    await scrollTo(page, "#card-fade-up");
    await waitOpaque(page, "#card-fade-up");
    await page.$eval(".pin-spacer", (el) => window.scrollTo(0, el.offsetTop + 600));
    await scrollTo(page, "#htmx");
    await page.click("#replace");
    await page.waitForSelector("#swap .swap-item:nth-child(3)[data-gsap-init]");
    await page.waitForTimeout(200);
    assert.deepEqual(await page.evaluate(() => window.__csp), []);
    assert.deepEqual(errors, []);
    await context.close();
  });
});
