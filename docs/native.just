# Blitz (dioxus-native) recipes - the docs app in a winit window instead of a
# browser tab. Imported by `justfile`; `root`, `dx`, `fork` and `_dx` come
# from there.
#
# The renderer picks the `native` feature (which turns on `dioxus/native`);
# fork mode still needs its own flag, so the two are concatenated rather than
# either replacing the other - unlike `split`, which is web-only.
#
# Default is `native-vello`, not bare `native`: dioxus-native's own default
# renderer panics with `AtlasLimitReached` after enough pages - see
# docs/Cargo.toml. `start-native-hybrid` is the one that still uses it.

# winit *guesses* a 1.5 scale factor on X11, and Blitz clips 1px borders at
# fractional scale - so pin it. Ignored outside X11.
x11 := "WINIT_X11_SCALE_FACTOR=1"

nfeat := "--features " + if fork == "1" { "native-vello,dioxus-fork" } else { "native-vello" }
nhybrid := "--features " + if fork == "1" { "native,dioxus-fork" } else { "native" }

# Compile-only check of the native build - faster than a full serve.
check-native:
    cd {{root}} && cargo check -p docs {{nfeat}}

# Hot-reloading serve in a Blitz window.
start-native: _dx
    {{x11}} {{dx}} serve --renderer native --debug-symbols=false {{nfeat}}

# dioxus-native's default renderer: lighter on the GPU, but its image atlas
# is fixed-size and never evicts, so it eventually panics. For comparison.
start-native-hybrid: _dx
    {{x11}} {{dx}} serve --renderer native --debug-symbols=false {{nhybrid}}

# Optimised end to end, via the workspace's `native` profile - `release`
# without its wasm-size trade-offs. Slow to build, and no hot reload.
start-native-release: _dx
    {{x11}} {{dx}} serve --renderer native --debug-symbols=false --profile native {{nfeat}}

# The shortest path to a window (no `dx`, no hot reload, no devserver) - use
# it when a failure might be `dx`'s rather than the app's.
run-native:
    cd {{root}} && {{x11}} cargo run -p docs {{nfeat}}

build-native: _dx
    {{dx}} build --renderer native --debug-symbols=false {{nfeat}}

build_release-native: _dx
    {{dx}} build --renderer native --debug-symbols=false --profile native {{nfeat}}

# --- CPU renderer -----------------------------------------------------------
# `native-cpu` replaces `native` rather than adding to it (see docs/Cargo.toml),
# so these pass their own feature set.

ncpu := "--features " + if fork == "1" { "native-cpu,dioxus-fork" } else { "native-cpu" }

check-native-cpu:
    cd {{root}} && cargo check -p docs {{ncpu}}

# `dx --renderer native` always adds the crate feature it finds enabling
# `dioxus/native` (the CLI's build/renderer.rs), which turns the GPU renderer
# back on beside the CPU one - a combination dioxus-native can't compile. So
# no hot-reloading equivalent of this recipe exists.
#
# A window on a machine with no usable Vulkan adapter.
run-native-cpu:
    cd {{root}} && {{x11}} cargo run -p docs {{ncpu}}
