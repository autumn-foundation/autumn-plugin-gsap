//! [`GsapPlugin`]: installs the GSAP assets in an Autumn app.

use std::borrow::Cow;

use autumn_web::app::AppBuilder;
use autumn_web::plugin::Plugin;

use crate::assets::GSAP_ASSETS;

/// The plugin name in Autumn diagnostics.
pub const PLUGIN_NAME: &str = "autumn-plugin-gsap";

/// Installs the GSAP asset bundle ([`GSAP_ASSETS`](crate::GSAP_ASSETS)).
///
/// ```rust,no_run
/// use autumn_plugin_gsap::GsapPlugin;
///
/// # async fn run() {
/// autumn_web::app().plugin(GsapPlugin::new()).run().await;
/// # }
/// ```
///
/// Then put [`gsap_script`](crate::gsap_script) in the page.
#[derive(Debug, Default, Clone, Copy)]
#[must_use]
pub struct GsapPlugin;

impl GsapPlugin {
    /// Makes the plugin. It reads no configuration.
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for GsapPlugin {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLUGIN_NAME)
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.plugin_assets(&GSAP_ASSETS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{
        GSAP_ASSETS, GSAP_CSS, GSAP_JS, INIT_JS, SCROLL_TRIGGER_JS, SPLIT_TEXT_JS,
    };
    use autumn_web::assets::{PLUGIN_ASSETS_ROUTE_MARKER, asset_url};
    use autumn_web::plugin_conformance::{ConformanceConfig, run_conformance};
    use autumn_web::route_listing::{RouteClassification, RouteSource};
    use autumn_web::test::{TestApp, TestClient};

    const JS: &str = "text/javascript; charset=utf-8";
    const CSS: &str = "text/css; charset=utf-8";
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";
    const REVALIDATE: &str = "public, max-age=0, must-revalidate";
    const ALL: [&str; 5] = [GSAP_JS, SCROLL_TRIGGER_JS, SPLIT_TEXT_JS, INIT_JS, GSAP_CSS];

    fn client() -> TestClient {
        TestApp::new().plugin(GsapPlugin::new()).build()
    }

    #[test]
    fn name_is_the_crate_name() {
        assert_eq!(GsapPlugin::new().name(), PLUGIN_NAME);
    }

    #[tokio::test]
    async fn every_file_serves_at_its_fingerprinted_url() {
        let client = client();
        for path in ALL {
            let css = std::path::Path::new(path)
                .extension()
                .is_some_and(|e| e == "css");
            let want = if css { CSS } else { JS };
            let response = client.get(&GSAP_ASSETS.url(path)).send().await;
            response
                .assert_ok()
                .assert_header("content-type", want)
                .assert_header("cache-control", IMMUTABLE);
            let bytes = GSAP_ASSETS.get(path).expect("bundled").bytes();
            assert_eq!(response.body.as_slice(), bytes, "{path}");
        }
    }

    #[tokio::test]
    async fn plain_urls_serve_with_revalidation_and_etags() {
        let client = client();
        for path in ALL {
            let plain = format!("/static/_plugins/gsap/{path}");
            let response = client.get(&plain).send().await;
            response
                .assert_ok()
                .assert_header("cache-control", REVALIDATE);
            let etag = response.header("etag").expect("etag").to_owned();
            client
                .get(&plain)
                .header("if-none-match", &etag)
                .send()
                .await
                .assert_status(304);
        }
    }

    #[tokio::test]
    async fn unbundled_and_stale_paths_are_not_found() {
        let client = client();
        for path in [
            "/static/_plugins/gsap/manifest.json",
            "/static/_plugins/gsap/init.00000000.js",
            "/static/_plugins/gsap/Flip.min.js",
        ] {
            client.get(path).send().await.assert_status(404);
        }
    }

    #[tokio::test]
    async fn asset_url_resolves_the_installed_bundle() {
        let _client = client();
        for path in ALL {
            assert_eq!(
                asset_url(&format!("_plugins/gsap/{path}")),
                GSAP_ASSETS.url(path)
            );
        }
    }

    #[test]
    fn bundle_routes_are_public_plugin_routes() {
        let app = autumn_web::app().plugin(GsapPlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let routes: Vec<_> = infos
            .iter()
            .filter(|info| info.path.starts_with("/static/_plugins/gsap/"))
            .collect();
        assert_eq!(routes.len(), 10, "five files, two URLs each: {infos:?}");
        for info in routes {
            assert_eq!(info.method, "GET");
            assert_eq!(info.classification, RouteClassification::Public);
            assert_eq!(info.middleware, [PLUGIN_ASSETS_ROUTE_MARKER]);
            assert_eq!(info.source, RouteSource::Plugin(PLUGIN_NAME.to_owned()));
        }
    }

    #[test]
    fn plugin_passes_conformance() {
        let app = autumn_web::app().plugin(GsapPlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let report = run_conformance(&ConformanceConfig::new(PLUGIN_NAME), &infos);
        assert!(report.passed(), "{}", report.to_text_report());
    }

    #[tokio::test]
    async fn installing_the_plugin_twice_is_harmless() {
        let client = TestApp::new()
            .plugin(GsapPlugin::new())
            .plugin(GsapPlugin::new())
            .build();
        client
            .get(&GSAP_ASSETS.url(INIT_JS))
            .send()
            .await
            .assert_ok();
    }
}
