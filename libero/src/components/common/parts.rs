use std::{fmt, marker::PhantomData, ops::Deref, sync::LazyLock};

use crate::{
    hooks::SxSource,
    sx::{Input, Sx},
};

/// A named inner element of a component, the value of its `data-slot`.
/// Each multi-part component declares one enum of these, e.g. [`AlertPart`](crate::components::AlertPart).
pub trait Part: Copy + 'static {
    /// Every part, in render order.
    const ALL: &'static [Self];

    /// The `data-slot` value.
    fn slot(self) -> &'static str;

    /// The selector relative to the component's root, e.g. `& > [data-slot='icon']`.
    fn selector(self) -> &'static str;
}

/// Styles for a component's inner parts, keyed by its part enum. Compiled into
/// the root's `sx` as part selectors; the instance `sx` wins a tie.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Alert, AlertPart, Parts};
/// # use libero::sx::sx;
/// # fn app() -> Element { rsx! {
/// Alert {
///     title: "Saved",
///     parts: Parts::new()
///         .part(AlertPart::Title, sx().font_weight("700"))
///         .part(AlertPart::Close, sx().color("error.6")),
///     "Your changes are live."
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
pub struct Parts<P: Part> {
    sx: Sx,
    // `fn() -> P`: no `P` bound on the derived-by-hand traits, and `Send + Sync` for a `static`.
    part: PhantomData<fn() -> P>,
}

impl<P: Part> Parts<P> {
    pub fn new() -> Self {
        Self {
            sx: Sx::new(),
            part: PhantomData,
        }
    }

    /// Styles `part`; a second call for the same part merges into the first.
    pub fn part(mut self, part: P, nested: Sx) -> Self {
        self.sx = self.sx.selector(part.selector(), nested);
        self
    }
}

/// The `parts` prop as an `sx` source; a `static` keeps its address identity.
pub(crate) fn parts_source<P: Part>(parts: &Input<Parts<P>>) -> Option<SxSource<'_>> {
    match parts {
        Input::None => None,
        Input::Value(parts) => Some(SxSource::Owned(&parts.sx)),
        Input::Static(parts) => Some(SxSource::Static(&parts.sx)),
    }
}

impl<P: Part> Default for Parts<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Part> Clone for Parts<P> {
    fn clone(&self) -> Self {
        Self {
            sx: self.sx.clone(),
            part: PhantomData,
        }
    }
}

impl<P: Part> PartialEq for Parts<P> {
    fn eq(&self, other: &Self) -> bool {
        self.sx == other.sx
    }
}

impl<P: Part> fmt::Debug for Parts<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Parts").field(&self.sx).finish()
    }
}

impl<P: Part, const N: usize> From<[(P, Sx); N]> for Parts<P> {
    fn from(parts: [(P, Sx); N]) -> Self {
        parts
            .into_iter()
            .fold(Self::new(), |parts, (part, nested)| {
                parts.part(part, nested)
            })
    }
}

/// [`Parts`] built once, on first use, for a `static`. Equal only to itself.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Alert, AlertPart, Parts, StaticParts};
/// # use libero::sx::sx;
/// static QUIET: StaticParts<AlertPart> =
///     StaticParts::new(|| Parts::new().part(AlertPart::Message, sx().color("gray.7")));
/// # fn app() -> Element { rsx! {
/// Alert { title: "Saved", parts: &QUIET }
/// # } }
/// ```
pub struct StaticParts<P: Part>(LazyLock<Parts<P>>);

impl<P: Part> StaticParts<P> {
    pub const fn new(init: fn() -> Parts<P>) -> Self {
        Self(LazyLock::new(init))
    }
}

impl<P: Part> Deref for StaticParts<P> {
    type Target = Parts<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: Part> PartialEq for StaticParts<P> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<P: Part> fmt::Debug for StaticParts<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StaticParts").finish()
    }
}

impl<P: Part> From<Parts<P>> for Input<Parts<P>> {
    fn from(parts: Parts<P>) -> Self {
        Self::Value(parts)
    }
}

impl<P: Part> From<Option<Parts<P>>> for Input<Parts<P>> {
    fn from(parts: Option<Parts<P>>) -> Self {
        parts.map_or(Self::None, Self::Value)
    }
}

impl<P: Part, const N: usize> From<[(P, Sx); N]> for Input<Parts<P>> {
    fn from(parts: [(P, Sx); N]) -> Self {
        Self::Value(parts.into())
    }
}

impl<P: Part> From<&'static StaticParts<P>> for Input<Parts<P>> {
    fn from(parts: &'static StaticParts<P>) -> Self {
        Self::Static(parts)
    }
}

/// Declares a component's part enum: `Variant = "slot" => "selector"`.
macro_rules! parts_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $slot:literal => $selector:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis enum $name {
            $($(#[$variant_meta])* $variant),+
        }

        impl $crate::components::Part for $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            fn slot(self) -> &'static str {
                match self {
                    $(Self::$variant => $slot),+
                }
            }

            fn selector(self) -> &'static str {
                match self {
                    $(Self::$variant => $selector),+
                }
            }
        }
    };
}

pub(crate) use parts_enum;

/// `(slot, selector)` per part, for the slot-table snapshot tests.
#[cfg(test)]
pub(crate) fn part_table<P: Part>() -> Vec<(&'static str, &'static str)> {
    P::ALL
        .iter()
        .map(|part| (part.slot(), part.selector()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{css::Stylesheet, sx::sx};

    parts_enum! {
        enum TestPart {
            Label = "label" => "& > [data-slot='label']",
            Hint = "hint" => "& [data-slot='hint']",
        }
    }

    fn css(sx: &Sx) -> String {
        Stylesheet::from(sx).as_str().to_string()
    }

    #[test]
    fn a_part_compiles_to_its_selector() {
        let parts = Parts::new().part(TestPart::Label, sx().color("red"));

        assert_eq!(
            parts.sx,
            sx().selector("& > [data-slot='label']", sx().color("red"))
        );
        assert!(css(&parts.sx).contains("> [data-slot='label']"));
    }

    #[test]
    fn the_array_form_matches_the_chain() {
        let chained = Parts::new()
            .part(TestPart::Label, sx().color("red"))
            .part(TestPart::Hint, sx().padding("xs"));
        let array = Parts::from([
            (TestPart::Label, sx().color("red")),
            (TestPart::Hint, sx().padding("xs")),
        ]);

        assert_eq!(chained, array);
    }

    static QUIET: StaticParts<TestPart> =
        StaticParts::new(|| Parts::new().part(TestPart::Hint, sx().color("gray")));

    #[test]
    fn a_static_is_equal_only_to_itself_by_address() {
        let input: Input<Parts<TestPart>> = (&QUIET).into();

        assert!(matches!(input, Input::Static(parts) if std::ptr::eq(parts, &*QUIET)));
    }
}
