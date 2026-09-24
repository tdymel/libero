use dioxus::prelude::*;
use pictogram_core::Svg as SvgData;

mod sets;

/// A glyph libero draws itself, by what it means. An [`IconProvider`] can swap
/// each one. Brand marks (GitHub, Google, ...) are not slots: they name a service.
///
/// ```rust
/// # use libero::IconSlot;
/// let slot = IconSlot::Close;
/// # let _ = slot;
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconSlot {
    /// Dialog, alert, chip and clear buttons; a failed step.
    Close,
    /// Select, Cascader, Accordion and Menu triggers; the caller rotates it.
    ChevronDown,
    /// A vertical carousel's previous slide; a window's move up.
    ChevronUp,
    /// Previous month, page or slide. Mirrored right to left by CSS.
    ChevronLeft,
    /// Next month, page or slide; a submenu. Mirrored right to left by CSS, except in a menu.
    ChevronRight,
    /// A pager's first page.
    ChevronFirst,
    /// A pager's last page.
    ChevronLast,
    /// A sortable table column; flipped for ascending.
    ArrowDown,
    /// A done step, a checked menu item, a picked swatch, a copied code block.
    Check,
    /// A checked checkbox, drawn at stroke width 3.
    CheckboxCheck,
    /// An indeterminate checkbox, drawn at stroke width 3.
    CheckboxIndeterminate,
    /// A number field's step up, a lightbox's zoom in.
    Plus,
    /// A number field's step down, a lightbox's zoom out.
    Minus,
    /// A password field's reveal button, secret hidden.
    Eye,
    /// A password field's reveal button, secret shown.
    EyeOff,
    /// A file field's dropzone.
    Upload,
    /// A colour field's eyedropper.
    EyeDropper,
    /// A copy button.
    Copy,
    /// A copy button after a denied write.
    CopyFailed,
    /// An anchor that opens a new tab.
    ExternalLink,
    /// An avatar with neither image nor initials.
    Person,
    /// The theme toggle, switching to light.
    Sun,
    /// The theme toggle, switching to dark.
    Moon,
    /// The theme toggle, handing the choice back to the platform.
    SystemScheme,
    /// A carousel's or marquee's play button.
    Play,
    /// A carousel's or marquee's pause button.
    Pause,
    /// The direction toggle, turning the text left to right.
    TextDirectionLtr,
    /// The direction toggle, turning the text right to left.
    TextDirectionRtl,
    /// The TL;DR trigger.
    Sparkles,
    /// A rating's symbol, filled with `currentColor` for the value: a stroked glyph
    /// turns solid, a solid one only changes colour.
    Star,
    /// A sortable item's drag handle.
    Grip,
}

const SLOTS: usize = IconSlot::Grip as usize + 1;

/// Glyphs by [`IconSlot`]; an empty slot keeps libero's default (lucide).
/// A whole set starts from its constructor, one `icons-<set>` feature each:
/// `IconSet::material_rounded()`, `tabler_outlined()`, `bootstrap_outlined()`, `phosphor_regular()`, ...
///
/// ```rust
/// # use libero::{IconSet, IconSlot};
/// const ICONS: IconSet = IconSet::new()
///     .with(IconSlot::Close, pictogram_icons_lucide::circle_x::outlined)
///     .with(IconSlot::ChevronDown, pictogram_icons_lucide::chevrons_down::outlined);
/// assert!(ICONS.get(IconSlot::Close).is_some());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IconSet([Option<SvgData>; SLOTS]);

impl IconSet {
    /// No slot set.
    pub const fn new() -> Self {
        Self([None; SLOTS])
    }

    /// `slot` drawn as `icon`.
    pub const fn with(mut self, slot: IconSlot, icon: SvgData) -> Self {
        self.0[slot as usize] = Some(icon);
        self
    }

    /// The glyph set for `slot`, if any.
    pub const fn get(&self, slot: IconSlot) -> Option<SvgData> {
        self.0[slot as usize]
    }

    /// `self` with `outer`'s slots filled in where `self` has none.
    fn over(mut self, outer: &IconSet) -> Self {
        for (own, outer) in self.0.iter_mut().zip(outer.0) {
            *own = own.or(outer);
        }
        self
    }
}

/// The glyphs in effect below the nearest [`IconProvider`].
#[derive(Clone, Copy)]
pub(crate) struct IconContext(pub(crate) Memo<IconSet>);

/// Swaps libero's own glyphs below it, slot by slot. Nested providers merge: the
/// inner one wins per slot, the rest come from the outer one, then the defaults.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{IconProvider, IconSet, IconSlot};
/// # fn app() -> Element {
/// rsx! {
///     IconProvider {
///         icons: IconSet::new().with(IconSlot::Close, pictogram_icons_lucide::circle_x::outlined),
///         "Every dialog, chip and clear button below draws a circled x."
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/icon-provider>
#[component]
pub fn IconProvider(
    /// The slots to swap; reactive.
    icons: ReadSignal<IconSet>,
    children: Element,
) -> Element {
    let outer = try_use_context::<IconContext>();
    let merged = use_memo(move || match outer {
        Some(outer) => icons().over(&outer.0.read()),
        None => icons(),
    });
    use_context_provider(|| IconContext(merged));
    rsx! {
        {children}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: SvgData = SvgData::new(r#"<svg viewBox="0 0 1 1"><path d="A"/></svg>"#);
    const B: SvgData = SvgData::new(r#"<svg viewBox="0 0 1 1"><path d="B"/></svg>"#);

    #[test]
    fn the_inner_set_wins_per_slot_and_the_outer_fills_the_rest() {
        let outer = IconSet::new()
            .with(IconSlot::Close, A)
            .with(IconSlot::Check, A);
        let inner = IconSet::new().with(IconSlot::Close, B);
        let merged = inner.over(&outer);
        assert_eq!(merged.get(IconSlot::Close), Some(B));
        assert_eq!(merged.get(IconSlot::Check), Some(A));
        assert_eq!(merged.get(IconSlot::Plus), None);
    }
}
