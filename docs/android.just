# Android recipes - the docs app in the emulator's WebView. Imported by
# `justfile`; `root`, `dx` and `_dx` come from there.
#
# Unlike native/Blitz there is no renderer to choose: `dioxus/mobile` is
# wry, so the pages are laid out by Android's own WebView and behave like the
# web build. The `mobile` feature only turns that on - `dx --platform android`
# still has to be passed, and picks the feature up itself.

sdk := env_var_or_default("ANDROID_HOME", env_var("HOME") / "Android/Sdk")
# `dx` finds the NDK the same way (newest under `$sdk/ndk`), but only once
# ANDROID_HOME is exported - which it isn't in this shell, hence both.
ndk := `ls -d "${ANDROID_HOME:-$HOME/Android/Sdk}"/ndk/* 2>/dev/null | sort -V | tail -1`
avd := "libero-docs"
# The image `just android-avd` creates the AVD from. x86_64 so it runs on the
# host CPU instead of being emulated instruction by instruction.
image := "system-images;android-34;google_apis;x86_64"

# Exported for `dx`, gradle and the emulator alike.
aenv := "ANDROID_HOME=" + sdk + " ANDROID_SDK_ROOT=" + sdk + " ANDROID_NDK_HOME=" + ndk

mfeat := "--features mobile"

# What's installed, what's missing, and whether an emulator is up.
android-doctor:
    @echo "sdk:      {{sdk}}"
    @echo "ndk:      {{ if ndk == "" { "MISSING - sdkmanager 'ndk;27.3.13750724'" } else { ndk } }}"
    @echo "java:     $(java -version 2>&1 | head -1)"
    @echo "targets:  $(rustup target list --installed | grep android | tr '\n' ' ')"
    @echo "avd:      $({{sdk}}/cmdline-tools/latest/bin/avdmanager list avd -c 2>/dev/null | tr '\n' ' ')"
    @echo "devices:  $(adb devices | tail -n +2 | tr '\n' ' ')"

# One-off: the emulator image `just android-emulator` boots.
android-avd:
    {{sdk}}/cmdline-tools/latest/bin/sdkmanager "{{image}}"
    echo no | {{sdk}}/cmdline-tools/latest/bin/avdmanager create avd -n {{avd}} -k "{{image}}" -d pixel_6 --force

# Boots the emulator and stays in the foreground - `start-android` needs it
# already running, so this belongs in its own terminal. Without a host GPU
# use `gpu=swangle_indirect`: `swiftshader_indirect` (and `guest`, which falls
# back to it) segfaults qemu every ~10 min under the WebView (995).
android-emulator gpu="host":
    {{aenv}} {{sdk}}/emulator/emulator -avd {{avd}} -gpu {{gpu}} -no-boot-anim

# Blocks until the booted emulator is ready to be installed onto.
android-wait:
    adb wait-for-device shell 'while [ "$(getprop sys.boot_completed)" != 1 ]; do sleep 1; done'

# Compile-only check of the mobile build, against the emulator's arch.
check-android:
    cd {{root}} && {{aenv}} cargo check -p docs --target x86_64-linux-android {{mfeat}}

# `dx` has no icon setting and rewrites its stock launcher icon on every build.
# A build type's gradle source set outranks `main` and `dx` never writes it, so
# the Libero logo goes there: `debug` (dx always packages that variant) and
# `release` (gradle's own). Create it before `dx` runs.
_android-icon profile:
    #!/usr/bin/env bash
    set -euo pipefail
    app="${CARGO_TARGET_DIR:-{{root}}/target/main}/dx/docs/{{profile}}/android/app/app/src"
    # Older runs copied these into `main`, where they clash with the variants' ("Duplicate resources").
    (cd "{{root}}/docs/android/res" && find . -type f) | while read -r f; do rm -f "$app/main/res/$f"; done
    for variant in debug release; do
        mkdir -p "$app/$variant/res"
        cp -r "{{root}}/docs/android/res/." "$app/$variant/res"
    done

# Build, install and run on whatever `adb devices` shows, with hot reload.
start-android: _dx android-wait (_android-icon "debug")
    cd {{root}}/docs && {{aenv}} {{dx}} serve --platform android {{mfeat}}

build-android: _dx (_android-icon "debug")
    cd {{root}}/docs && {{aenv}} {{dx}} build --platform android {{mfeat}}

# Signing key for the sideloaded APK. Created on first use, outside the repo;
# keep it, since an update signed with another key has to uninstall first.
keystore := env_var_or_default("LIBERO_KEYSTORE", env_var("HOME") / ".android/libero-docs.jks")
ks_pass := env_var_or_default("LIBERO_KEYSTORE_PASS", "android")
apk_dir := clean(root / "target/android")

# Signed release APK, arm64 for a phone (`x86_64` for the emulator). `dx` only
# packages gradle's debug variant, so gradle builds the release one itself.
apk-android arch="aarch64": _dx (_android-icon "release")
    #!/usr/bin/env bash
    set -euo pipefail
    cd {{root}}/docs
    app="${CARGO_TARGET_DIR:-{{root}}/target/main}/dx/docs/release/android/app"
    # `dx` keeps the other arch's library from an earlier build.
    rm -rf "$app/app/src/main/jniLibs"
    {{aenv}} {{dx}} build --platform android --release --target {{arch}}-linux-android {{mfeat}}
    # No R8: it strips the Kotlin classes wry reaches only through JNI.
    sed -i 's/isMinifyEnabled = true/isMinifyEnabled = false/' "$app/app/build.gradle.kts"
    (cd "$app" && {{aenv}} ./gradlew -q assembleRelease)
    [ -f "{{keystore}}" ] || keytool -genkeypair -keystore "{{keystore}}" -storepass "{{ks_pass}}" \
        -alias libero-docs -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Libero docs"
    tools=$(ls -d {{sdk}}/build-tools/* | sort -V | tail -1)
    # The commit is in the file name, so a stale APK shows at once; the
    # unstamped name stays as a symlink to the newest build.
    apk="{{apk_dir}}/libero-docs-{{arch}}-$(git -C {{root}} describe --always --dirty).apk"
    mkdir -p "{{apk_dir}}"
    rm -f "{{apk_dir}}"/libero-docs-{{arch}}-*.apk
    "$tools/zipalign" -f -p 4 "$app/app/build/outputs/apk/release/app-release-unsigned.apk" "$apk"
    "$tools/apksigner" sign --ks "{{keystore}}" --ks-pass "pass:{{ks_pass}}" "$apk"
    ln -sf "$(basename "$apk")" "{{apk_dir}}/libero-docs-{{arch}}.apk"
    echo "$apk"

# Regenerates `docs/android/res` from `assets/logo.svg`: an adaptive icon (the
# logo inside the 66dp safe zone of a 108dp canvas) plus square legacy icons.
android-icons:
    #!/usr/bin/env bash
    set -euo pipefail
    cd {{root}}/docs
    for spec in mdpi:48:108 hdpi:72:162 xhdpi:96:216 xxhdpi:144:324 xxxhdpi:192:432; do
        IFS=: read -r density icon fg <<< "$spec"
        dir=android/res/mipmap-$density
        mkdir -p "$dir"
        rsvg-convert -w $((fg * 56 / 100)) assets/logo.svg \
            | magick png:- -background none -gravity center -extent ${fg}x${fg} "$dir/ic_launcher_foreground.png"
        rsvg-convert -w $((icon * 80 / 100)) assets/logo.svg \
            | magick png:- -background white -gravity center -extent ${icon}x${icon} "$dir/ic_launcher.png"
    done

# Installs `apk-android`'s newest file on the one device `adb devices` shows.
install-android arch="aarch64":
    @echo "installing $(readlink "{{apk_dir}}/libero-docs-{{arch}}.apk")"
    adb install -r "{{apk_dir}}/libero-docs-{{arch}}.apk"

# The app's own log lines; a Rust panic surfaces here and nowhere else.
android-logcat:
    adb logcat -v color RustStdoutStderr:V Dioxus:V '*:S'
