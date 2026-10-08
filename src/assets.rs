//! Vendored GSAP files, embedded at compile time.
//!
//! [`GSAP_ASSETS`] holds the upstream GSAP builds and the plugin files.
//! [`GsapPlugin`](crate::GsapPlugin) installs it through the Autumn
//! `AppBuilder::plugin_assets` seam. The framework serves each file under
//! `/static/_plugins/gsap/` at a hashed URL (immutable) and a plain URL
//! (`must-revalidate`), with `ETag`/`304`, `Range` and a computed `sha384` SRI hash.
//!
//! The bundle lists its files one by one. Thus `manifest.json` is not served.

use autumn_web::assets::PluginAssets;

/// URL namespace: files serve under `/static/_plugins/gsap/`.
pub const ASSETS_NAMESPACE: &str = "gsap";

/// GSAP core (upstream, sets `window.gsap`).
pub(crate) const GSAP_JS: &str = "gsap.min.js";
/// ScrollTrigger plugin (upstream).
pub(crate) const SCROLL_TRIGGER_JS: &str = "ScrollTrigger.min.js";
/// SplitText plugin (upstream).
pub(crate) const SPLIT_TEXT_JS: &str = "SplitText.min.js";
/// The declarative scanner (plugin file).
pub(crate) const INIT_JS: &str = "init.js";
/// Default styles (plugin file).
pub(crate) const GSAP_CSS: &str = "gsap.css";

/// The asset bundle: GSAP, ScrollTrigger, SplitText, `init.js` and `gsap.css`.
///
/// ```rust
/// use autumn_plugin_gsap::GSAP_ASSETS;
///
/// let url = GSAP_ASSETS.url("init.js");
/// assert!(url.starts_with("/static/_plugins/gsap/init."), "{url}");
/// assert!(GSAP_ASSETS.integrity("gsap.min.js").is_some());
/// ```
pub static GSAP_ASSETS: PluginAssets = PluginAssets::from_files(
    ASSETS_NAMESPACE,
    &[
        (GSAP_JS, include_bytes!("../assets/gsap.min.js")),
        (
            SCROLL_TRIGGER_JS,
            include_bytes!("../assets/ScrollTrigger.min.js"),
        ),
        (SPLIT_TEXT_JS, include_bytes!("../assets/SplitText.min.js")),
        (INIT_JS, include_bytes!("../assets/init.js")),
        (GSAP_CSS, include_bytes!("../assets/gsap.css")),
    ],
);

/// The vendored GSAP version.
pub const GSAP_VERSION: &str = "3.15.0";

/// The GSAP license of the vendored files. The plugin code is Apache-2.0.
pub const GSAP_LICENSE: &str = "https://gsap.com/standard-license";

/// The upstream source of each vendored file, with its pinned `sha384` SRI hash.
///
/// The `<script>` tags do not read these values. A test checks that the
/// embedded bytes still match them.
pub const GSAP_UPSTREAM: [(&str, &str, &str); 3] = [
    (
        GSAP_JS,
        "https://cdn.jsdelivr.net/npm/gsap@3.15.0/dist/gsap.min.js",
        "sha384-XmJ9SoHtVOHoQUcKvFAzVXwdkKo1Ie3bhmSoIAkcdsHGaIrVJIkmozyq0FJeb/Ly",
    ),
    (
        SCROLL_TRIGGER_JS,
        "https://cdn.jsdelivr.net/npm/gsap@3.15.0/dist/ScrollTrigger.min.js",
        "sha384-wl5TeDVvOWt30Pbf8aSo2ZrzsOjddu3avOBvHe+p+OhJt9gP6w9YXmDkN5DK2/dF",
    ),
    (
        SPLIT_TEXT_JS,
        "https://cdn.jsdelivr.net/npm/gsap@3.15.0/dist/SplitText.min.js",
        "sha384-SWJ0lLVRoipvHh59xj0pL7uC7Ih51F+5smaFtrG+2nr+TlDZU5SYJHmxfolbeNTr",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use sha2::{Digest as _, Sha384};

    fn sri(bytes: &[u8]) -> String {
        let digest = Sha384::digest(bytes);
        format!(
            "sha384-{}",
            base64::engine::general_purpose::STANDARD.encode(digest)
        )
    }

    #[test]
    fn bundle_holds_exactly_the_served_files() {
        let files: Vec<&str> = GSAP_ASSETS
            .iter()
            .map(autumn_web::assets::PluginAsset::logical_path)
            .collect();
        assert_eq!(
            files,
            [SCROLL_TRIGGER_JS, SPLIT_TEXT_JS, GSAP_CSS, GSAP_JS, INIT_JS],
            "sorted by path; manifest.json is absent"
        );
        assert_eq!(GSAP_ASSETS.namespace(), ASSETS_NAMESPACE);
        assert_eq!(GSAP_ASSETS.mount_path(), "/static/_plugins/gsap");
    }

    #[test]
    fn plugin_files_are_not_empty() {
        for path in [INIT_JS, GSAP_CSS] {
            let asset = GSAP_ASSETS.get(path).expect("bundled");
            assert!(asset.bytes().len() > 100, "{path} has content");
        }
    }

    #[test]
    fn bundle_integrity_matches_embedded_bytes() {
        for asset in GSAP_ASSETS.iter() {
            assert_eq!(
                asset.integrity(),
                sri(asset.bytes()),
                "{}",
                asset.logical_path()
            );
        }
    }

    #[test]
    fn vendored_files_match_the_pinned_upstream_hashes() {
        for (path, source, pin) in GSAP_UPSTREAM {
            let asset = GSAP_ASSETS.get(path).expect("bundled");
            assert_eq!(sri(asset.bytes()), pin, "{path}");
            assert!(
                source.contains(&format!("gsap@{GSAP_VERSION}/dist/{path}")),
                "{source}"
            );
        }
    }

    #[test]
    fn vendored_files_keep_the_upstream_license_header() {
        for (path, _, _) in GSAP_UPSTREAM {
            let bytes = GSAP_ASSETS.get(path).expect("bundled").bytes();
            let head = String::from_utf8_lossy(&bytes[..300]);
            assert!(head.contains(GSAP_VERSION), "{path}: {head}");
            assert!(head.contains(GSAP_LICENSE), "{path}: {head}");
        }
    }

    #[test]
    fn urls_are_fingerprinted_under_the_plugin_mount() {
        for asset in GSAP_ASSETS.iter() {
            let path = asset.logical_path();
            assert_eq!(asset.plain_url(), format!("/static/_plugins/gsap/{path}"));
            let (stem, ext) = path.rsplit_once('.').expect("extension");
            let url = asset.url();
            let hash = url
                .strip_prefix(&format!("/static/_plugins/gsap/{stem}."))
                .and_then(|rest| rest.strip_suffix(&format!(".{ext}")))
                .unwrap_or_else(|| panic!("{url} is the hashed form of {path}"));
            assert_eq!(hash.len(), 8, "{url}");
            assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()), "{url}");
        }
    }

    #[test]
    fn content_types_match_the_files() {
        for asset in GSAP_ASSETS.iter() {
            let want = if std::path::Path::new(asset.logical_path())
                .extension()
                .is_some_and(|e| e == "css")
            {
                "text/css; charset=utf-8"
            } else {
                "text/javascript; charset=utf-8"
            };
            assert_eq!(asset.content_type(), want, "{}", asset.logical_path());
        }
    }

    #[test]
    fn manifest_agrees_with_constants() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../assets/manifest.json")).expect("json");
        assert_eq!(manifest["version"], GSAP_VERSION);
        assert_eq!(manifest["license"], GSAP_LICENSE);
        for (path, source, pin) in GSAP_UPSTREAM {
            let entry = &manifest["files"][path];
            assert_eq!(entry["source"], source, "{path}");
            assert_eq!(entry["integrity"], pin, "{path}");
            let bytes = GSAP_ASSETS.get(path).expect("bundled").bytes().len();
            assert_eq!(entry["bytes"], bytes, "{path}");
        }
    }
}
