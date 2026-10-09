//! The fixture app on the web, served by the e2e runner's `dx run`.

/// The desktop (1126) and iOS (2784) drivers' console record, in the head so errors
/// before the app mounts count too.
#[cfg(any(feature = "desktop", feature = "ios"))]
const CONSOLE_HOOK: &str = "<script>{ window.__e2eErrors = [];
    const error = console.error.bind(console);
    console.error = (...args) => { __e2eErrors.push(args.map(String).join(' ')); error(...args); };
    addEventListener('error', e => __e2eErrors.push(String(e.message)));
    addEventListener('unhandledrejection', e => __e2eErrors.push(String(e.reason))); }</script>";

fn main() {
    e2e_fixtures::perf::install();
    // A fresh store per launch, so one scenario's kept scheme never reaches the next;
    // `E2E_STORAGE_DIR` lets a relaunch read what the last launch kept.
    #[cfg(feature = "desktop")]
    libero::platform::set_storage_dir(std::env::var_os("E2E_STORAGE_DIR").map_or_else(
        || std::env::temp_dir().join(format!("e2e-fixtures-storage-{}", std::process::id())),
        std::path::PathBuf::from,
    ));
    #[cfg(feature = "desktop")]
    if std::env::var_os("E2E_BRIDGE").is_some() {
        let config = dioxus::desktop::Config::new().with_custom_head(CONSOLE_HOOK.into());
        return dioxus::LaunchBuilder::new()
            .with_cfg(config)
            .launch(e2e_fixtures::App);
    }
    // `SIMCTL_CHILD_E2E_BRIDGE` on `simctl launch` reaches the app as `E2E_BRIDGE`.
    #[cfg(feature = "ios")]
    if std::env::var_os("E2E_BRIDGE").is_some() {
        let config = dioxus::mobile::Config::new().with_custom_head(CONSOLE_HOOK.into());
        return dioxus::LaunchBuilder::new()
            .with_cfg(config)
            .launch(e2e_fixtures::App);
    }
    dioxus::launch(e2e_fixtures::App);
}
