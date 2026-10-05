use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    LiberoContext,
    outlets::{StyleOutlet, ThemeStyle},
};
use crate::{
    context::{ModalHost, PortalHost, PortalOutlet, WindowHost, window::ZLayers},
    localization::{Formats, Localization},
    platform::{
        self, A11yAnswers, a11y_media, answers_a11y_media, apply_direction, color_scheme,
        set_current_a11y_answers, set_root_direction, stored_direction,
    },
    theme::ThemeSet,
    tokens::{ColorScheme, ColorSchemeSetting, Direction},
};

/// The root every libero app renders once: themes, localization, stylesheets,
/// portals and the modal stack.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{LiberoProvider, localization::{Formats, Localization}};
/// # fn app() -> Element {
/// // English words, German dates: `14. September 2026`, `15:30`.
/// rsx! {
///     LiberoProvider {
///         localization: &Localization::ENGLISH,
///         formats: &Formats::GERMAN,
///         "Hello"
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/about/getting-started>
#[component]
pub fn LiberoProvider(
    /// Every theme the app ships; the light and dark halves share one sheet.
    /// A lone `&'static Theme` is a set of one. Default [`ThemeSet::DEFAULT`]. Read
    /// at mount; swap it later with [`use_theme_set`](crate::hooks::use_theme_set).
    #[props(default, into)]
    themes: ThemeSet,
    /// Every string libero shows a reader. Read at mount; switch it later with
    /// [`use_localization_handle`](crate::hooks::use_localization_handle).
    #[props(default = &Localization::ENGLISH)]
    localization: &'static Localization,
    /// How dates, times and numbers are written, whatever the language. Read at
    /// mount; switch it later with [`use_formats_handle`](crate::hooks::use_formats_handle).
    #[props(default = &Formats::AMERICAN)]
    formats: &'static Formats,
    /// The start text direction, set as the root's `dir`. A choice made through
    /// [`use_direction`](crate::hooks::use_direction) is kept on the web and wins.
    /// Read at mount; turn it later with that hook.
    #[props(default, into)]
    direction: Option<Direction>,
    children: Element,
) -> Element {
    let themes = use_hook(|| themes.clone());
    let outer_css =
        use_hook(|| try_consume_context::<LiberoContext>().map(|outer| outer.theme_css));
    // Only the outermost provider owns the root's `lang`: two would race for it.
    let outermost = outer_css.is_none();
    let localization = use_signal(|| localization);
    let formats = use_signal(|| formats);
    // Read at mount, so the first render paints the kept scheme, not a light flash.
    let setting = use_hook(|| {
        color_scheme()
            .and_then(|platform| platform.stored())
            .unwrap_or_default()
    });
    let system =
        use_hook(|| color_scheme().map_or(ColorScheme::Light, |platform| platform.system()));
    let scheme = setting.resolve(system);

    let scheme_setting = use_signal(|| setting);
    let system_scheme = use_signal(|| system);
    let active = use_signal(|| scheme.as_str());
    let theme_signal = use_signal({
        let themes = themes.clone();
        move || {
            themes
                .get(scheme.as_str())
                .unwrap_or_else(|| themes.light_theme())
        }
    });
    let theme_css = use_signal({
        let themes = themes.clone();
        move || themes.sheet_css()
    });
    let theme_set = use_signal(|| themes);
    // Set before the first render where the root is in reach (the web).
    let kept_direction = use_signal(stored_direction);
    let start_direction = use_hook(|| direction);
    let chosen_direction = use_hook(|| kept_direction.peek().or(start_direction));
    let direction_set = use_hook(|| chosen_direction.is_some_and(set_root_direction));
    let direction_signal = use_signal(|| chosen_direction.unwrap_or_default());

    // Natively at once, so the first frame already answers; on the web in an
    // effect below, as a server cannot know it.
    let accessibility_system = use_signal(|| {
        a11y_media()
            .filter(|_| answers_a11y_media())
            .map(|platform| platform.system())
            .unwrap_or_default()
    });
    // A kept choice. The web reads it after mount: it rewrites the sheets' text,
    // which a hydrating client must render as the server did.
    let forced_reduced_motion = use_signal(|| {
        a11y_media()
            .filter(|_| !cfg!(target_arch = "wasm32"))
            .and_then(|platform| platform.stored_reduced_motion())
    });

    let stylesheet_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| {
        LiberoContext::new(
            theme_set,
            theme_signal,
            localization,
            formats,
            active,
            scheme_setting,
            system_scheme,
            direction_signal,
            kept_direction,
            start_direction,
            accessibility_system,
            forced_reduced_motion,
            theme_css,
            stylesheet_registry_version,
        )
    });
    use_hook(|| {
        set_current_a11y_answers(A11yAnswers::new(
            *accessibility_system.peek(),
            *forced_reduced_motion.peek(),
        ))
    });
    // Every later change of the platform's settings, held for the provider's lifetime.
    let _accessibility_subscription = use_hook(|| {
        let context = context.clone();
        Rc::new(a11y_media().map(|platform| {
            platform.on_change(Box::new(move |system| {
                context.set_accessibility_system(system)
            }))
        }))
    });
    use_effect({
        let context = context.clone();
        move || {
            if let Some(platform) = a11y_media().filter(|_| !answers_a11y_media()) {
                context.set_accessibility_system(platform.system());
                // A choice made during the first render wins over the kept one.
                if cfg!(target_arch = "wasm32")
                    && context.forced_reduced_motion.peek().is_none()
                    && let Some(kept) = platform.stored_reduced_motion()
                {
                    context.force_reduced_motion(Some(kept));
                }
            }
        }
    });
    // Elsewhere after mount: Blitz reaches its root through an element the provider renders.
    // The latest choice, not the mount's: a `set` during the first render found no root (956).
    let kept = context.kept_direction;
    use_effect(move || {
        let current = kept.peek().or(start_direction);
        if let Some(direction) = current.filter(|_| !direction_set || current != chosen_direction) {
            apply_direction(direction);
        }
    });

    // After mount and on every switch, so a screen reader speaks the words in their language.
    use_effect(move || {
        if outermost {
            platform::apply_lang(localization().lang);
        }
    });

    // A platform scheme change reaches the Rust side too; the CSS follows on its own.
    let _scheme_subscription = use_hook(|| {
        let context = context.clone();
        // `Rc`: a hook value must be `Clone`, and a subscription is not.
        Rc::new(color_scheme().map(|platform| {
            platform.on_change(Box::new(move |scheme| {
                let mut system = context.system_scheme;
                system.set(scheme);
                if *context.scheme_setting.peek() == ColorSchemeSetting::System {
                    context.follow_system(scheme);
                }
            }))
        }))
    });

    // A stored choice must reach the root, or the media block keeps answering.
    // The inline script does it before paint; this covers a client-only build.
    use_hook(|| {
        if let Some(pinned) = setting.fixed() {
            context.set_color_scheme(pinned.into());
        }
    });

    let portal_entries = use_signal(Vec::new);
    use_context_provider(|| PortalHost::new(portal_entries));

    // From the active theme, so a switch to other `z_index` values moves the stacks too.
    let modal_layers = use_memo(move || {
        let z = theme_signal().z_index;
        ZLayers {
            base: z.modal,
            step: z.modal_step,
            ceiling: z.popover,
        }
    });
    let modal_stack = use_signal(Vec::new);
    use_context_provider(|| ModalHost::new(modal_stack, modal_layers.into()));

    let window_layers = use_memo(move || {
        let z = theme_signal().z_index;
        ZLayers {
            base: z.window,
            step: z.window_step,
            ceiling: z.overlay,
        }
    });
    let window_stack = use_signal(Vec::new);
    use_context_provider(|| WindowHost::new(window_stack, window_layers.into()));

    rsx! {
        style {
            dangerous_inner_html: "{context.layer_order_css}"
        }
        ThemeStyle { outer: outer_css }
        {platform::Listener(rsx! {
            {children}
            PortalOutlet {}
        })}
        StyleOutlet {}
        platform::Outlet {}
    }
}
