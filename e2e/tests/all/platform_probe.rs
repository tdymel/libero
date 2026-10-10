//! A probe, not a feature test: what each WebView offers (secure context, capability APIs,
//! viewport, CSS support, media queries), kept as `platform-probe-<platform>-<host>.json` in
//! the run's artifacts. It answers todos 1227, 1236, 1239 and 1248 per OS; it asserts only
//! what holds everywhere.

use anyhow::{Context, Result, ensure};
use e2e::driver::{Driver, Host, eventually_text};
use serde_json::Value;

/// One object of facts; every read is guarded, so an engine lacking one still answers.
const PROBE: &str = r#"(() => {
    const read = (f) => { try { return f(); } catch (e) { return 'threw: ' + e; } };
    const supports = (property, value) => read(() => CSS.supports(property, value));
    const matches = (query) => read(() => matchMedia(query).matches);
    const style = (id) => getComputedStyle(document.getElementById(id));
    return {
        origin: location.origin,
        href: location.href,
        isSecureContext: window.isSecureContext ?? null,
        typeof: {
            mediaDevices: typeof navigator.mediaDevices,
            getUserMedia: typeof navigator.mediaDevices?.getUserMedia,
            Notification: typeof window.Notification,
            geolocation: typeof navigator.geolocation,
            clipboard: typeof navigator.clipboard,
            serviceWorker: typeof navigator.serviceWorker,
            visualViewport: typeof window.visualViewport,
            permissions: typeof navigator.permissions,
            cryptoSubtle: typeof window.crypto?.subtle,
            PushManager: typeof window.PushManager,
        },
        notificationPermission: read(() => window.Notification?.permission ?? null),
        fullscreenEnabled: document.fullscreenEnabled ?? null,
        webkitFullscreenEnabled: document.webkitFullscreenEnabled ?? null,
        viewportMeta: document.querySelector('meta[name=viewport]')?.content ?? null,
        safeAreaInsetTop: parseFloat(style('inset-top').paddingTop),
        userSelect: { computed: style('no-select').userSelect ?? null, webkit: style('no-select').webkitUserSelect ?? null },
        devicePixelRatio: window.devicePixelRatio,
        innerSize: [innerWidth, innerHeight],
        visualViewportScale: window.visualViewport?.scale ?? null,
        navigatorPlatform: navigator.platform,
        userAgent: navigator.userAgent,
        maxTouchPoints: navigator.maxTouchPoints,
        cssSupports: {
            userSelect: supports('user-select', 'none'),
            webkitUserSelect: supports('-webkit-user-select', 'none'),
            backdropFilter: supports('backdrop-filter', 'blur(4px)'),
            webkitBackdropFilter: supports('-webkit-backdrop-filter', 'blur(4px)'),
            dvh: supports('height', '100dvh'),
            safeAreaEnv: supports('padding-top', 'env(safe-area-inset-top)'),
        },
        media: {
            prefersReducedMotion: matches('(prefers-reduced-motion: reduce)'),
            forcedColors: matches('(forced-colors: active)'),
            prefersContrastMore: matches('(prefers-contrast: more)'),
            prefersDark: matches('(prefers-color-scheme: dark)'),
            pointerCoarse: matches('(pointer: coarse)'),
            hover: matches('(hover: hover)'),
        },
        localStorage: read(() => {
            localStorage.setItem('e2e-probe', 'kept');
            const kept = localStorage.getItem('e2e-probe') === 'kept';
            localStorage.removeItem('e2e-probe');
            return kept;
        }),
    };
})()"#;

async fn probe<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#inset-top", "probe", "mount").await?;
    let mut facts = d.evaluate(PROBE).await?;
    let platform = format!("{:?}", d.platform()).to_lowercase();
    let host = format!("{:?}", Host::current()).to_lowercase();
    facts["platform"] = Value::from(platform.as_str());
    facts["host"] = Value::from(host.as_str());
    let line = serde_json::to_string(&facts)?;
    // Written before the asserts, so a red probe still answers.
    if let Some(dir) = e2e::journal::dir() {
        let path = dir.join(format!("platform-probe-{platform}-{host}.json"));
        std::fs::write(&path, &line).with_context(|| format!("write {}", path.display()))?;
    }
    eprintln!("platform probe: {line}");
    ensure!(
        facts["origin"]
            .as_str()
            .is_some_and(|origin| !origin.is_empty()),
        "no origin: {line}"
    );
    ensure!(
        facts["localStorage"] == Value::Bool(true),
        "localStorage round trip: {}",
        facts["localStorage"]
    );
    Ok(())
}

e2e::scenario!(
    the_platform_probe_records_each_engine,
    "/platform-probe",
    probe,
    native: skip("Blitz runs no script")
);
