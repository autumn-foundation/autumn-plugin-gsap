/* autumn-plugin-gsap: declarative GSAP init (0.1.0).
 *
 * - Reads `data-gsap*` attributes and starts GSAP tweens and timelines.
 * - Scans on page load and on each `htmx:load`.
 * - Reverts tweens, ScrollTriggers and SplitText on `htmx:beforeCleanupElement`.
 * - Does nothing when GSAP is missing. Skips elements when the user prefers reduced motion.
 * - Ignores bad attribute values (with a console warning). A bad value never stops the scan.
 *
 * The attribute contract is in README.md. The Rust builders write the same values.
 */
(function () {
    "use strict";

    var VERSION = "0.1.0";
    var INIT = "data-gsap-init";
    var TIMELINE = "data-gsap-timeline";
    var TIMELINE_SEL = "[" + TIMELINE + "]";
    // Regions that never animate. `hx-disable` is the htmx marker for untrusted content.
    var IGNORE_SEL = "[data-gsap-ignore],[hx-disable],[data-hx-disable]";
    var INTERACTIVE_SEL = "a,button,input,select,textarea,[tabindex],[contenteditable]";

    // Value grammar. Keep these lines equal to src/grammar.rs (a Rust test checks it).
    var RE_SECS = /^(\d+(\.\d+)?)$/;
    var RE_NUM = /^(-?\d+(\.\d+)?)$/;
    var RE_EASE = /^(none|(power[1-4]|sine|expo|circ|bounce)\.(in|out|inOut)|back\.(in|out|inOut)(\(-?\d+(\.\d+)?\))?|elastic\.(in|out|inOut)(\(-?\d+(\.\d+)?,-?\d+(\.\d+)?\))?|steps\([1-9]\d*\))$/;
    var RE_SCROLL_POS = /^((top|center|bottom|-?\d+(\.\d+)?(px|%)) (top|center|bottom|-?\d+(\.\d+)?(px|%))|\+=\d+(\.\d+)?(px|%)?)$/;
    var RE_POSITION = /^(<|>|[<>]-?\d+(\.\d+)?|[+-]=\d+(\.\d+)?|\d+(\.\d+)?)$/;
    var RE_INDEX = /^(\d+)$/;

    var ACTIONS = ["play", "pause", "resume", "reverse", "restart", "reset", "complete", "none"];
    var STAGGER_WORDS = ["start", "center", "end", "edges", "random"];
    var SPLITS = ["chars", "words", "lines"];
    var PROPS = [
        "x", "y", "xPercent", "yPercent", "scale", "scaleX", "scaleY",
        "rotation", "rotationX", "rotationY", "skewX", "skewY", "opacity", "autoAlpha",
    ];

    var DEFAULTS = {
        kind: "fade-up",
        duration: 0.8,
        ease: "power3.out",
        start: "top 85%",
        scrubStart: "top bottom",
        scrubEnd: "bottom top",
        splitStagger: 0.03,
        parallax: 0.3,
    };

    // Preset start states. Keep the names equal to `Preset` in src/tween.rs.
    var PRESETS = {
        fade: { opacity: 0 },
        "fade-up": { opacity: 0, y: 32 },
        "fade-down": { opacity: 0, y: -32 },
        "fade-left": { opacity: 0, x: 32 },
        "fade-right": { opacity: 0, x: -32 },
        scale: { opacity: 0, scale: 0.92 },
        "zoom-in": { opacity: 0, scale: 0.6 },
        "zoom-out": { opacity: 0, scale: 1.3 },
        "slide-up": { opacity: 0, yPercent: 100 },
        "slide-down": { opacity: 0, yPercent: -100 },
        "slide-left": { opacity: 0, xPercent: 100 },
        "slide-right": { opacity: 0, xPercent: -100 },
        "rotate-in": { opacity: 0, rotation: -12 },
        "blur-in": { opacity: 0, filter: "blur(12px)" },
        "flip-x": { opacity: 0, rotationX: -90, transformPerspective: 600, transformOrigin: "50% 0%" },
    };
    // End states for presets that need `fromTo`. GSAP animates only the properties
    // in the end state, so each end state lists every property of its start state.
    var PRESET_ENDS = {
        "blur-in": { opacity: 1, filter: "blur(0px)" },
        // Keep the perspective and origin constant. A `from` would animate them to 0.
        "flip-x": { opacity: 1, rotationX: 0, transformPerspective: 600, transformOrigin: "50% 0%" },
    };
    var SPECIAL = ["custom", "parallax", "scroll-progress"];
    var KINDS = Object.keys(PRESETS).concat(SPECIAL);

    // ---- Parsers: each returns the value, or null for a bad value. ----

    function has(list, v) {
        return list.indexOf(v) >= 0;
    }

    function match(re, raw) {
        return typeof raw === "string" && re.test(raw) ? raw : null;
    }

    function secs(raw) {
        return match(RE_SECS, raw) === null ? null : parseFloat(raw);
    }

    function factor(raw) {
        return match(RE_NUM, raw) === null ? null : parseFloat(raw);
    }

    function ease(raw) {
        return match(RE_EASE, raw);
    }

    function scrollPos(raw) {
        return match(RE_SCROLL_POS, raw);
    }

    function position(raw) {
        return match(RE_POSITION, raw);
    }

    function repeat(raw) {
        if (raw === "-1") {
            return -1;
        }
        return match(RE_INDEX, raw) === null ? null : parseInt(raw, 10);
    }

    function toggleActions(raw) {
        if (typeof raw !== "string") {
            return null;
        }
        var parts = raw.split(" ");
        for (var i = 0; i < parts.length; i++) {
            if (!has(ACTIONS, parts[i])) {
                return null;
            }
        }
        return parts.length === 4 ? raw : null;
    }

    function scrub(raw) {
        return raw === "true" ? true : secs(raw);
    }

    function staggerFrom(raw) {
        if (has(STAGGER_WORDS, raw)) {
            return raw;
        }
        return match(RE_INDEX, raw) === null ? null : parseInt(raw, 10);
    }

    function split(raw) {
        return has(SPLITS, raw) ? raw : null;
    }

    function kind(raw) {
        return has(KINDS, raw) ? raw : null;
    }

    function on(raw) {
        return raw === "load" || raw === "scroll" ? raw : null;
    }

    function once(raw) {
        if (raw === "true") {
            return true;
        }
        return raw === "false" ? false : null;
    }

    function reduced(raw) {
        return raw === "animate" ? raw : null;
    }

    function selector(raw) {
        return typeof raw === "string" && raw.trim() !== "" ? raw.trim() : null;
    }

    // A bare attribute: any value means "on".
    function flag(raw) {
        return typeof raw === "string";
    }

    // JSON tween properties. Keeps only listed names with finite numbers.
    function vars(raw) {
        var data;
        try {
            data = JSON.parse(raw);
        } catch (e) {
            return null;
        }
        if (!data || typeof data !== "object" || Array.isArray(data)) {
            return null;
        }
        var out = {};
        var count = 0;
        PROPS.forEach(function (name) {
            var v = Object.prototype.hasOwnProperty.call(data, name) ? data[name] : null;
            if (typeof v === "number" && isFinite(v)) {
                out[name] = v;
                count += 1;
            }
        });
        return count > 0 ? out : null;
    }

    var PARSERS = {
        "data-gsap": kind,
        "data-gsap-timeline": flag,
        "data-gsap-from": vars,
        "data-gsap-to": vars,
        "data-gsap-parallax": factor,
        "data-gsap-on": on,
        "data-gsap-delay": secs,
        "data-gsap-duration": secs,
        "data-gsap-ease": ease,
        "data-gsap-repeat": repeat,
        "data-gsap-yoyo": flag,
        "data-gsap-repeat-delay": secs,
        "data-gsap-trigger": selector,
        "data-gsap-start": scrollPos,
        "data-gsap-end": scrollPos,
        "data-gsap-toggle-actions": toggleActions,
        "data-gsap-once": once,
        "data-gsap-scrub": scrub,
        "data-gsap-pin": flag,
        "data-gsap-markers": flag,
        "data-gsap-reduced": reduced,
        "data-gsap-stagger": secs,
        "data-gsap-stagger-from": staggerFrom,
        "data-gsap-stagger-ease": ease,
        "data-gsap-split": split,
        "data-gsap-split-mask": flag,
        "data-gsap-position": position,
        "data-gsap-ignore": flag,
    };

    function known(name) {
        return Object.prototype.hasOwnProperty.call(PARSERS, name);
    }

    function attr(name, raw) {
        return known(name) ? PARSERS[name](raw) : undefined;
    }

    function warn(el, name, raw) {
        if (typeof console !== "undefined" && console.warn) {
            // Format arguments, so a value cannot inject console format codes.
            console.warn('[autumn-plugin-gsap] ignored %s="%s"', name, String(raw), el);
        }
    }

    // Reads and parses one attribute. Returns null when it is absent or bad.
    function read(el, name) {
        var raw = el.getAttribute(name);
        if (raw === null) {
            return null;
        }
        var v = PARSERS[name](raw);
        if (v === null) {
            warn(el, name, raw);
        }
        return v;
    }

    // ---- Animation ----

    function assign(target) {
        for (var i = 1; i < arguments.length; i++) {
            var src = arguments[i];
            if (src) {
                for (var k in src) {
                    if (Object.prototype.hasOwnProperty.call(src, k)) {
                        target[k] = src[k];
                    }
                }
            }
        }
        return target;
    }

    function reducedMotion() {
        return (
            typeof window.matchMedia === "function" &&
            window.matchMedia("(prefers-reduced-motion: reduce)").matches
        );
    }

    // The timing vars that the element sets. With `withDefaults`, missing values get defaults.
    function timing(el, withDefaults) {
        var out = {};
        var duration = read(el, "data-gsap-duration");
        var easeV = read(el, "data-gsap-ease");
        if (duration !== null || withDefaults) {
            out.duration = duration === null ? DEFAULTS.duration : duration;
        }
        if (easeV !== null || withDefaults) {
            out.ease = easeV === null ? DEFAULTS.ease : easeV;
        }
        var delay = read(el, "data-gsap-delay");
        if (delay !== null) {
            out.delay = delay;
        }
        var rep = read(el, "data-gsap-repeat");
        if (rep !== null) {
            out.repeat = rep;
        }
        if (read(el, "data-gsap-yoyo")) {
            out.yoyo = true;
        }
        var repeatDelay = read(el, "data-gsap-repeat-delay");
        if (repeatDelay !== null) {
            out.repeatDelay = repeatDelay;
        }
        return out;
    }

    function resolveTrigger(el) {
        var sel = read(el, "data-gsap-trigger");
        if (sel !== null) {
            try {
                var found = document.querySelector(sel);
                if (found) {
                    return found;
                }
            } catch (e) {
                // A bad selector: use the element itself.
            }
            warn(el, "data-gsap-trigger", sel);
        }
        return el;
    }

    // The default start: `top 85%`, but never after the last scroll position.
    // Without the clamp, an element near the page end never plays and stays hidden.
    function defaultStart(trigger) {
        return function () {
            var top = trigger.getBoundingClientRect().top + window.scrollY - window.innerHeight * 0.85;
            var ST = window.ScrollTrigger;
            var max = ST && typeof ST.maxScroll === "function" ? ST.maxScroll(window) : top + 1;
            return Math.min(top, max - 1);
        };
    }

    // The ScrollTrigger vars, or null when the element plays on load.
    function scrollTrigger(el) {
        if (read(el, "data-gsap-on") === "load") {
            return null;
        }
        var st = { trigger: resolveTrigger(el) };
        var start = read(el, "data-gsap-start");
        var end = read(el, "data-gsap-end");
        var scrubV = read(el, "data-gsap-scrub");
        var pin = read(el, "data-gsap-pin");
        if (scrubV !== null) {
            st.scrub = scrubV;
            st.start = start || DEFAULTS.scrubStart;
            st.end = end || DEFAULTS.scrubEnd;
        } else {
            st.start = start || defaultStart(st.trigger);
            if (end) {
                st.end = end;
            }
            var actions = read(el, "data-gsap-toggle-actions");
            var onceV = read(el, "data-gsap-once");
            if (actions) {
                st.toggleActions = actions;
            } else if (onceV === false) {
                st.toggleActions = "play none none reverse";
            } else if (!pin) {
                // A pin must stay alive. With `once`, the trigger dies and leaves an empty gap.
                st.once = true;
            }
        }
        if (pin) {
            // Pin only the element or an element in it. Do not move other page content.
            if (st.trigger === el || el.contains(st.trigger)) {
                st.pin = true;
            } else {
                warn(el, "data-gsap-pin", "the trigger is outside the element");
            }
        }
        if (read(el, "data-gsap-markers")) {
            st.markers = true;
        }
        return st;
    }

    // The start and end states of an element: { from, to }, or null.
    function states(el, k) {
        if (k === "custom") {
            var from = read(el, "data-gsap-from");
            var to = read(el, "data-gsap-to");
            return from || to ? { from: from, to: to } : null;
        }
        var to = PRESET_ENDS[k] ? assign({}, PRESET_ENDS[k]) : null;
        if (k === "blur-in") {
            // Remove the inline filter at the end, so a CSS filter applies again.
            to.clearProps = "filter";
        }
        return { from: assign({}, PRESETS[k]), to: to };
    }

    // An end state with opacity 1 must use the CSS opacity of each target.
    // Read it before the tween sets the start state.
    function endOpacity(gsap, st, list) {
        if (!st.to || st.to.opacity !== 1 || st.from.opacity === undefined) {
            return;
        }
        var all = gsap.utils.toArray(list);
        var values = all.map(function (t) {
            return gsap.getProperty(t, "opacity");
        });
        st.to.opacity = function (i) {
            return values[i];
        };
    }

    // Marks `node` as handled and keeps its style attribute, for the history snapshot.
    function claim(node, rec) {
        node.setAttribute(INIT, "true");
        rec.claimed.push(node);
        rec.styles.push({ node: node, style: node.getAttribute("style") });
    }

    // The tween targets: split text units, direct children (stagger) or the element.
    function targets(el, rec) {
        var unit = read(el, "data-gsap-split");
        if (unit !== null && el.querySelector(INTERACTIVE_SEL)) {
            // SplitText hides the parts from screen readers. A link or control in them
            // loses its name, so animate the element as a whole.
            warn(el, "data-gsap-split", "skipped: the text contains links or controls");
            unit = null;
        }
        if (unit !== null && typeof window.SplitText === "function") {
            // Split chars inside words, so a line wrap never breaks a word.
            var opts = {
                type: unit === "chars" ? "words,chars" : unit,
                aria: "auto",
                charsClass: "gsap-char",
                wordsClass: "gsap-word",
                linesClass: "gsap-line",
            };
            if (read(el, "data-gsap-split-mask")) {
                opts.mask = unit;
            }
            var before = {
                el: el,
                html: el.innerHTML,
                label: el.getAttribute("aria-label"),
            };
            var parts = window.SplitText.create(el, opts)[unit];
            if (parts && parts.length) {
                rec.splits.push(before);
                rec.owns = true;
                return { list: parts, split: true };
            }
        }
        if (el.hasAttribute("data-gsap-stagger")) {
            var kids = [];
            for (var i = 0; i < el.children.length; i++) {
                var c = el.children[i];
                if (!c.hasAttribute(INIT)) {
                    claim(c, rec);
                    kids.push(c);
                }
            }
            rec.owns = true;
            return { list: kids, split: false };
        }
        return { list: el, split: false };
    }

    function staggerVars(el, isSplit) {
        var each = read(el, "data-gsap-stagger");
        if (each === null && isSplit) {
            each = DEFAULTS.splitStagger;
        }
        if (each === null) {
            return null;
        }
        var out = { each: each };
        var from = read(el, "data-gsap-stagger-from");
        if (from !== null) {
            out.from = from;
        }
        var se = read(el, "data-gsap-stagger-ease");
        if (se !== null) {
            out.ease = se;
        }
        return out;
    }

    // Adds one tween to `host` (gsap or a timeline).
    function addTween(host, list, st, timingVars, pos) {
        var args;
        if (st.from && st.to) {
            args = [list, st.from, assign({}, st.to, timingVars)];
            return pos === undefined ? host.fromTo.apply(host, args) : host.fromTo(args[0], args[1], args[2], pos);
        }
        if (st.to) {
            var tv = assign({}, st.to, timingVars);
            return pos === undefined ? host.to(list, tv) : host.to(list, tv, pos);
        }
        var fv = assign({}, st.from, timingVars);
        return pos === undefined ? host.from(list, fv) : host.from(list, fv, pos);
    }

    function kindOf(el) {
        var k = read(el, "data-gsap");
        return k === null ? DEFAULTS.kind : k;
    }

    function animateScrollProgress(gsap, el) {
        gsap.fromTo(
            el,
            { scaleX: 0 },
            {
                scaleX: 1,
                ease: "none",
                scrollTrigger: {
                    trigger: document.documentElement,
                    start: "top top",
                    end: "bottom bottom",
                    scrub: true,
                },
            },
        );
    }

    function animateParallax(gsap, el) {
        var f = read(el, "data-gsap-parallax");
        gsap.to(el, {
            yPercent: (f === null ? DEFAULTS.parallax : f) * 100,
            ease: "none",
            scrollTrigger: { trigger: el, start: "top bottom", end: "bottom top", scrub: true },
        });
    }

    function animateElement(gsap, el, rec) {
        var k = kindOf(el);
        if (k === "scroll-progress") {
            return animateScrollProgress(gsap, el);
        }
        if (k === "parallax") {
            return animateParallax(gsap, el);
        }
        var st = states(el, k);
        if (st === null) {
            return warn(el, "data-gsap", k + " needs data-gsap-from or data-gsap-to");
        }
        var t = targets(el, rec);
        if (t.list.length === 0) {
            return;
        }
        endOpacity(gsap, st, t.list);
        var v = timing(el, true);
        var stagger = staggerVars(el, t.split);
        if (stagger) {
            v.stagger = stagger;
        }
        var trig = scrollTrigger(el);
        if (trig) {
            v.scrollTrigger = trig;
        }
        addTween(gsap, t.list, st, v);
    }

    function animateTimeline(gsap, box, rec) {
        var own = timing(box, true);
        var tv = { defaults: { duration: own.duration, ease: own.ease } };
        ["delay", "repeat", "yoyo", "repeatDelay"].forEach(function (key) {
            if (key in own) {
                tv[key] = own[key];
            }
        });
        var trig = scrollTrigger(box);
        if (trig) {
            tv.scrollTrigger = trig;
        }
        var tl = gsap.timeline(tv);
        rec.owns = true;
        var found = box.querySelectorAll("[data-gsap]");
        for (var i = 0; i < found.length; i++) {
            var child = found[i];
            if (child.hasAttribute(INIT) || child.closest(TIMELINE_SEL) !== box || ignored(child)) {
                continue;
            }
            claim(child, rec);
            var k = kindOf(child);
            var st = has(SPECIAL, k) && k !== "custom" ? null : states(child, k);
            if (st === null) {
                warn(child, "data-gsap", k + " cannot be a timeline step");
                continue;
            }
            var t = targets(child, rec);
            if (t.list.length === 0) {
                continue;
            }
            endOpacity(gsap, st, t.list);
            var v = timing(child, false);
            var stagger = staggerVars(child, t.split);
            if (stagger) {
                v.stagger = stagger;
            }
            var pos = read(child, "data-gsap-position");
            addTween(tl, t.list, st, v, pos === null ? undefined : pos);
        }
    }

    // ---- Records: one gsap.context per element, so htmx cleanup can revert it. ----

    var RECORDS = typeof WeakMap === "function" ? new WeakMap() : null;
    // The live records, for the history snapshot. `release` removes a record.
    var LIVE = typeof Set === "function" ? new Set() : null;
    var registered = false;

    function register(gsap) {
        if (registered) {
            return;
        }
        registered = true;
        // `typeof` checks: an element with id="SplitText" must not count as the plugin.
        var plugins = [window.ScrollTrigger, window.SplitText].filter(function (p) {
            return typeof p === "function";
        });
        if (plugins.length) {
            gsap.registerPlugin.apply(gsap, plugins);
        }
    }

    function start(gsap, el, fn) {
        var rec = { el: el, claimed: [], styles: [], splits: [], owns: false, ctx: null };
        var failed = null;
        claim(el, rec);
        // Catch inside the context function. GSAP closes the context only when it returns.
        rec.ctx = gsap.context(function () {
            try {
                fn(gsap, el, rec);
            } catch (e) {
                failed = e;
            }
        });
        if (failed !== null) {
            warn(el, "data-gsap", String(failed && failed.message ? failed.message : failed));
            rec.ctx.revert();
            rec.claimed.forEach(function (c) {
                c.removeAttribute(INIT);
            });
            return false;
        }
        if (RECORDS) {
            RECORDS.set(el, rec);
        }
        if (LIVE) {
            LIVE.add(rec);
        }
        return true;
    }

    function optedIn(el) {
        return read(el, "data-gsap-reduced") === "animate";
    }

    function ignored(el) {
        return el.closest(IGNORE_SEL) !== null;
    }

    function collect(scope, sel) {
        var out = [];
        if (scope !== document && scope.matches && scope.matches(sel)) {
            out.push(scope);
        }
        var found = scope.querySelectorAll(sel);
        for (var i = 0; i < found.length; i++) {
            out.push(found[i]);
        }
        return out;
    }

    /** Scans `root` (default: the document). Returns the number of new animations. */
    function scan(root) {
        var gsap = window.gsap;
        if (!gsap || typeof gsap.context !== "function") {
            return 0;
        }
        register(gsap);
        var scope = root && root.querySelectorAll ? root : document;
        var reduce = reducedMotion();
        var count = 0;
        collect(scope, TIMELINE_SEL).forEach(function (box) {
            if (box.hasAttribute(INIT) || ignored(box) || (reduce && !optedIn(box))) {
                return;
            }
            if (start(gsap, box, animateTimeline)) {
                count += 1;
            }
        });
        collect(scope, "[data-gsap]").forEach(function (el) {
            if (
                el.hasAttribute(INIT) ||
                el.closest(TIMELINE_SEL) ||
                ignored(el) ||
                (reduce && !optedIn(el))
            ) {
                return;
            }
            if (start(gsap, el, animateElement)) {
                count += 1;
            }
        });
        if (count > 0) {
            dirty = true;
        }
        return count;
    }

    // Stops the animations of `el`. With `restore`, it also puts back the start DOM
    // (styles, split text, pin spacers). Without it, the DOM stays as it is: use this
    // while htmx removes the element, because a DOM change then breaks the swap.
    function release(el, restore) {
        var rec = RECORDS && RECORDS.get(el);
        if (!rec) {
            return false;
        }
        RECORDS.delete(el);
        if (LIVE) {
            LIVE.delete(rec);
        }
        if (restore) {
            rec.ctx.revert();
        } else {
            rec.ctx.data.slice().forEach(function (d) {
                // kill(false): ScrollTrigger keeps the DOM; tweens and SplitText stop.
                if (d && typeof d.kill === "function") {
                    d.kill(false);
                }
            });
            if (typeof rec.ctx.clear === "function") {
                rec.ctx.clear();
            }
        }
        rec.claimed.forEach(function (c) {
            c.removeAttribute(INIT);
        });
        return true;
    }

    /** Reverts the animations in `root` (default: the document). Returns the count. */
    function revert(root) {
        var scope = root && root.querySelectorAll ? root : document;
        var count = 0;
        collect(scope, "[" + INIT + "]").forEach(function (el) {
            if (release(el, true)) {
                count += 1;
            }
        });
        if (count > 0) {
            dirty = true;
        }
        return count;
    }

    var refreshTimer = null;
    // `true` after a scan or revert changed something. Only then is a refresh necessary.
    var dirty = false;

    function refreshSoon() {
        if (typeof window.ScrollTrigger !== "function" || !dirty) {
            return;
        }
        dirty = false;
        clearTimeout(refreshTimer);
        refreshTimer = setTimeout(function () {
            window.ScrollTrigger.refresh();
        }, 50);
    }

    function eventElement(e) {
        return e.detail && e.detail.elt ? e.detail.elt : e.target;
    }

    window.AutumnGsap = {
        version: VERSION,
        scan: scan,
        revert: revert,
        presets: KINDS.slice(),
        parse: {
            secs: secs,
            factor: factor,
            ease: ease,
            scrollPos: scrollPos,
            position: position,
            repeat: repeat,
            toggleActions: toggleActions,
            scrub: scrub,
            staggerFrom: staggerFrom,
            split: split,
            kind: kind,
            on: on,
            once: once,
            vars: vars,
            known: known,
            attr: attr,
        },
    };

    document.addEventListener("htmx:load", function (e) {
        scan(eventElement(e));
    });
    document.addEventListener("htmx:beforeCleanupElement", function (e) {
        // htmx removes this element now. Stop it, but do not change the DOM.
        release(eventElement(e), false);
    });
    document.addEventListener("htmx:afterSettle", function () {
        // A swap can change the page height, also with no animated content.
        dirty = true;
        refreshSoon();
    });

    // The swap style of an htmx swap: the HX-Reswap override, the nearest hx-swap,
    // or the htmx default.
    function swapStyle(d) {
        var spec = d.swapOverride;
        if (!spec) {
            // htmx sets `detail.elt` to the target. The trigger element is in requestConfig.
            var src = (d.requestConfig && d.requestConfig.elt) || d.elt;
            var holder = src && src.closest ? src.closest("[hx-swap],[data-hx-swap]") : null;
            spec = holder ? holder.getAttribute("hx-swap") || holder.getAttribute("data-hx-swap") : null;
        }
        if (!spec) {
            var h = window.htmx;
            spec = (h && h.config && h.config.defaultSwapStyle) || "innerHTML";
        }
        return String(spec).trim().split(/\s+/)[0];
    }

    // Targets that init.js reverted before an outerHTML swap. If the swap does not
    // come (a later listener cancels it), init.js scans them again.
    var pendingOuter = [];
    // Containers that animate their children (stagger, split, timeline). An innerHTML
    // swap replaces the children, so init.js starts the container again after the swap.
    var restartAfterSwap = typeof WeakSet === "function" ? new WeakSet() : null;

    // Runs on the kebab-case event, so `hx-on::before-swap` handlers run first.
    document.addEventListener("htmx:before-swap", function (e) {
        var d = e.detail || {};
        var target = d.target;
        if (!target || d.shouldSwap === false || !target.querySelectorAll) {
            return;
        }
        var style = swapStyle(d);
        if (style === "outerHTML" || style === "delete") {
            // The target leaves the page. Restore it now: a pin spacer must not wrap
            // the new content.
            if (revert(target) > 0) {
                pendingOuter.push(target);
            }
        } else if (style === "innerHTML" || style === "textContent") {
            var rec = RECORDS && RECORDS.get(target);
            if (rec && rec.owns && restartAfterSwap) {
                restartAfterSwap.add(target);
            }
        }
    });

    document.addEventListener("htmx:afterSwap", function (e) {
        var target = e.detail && e.detail.target;
        if (target && restartAfterSwap && restartAfterSwap.has(target)) {
            restartAfterSwap.delete(target);
            release(target, false);
            scan(target);
        }
    });

    document.addEventListener("htmx:afterRequest", function () {
        setTimeout(function () {
            var list = pendingOuter;
            pendingOuter = [];
            list.forEach(function (t) {
                if (t.isConnected && !t.hasAttribute(INIT)) {
                    scan(t);
                }
            });
        }, 0);
    });

    // Removes the GSAP state from the DOM in `root` (inline styles, split text,
    // init marks, pin spacers). Returns a function that puts the state back.
    function detach(root) {
        var undo = [];
        if (LIVE) {
            LIVE.forEach(function (rec) {
                if (!rec.el.isConnected) {
                    LIVE.delete(rec);
                    return;
                }
                if (!root.contains(rec.el)) {
                    return;
                }
                rec.splits.forEach(function (s) {
                    var nodes = Array.prototype.slice.call(s.el.childNodes);
                    var label = s.el.getAttribute("aria-label");
                    s.el.innerHTML = s.html;
                    setAttr(s.el, "aria-label", s.label);
                    undo.push(function () {
                        while (s.el.firstChild) {
                            s.el.removeChild(s.el.firstChild);
                        }
                        nodes.forEach(function (n) {
                            s.el.appendChild(n);
                        });
                        setAttr(s.el, "aria-label", label);
                    });
                });
                rec.styles.forEach(function (x) {
                    var now = x.node.getAttribute("style");
                    setAttr(x.node, "style", x.style);
                    undo.push(function () {
                        setAttr(x.node, "style", now);
                    });
                });
                rec.claimed.forEach(function (c) {
                    c.removeAttribute(INIT);
                    undo.push(function () {
                        c.setAttribute(INIT, "true");
                    });
                });
            });
        }
        var spacers = root.querySelectorAll(".pin-spacer");
        for (var i = 0; i < spacers.length; i++) {
            (function (sp) {
                var child = sp.firstElementChild;
                if (!child || !sp.parentNode) {
                    return;
                }
                sp.parentNode.insertBefore(child, sp);
                sp.parentNode.removeChild(sp);
                undo.push(function () {
                    if (child.parentNode) {
                        child.parentNode.insertBefore(sp, child);
                        sp.appendChild(child);
                    }
                });
            })(spacers[i]);
        }
        return function () {
            for (var j = undo.length - 1; j >= 0; j--) {
                undo[j]();
            }
        };
    }

    function setAttr(el, name, value) {
        if (value === null) {
            el.removeAttribute(name);
        } else {
            el.setAttribute(name, value);
        }
    }

    // htmx saves a copy of the page for the Back button right after this event.
    // Remove the GSAP state for that copy only, and put it back in a microtask.
    // The page does not change on screen, and Back gets clean content to animate.
    document.addEventListener("htmx:beforeHistorySave", function (e) {
        var elt = (e.detail && e.detail.historyElt) || document.body;
        var putBack = detach(elt);
        if (typeof queueMicrotask === "function") {
            queueMicrotask(putBack);
        } else {
            Promise.resolve().then(putBack);
        }
    });

    // A reduced-motion change after load: revert the animations (content stays visible),
    // or start them again.
    if (typeof window.matchMedia === "function") {
        var mq = window.matchMedia("(prefers-reduced-motion: reduce)");
        var onMotionChange = function () {
            if (mq.matches) {
                collect(document, "[" + INIT + "]").forEach(function (el) {
                    if (!optedIn(el) && release(el, true)) {
                        dirty = true;
                    }
                });
                refreshSoon();
            } else {
                scan(document);
                refreshSoon();
            }
        };
        if (typeof mq.addEventListener === "function") {
            mq.addEventListener("change", onMotionChange);
        }
    }

    // A pin adds height after the browser restores the scroll position on reload.
    // Keep the position in sessionStorage and set it again after the first scan.
    function scrollKey() {
        var l = window.location;
        return "autumn-gsap-scroll:" + (l ? l.pathname + l.search : "");
    }

    function storage() {
        try {
            return window.sessionStorage;
        } catch (e) {
            return null;
        }
    }

    function onPageHide() {
        var s = storage();
        if (s) {
            try {
                s.setItem(scrollKey(), String(window.scrollY));
            } catch (e) {
                // Storage is full or blocked. The browser restores the position.
            }
        }
    }

    if (typeof window.addEventListener === "function") {
        window.addEventListener("pagehide", onPageHide);
    }

    function restoreScroll() {
        var s = storage();
        var nav = window.performance && performance.getEntriesByType
            ? performance.getEntriesByType("navigation")[0]
            : null;
        var saved = s ? s.getItem(scrollKey()) : null;
        if (!nav || nav.type !== "reload" || saved === null || window.location.hash) {
            return;
        }
        var y = parseFloat(saved);
        if (isFinite(y) && typeof window.ScrollTrigger === "function") {
            window.ScrollTrigger.refresh();
            window.scrollTo(0, y);
        }
    }

    function boot() {
        scan(document);
        restoreScroll();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", boot);
    } else {
        boot();
    }
})();
