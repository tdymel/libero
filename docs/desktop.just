# Desktop WebView (wry) recipes - the docs app in a desktop window, laid out by
# the system's WebView. Imported by `justfile`; `root`, `dx` and `_dx` come from there.
#
# A compile-time choice beside `native` (Blitz): the same pages through a browser
# engine, so a Blitz-only glitch shows up as a difference between the two.

dfeat := "--features desktop"

# `--renderer webview` for the same reason native.just names its renderer: two
# feature keys map to a WebView (`desktop`, `mobile`), so dx cannot guess.
dplat := "--platform desktop --renderer webview"

# Compile-only check of the desktop build.
check-desktop:
    cd {{root}} && cargo check -p docs {{dfeat}}

# Hot-reloading serve in a WebView window.
start-desktop: _dx
    {{dx}} serve {{dplat}} --debug-symbols=false {{dfeat}}

# The shortest path to a window (no `dx`, no hot reload). Only dx bundles
# `assets/`, so every `asset!()` image is missing here, as with `run-native`.
run-desktop:
    cd {{root}} && cargo run -p docs {{dfeat}}

build-desktop: _dx
    {{dx}} build {{dplat}} --debug-symbols=false {{dfeat}}
