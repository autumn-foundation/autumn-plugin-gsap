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

    // Value grammar. Keep these lines equal to src/grammar.rs (a Rust test checks it).
    var RE_SECS = /^(\d+(\.\d+)?)$/;
    var RE_NUM = /^(-?\d+(\.\d+)?)$/;
    var RE_EASE = /^(none|(power[1-4]|sine|expo|circ|bounce)\.(in|out|inOut)|back\.(in|out|inOut)(\(-?\d+(\.\d+)?\))?|elastic\.(in|out|inOut)(\(-?\d+(\.\d+)?,-?\d+(\.\d+)?\))?|steps\(\d+\))$/;
    var RE_SCROLL_POS = /^((top|center|bottom|-?\d+(\.\d+)?(px|%)) (top|center|bottom|-?\d+(\.\d+)?(px|%))|\+=\d+(\.\d+)?(px|%)?)$/;
    var RE_POSITION = /^(<|>|[<>]-?\d+(\.\d+)?|[+-]=\d+(\.\d+)?|\d+(\.\d+)?)$/;
    var RE_INDEX = /^\d+$/;

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
    // End states for presets that GSAP cannot read from CSS.
    var PRESET_ENDS = { "blur-in": { filter: "blur(0px)" } };
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
    };

    function known(name) {
        return Object.prototype.hasOwnProperty.call(PARSERS, name);
    }

    function attr(name, raw) {
        return known(name) ? PARSERS[name](raw) : undefined;
    }

    function warn(el, name, raw) {
        if (typeof console !== "undefined" && console.warn) {
            console.warn("[autumn-plugin-gsap] ignored " + name + '="' + raw + '"', el);
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

    // The ScrollTrigger vars, or null when the element plays on load.
    function scrollTrigger(el) {
        if (read(el, "data-gsap-on") === "load") {
            return null;
        }
        var st = { trigger: resolveTrigger(el) };
        var start = read(el, "data-gsap-start");
        var end = read(el, "data-gsap-end");
        var scrubV = read(el, "data-gsap-scrub");
        if (scrubV !== null) {
            st.scrub = scrubV;
            st.start = start || DEFAULTS.scrubStart;
            st.end = end || DEFAULTS.scrubEnd;
        } else {
            st.start = start || DEFAULTS.start;
            if (end) {
                st.end = end;
            }
            var actions = read(el, "data-gsap-toggle-actions");
            var onceV = read(el, "data-gsap-once");
            if (actions) {
                st.toggleActions = actions;
            } else if (onceV === false) {
                st.toggleActions = "play none none reverse";
            } else {
                st.once = true;
            }
        }
        if (read(el, "data-gsap-pin")) {
            st.pin = true;
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
        return { from: assign({}, PRESETS[k]), to: PRESET_ENDS[k] ? assign({}, PRESET_ENDS[k]) : null };
    }

    // The tween targets: split text units, direct children (stagger) or the element.
    function targets(el, rec) {
        var unit = read(el, "data-gsap-split");
        if (unit !== null && window.SplitText) {
            var opts = { type: unit, aria: "auto" };
            if (read(el, "data-gsap-split-mask")) {
                opts.mask = unit;
            }
            var parts = window.SplitText.create(el, opts)[unit];
            if (parts && parts.length) {
                return { list: parts, split: true };
            }
        }
        if (el.hasAttribute("data-gsap-stagger")) {
            var kids = [];
            for (var i = 0; i < el.children.length; i++) {
                var c = el.children[i];
                if (!c.hasAttribute(INIT)) {
                    c.setAttribute(INIT, "true");
                    rec.claimed.push(c);
                    kids.push(c);
                }
            }
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
        var found = box.querySelectorAll("[data-gsap]");
        for (var i = 0; i < found.length; i++) {
            var child = found[i];
            if (child.hasAttribute(INIT) || child.closest(TIMELINE_SEL) !== box) {
                continue;
            }
            child.setAttribute(INIT, "true");
            rec.claimed.push(child);
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
    var registered = false;

    function register(gsap) {
        if (registered) {
            return;
        }
        registered = true;
        var plugins = [window.ScrollTrigger, window.SplitText].filter(Boolean);
        if (plugins.length) {
            gsap.registerPlugin.apply(gsap, plugins);
        }
    }

    function start(gsap, el, fn) {
        var rec = { claimed: [el], ctx: null };
        el.setAttribute(INIT, "true");
        try {
            rec.ctx = gsap.context(function () {
                fn(gsap, el, rec);
            });
        } catch (e) {
            warn(el, "data-gsap", String(e && e.message ? e.message : e));
        }
        if (RECORDS && rec.ctx) {
            RECORDS.set(el, rec);
        }
    }

    function optedIn(el) {
        return read(el, "data-gsap-reduced") === "animate";
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
        if (!gsap) {
            return 0;
        }
        register(gsap);
        var scope = root && root.querySelectorAll ? root : document;
        var reduce = reducedMotion();
        var count = 0;
        collect(scope, TIMELINE_SEL).forEach(function (box) {
            if (box.hasAttribute(INIT) || (reduce && !optedIn(box))) {
                return;
            }
            start(gsap, box, animateTimeline);
            count += 1;
        });
        collect(scope, "[data-gsap]").forEach(function (el) {
            if (el.hasAttribute(INIT) || el.closest(TIMELINE_SEL) || (reduce && !optedIn(el))) {
                return;
            }
            start(gsap, el, animateElement);
            count += 1;
        });
        return count;
    }

    function release(el) {
        var rec = RECORDS && RECORDS.get(el);
        if (!rec) {
            return false;
        }
        RECORDS.delete(el);
        rec.ctx.revert();
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
            if (release(el)) {
                count += 1;
            }
        });
        return count;
    }

    var refreshTimer = null;

    function refreshSoon() {
        if (!window.ScrollTrigger) {
            return;
        }
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
        release(eventElement(e));
    });
    document.addEventListener("htmx:afterSettle", refreshSoon);

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", function () {
            scan(document);
        });
    } else {
        scan(document);
    }
})();
