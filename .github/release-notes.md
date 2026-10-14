The attached files are the [Libero docs](https://libero-ui.dev) built as an app on each platform, a showcase of the components outside the browser. `desktop` renders in the system WebView, `native` with [Blitz](https://github.com/DioxusLabs/blitz). Add libero to your own app from [crates.io](https://crates.io/crates/libero).

The builds are unsigned:

- **macOS**: if macOS says the app is damaged, run `xattr -dr com.apple.quarantine /Applications/<app>.app`, then open it.
- **Windows**: SmartScreen warns about an unknown publisher; choose *More info* > *Run anyway*.
- **Linux**: `chmod +x` the AppImage and run it, or install the `.deb`.
- **Android**: sideload the APK (arm64). Each release has its own signing key, so uninstall an older one first.
- **iOS**: a simulator build only. Unzip it and run `xcrun simctl install booted <app>.app`.
