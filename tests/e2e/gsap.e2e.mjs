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
    assert.ok(marked, "timeline and steps are marked");
    await waitOpaque(page, "#lede");
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
    assert.equal(split.label, "Server HTML, GSAP motion.");
    assert.ok(split.parts >= 20, `chars are split: ${split.parts}`);
    await context.close();
  });

  test("a card is hidden below the fold and reveals on scroll", async () => {
    const { page, context } = await open(url);
    assert.ok((await opacity(page, "#card-fade-up")) < 0.05);
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
    await page.click("#load-more");
    // htmx fires htmx:load after its settle delay, so wait for the marks first.
    await page.waitForSelector(".more-item:nth-child(3)[data-gsap-init]");
    await waitOpaque(page, ".more-item:last-child");

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
    assert.ok(warnings.some((w) => w.includes('data-gsap-ease="nope"')), warnings.join("\n"));
    assert.ok(warnings.some((w) => w.includes('data-gsap="bogus"')), warnings.join("\n"));
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
