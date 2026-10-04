use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    sx::ThemeAwareValue,
    theme::{GlassTint, Gradient, Theme, ThemeSet, glass_gradient_declarations, glass_tint},
};

/// The active theme. Reactive, so a Rust-side read never disagrees with the CSS vars.
/// Panics outside a `LiberoProvider`.
///
/// Docs: <https://libero-ui.dev/about/theming>
pub fn use_theme() -> &'static Theme {
    *use_context::<LiberoContext>().theme.read()
}

/// The active [`ThemeSet`], and how to swap it: a theme picker's base.
/// Reactive. A swap rebuilds the stylesheet; the colour-scheme setting survives.
/// Panics outside a `LiberoProvider`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{theme::ThemeSet, use_theme_set};
/// # fn app() -> Element {
/// let themes = use_theme_set();
///
/// rsx! {
///     for set in ThemeSet::CATALOGUE {
///         button {
///             onclick: {
///                 let themes = themes.clone();
///                 move |_| themes.set((*set).clone())
///             },
///             "{set.name()}"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-theme-set>
pub fn use_theme_set() -> ThemeSetHandle {
    ThemeSetHandle {
        context: use_context::<LiberoContext>(),
    }
}

/// The inline gradient vars for `gradient` from the component's `color`, `None`
/// unless `active` and either is set (the theme's `:root` vars cover the rest).
/// Measured on both themes of the set, since a scheme switch re-renders nothing.
/// `text` puts palette stops in the text role, for gradient text.
pub(crate) fn use_gradient_style(
    gradient: Option<&Gradient>,
    color: Option<&ThemeAwareValue>,
    active: bool,
    text: bool,
) -> Option<String> {
    let context = use_context::<LiberoContext>();
    let shown = active && (gradient.is_some() || color.is_some());
    let themes = if shown {
        scheme_themes(&context)
    } else {
        vec![]
    };
    use_cached(
        gradient_key(gradient, color, [shown, text], &themes),
        || {
            if !shown {
                return None;
            }
            let gradient = gradient.cloned().unwrap_or_default();
            Some(
                gradient
                    .declarations(color, &themes, text)
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            )
        },
    )
}

/// The gradient vars of [`use_gradient_style`] plus, for a glass surface, the label
/// and share the glass raises them to; `None` under the same conditions.
pub(crate) fn use_glass_gradient_style(
    gradient: Option<&Gradient>,
    color: Option<&ThemeAwareValue>,
    active: bool,
    glass: bool,
) -> Option<String> {
    let style = use_gradient_style(gradient, color, active, false);
    // A hook: called before the early return, or toggling `glass` panics (todo 2134).
    let context = use_context::<LiberoContext>();
    // Read only when used, as before: a theme switch re-renders no plain component.
    let themes = if glass {
        scheme_themes(&context)
    } else {
        vec![]
    };
    // `best_label` measures contrast per stop: 3 ms a Header render without the cache (todo 2148).
    let glass_style = use_cached(gradient_key(gradient, color, [glass], &themes), || {
        if !glass {
            return None;
        }
        let gradient = gradient.cloned().unwrap_or_default();
        let glass = glass_gradient_declarations(&gradient, color, &themes);
        Some(glass.iter().map(ToString::to_string).collect())
    });
    if !glass {
        return style;
    }
    Some(style? + &glass_style?)
}

type GradientKey<const N: usize> = (
    Option<Gradient>,
    Option<ThemeAwareValue>,
    [bool; N],
    Vec<*const Theme>,
);

fn gradient_key<const N: usize>(
    gradient: Option<&Gradient>,
    color: Option<&ThemeAwareValue>,
    flags: [bool; N],
    themes: &[&'static Theme],
) -> GradientKey<N> {
    let themes = themes.iter().map(|theme| *theme as *const Theme).collect();
    (gradient.cloned(), color.cloned(), flags, themes)
}

/// `compute`'s result, rerun only when `key` changes from the last render's.
fn use_cached<K: PartialEq + 'static>(
    key: K,
    compute: impl FnOnce() -> Option<String>,
) -> Option<String> {
    let mut cache = use_hook(|| CopyValue::new(None::<(K, Option<String>)>));
    if let Some((last, value)) = &*cache.peek()
        && *last == key
    {
        return value.clone();
    }
    let value = compute();
    cache.set(Some((key, value.clone())));
    value
}

/// The label and share of a glass tint of `fill` (see [`glass_tint`]), measured
/// as [`use_gradient_style`] is. `None` without a `fill`.
pub(crate) fn use_glass_tint(fill: Option<&ThemeAwareValue>) -> Option<GlassTint> {
    let context = use_context::<LiberoContext>();
    glass_tint(fill?, &scheme_themes(&context))
}

/// The active theme first, then every theme a scheme switch can show.
fn scheme_themes(context: &LiberoContext) -> Vec<&'static Theme> {
    let set = context.themes.read();
    let mut themes = vec![*context.theme.read(), set.light_theme()];
    themes.extend(set.dark_theme());
    themes
}

/// What [`use_theme_set`] hands back.
#[derive(Clone)]
pub struct ThemeSetHandle {
    context: LiberoContext,
}

impl ThemeSetHandle {
    /// The active set. Cloned, because a set owns its list of extra themes.
    pub fn get(&self) -> ThemeSet {
        self.context.themes.read().clone()
    }

    /// What the active set calls itself.
    pub fn name(&self) -> &'static str {
        self.context.themes.read().name()
    }

    /// Swaps the set and rebuilds the stylesheet.
    pub fn set(&self, themes: ThemeSet) {
        self.context.set_theme_set(themes);
    }
}
