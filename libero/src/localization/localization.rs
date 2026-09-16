use super::*;

/// Every string libero puts in front of a reader, one group per component
/// plus [`CommonLabels`]. English by default; no other language ships.
///
/// Handed to `LiberoProvider { localization }` and read with
/// [`use_localization`](crate::hooks::use_localization). A per-instance prop
/// (`aria_label`, `close_label`, ...) wins over it. Override with struct
/// update, one group or one string at a time:
///
/// ```
/// use libero::localization::{CommonLabels, Localization, PaginationLabels};
///
/// static GERMAN: Localization = Localization {
///     common: CommonLabels { close: "Schließen", ..CommonLabels::ENGLISH },
///     pagination: PaginationLabels { page: "Gehe zu Seite {n}", ..PaginationLabels::ENGLISH },
///     ..Localization::ENGLISH
/// };
/// assert_eq!(GERMAN.common.close, "Schließen");
/// ```
///
/// Templates take named holes (`{n}`, `{m}`, `{names}`), so a language can
/// reorder them. A catalogue loaded at runtime is leaked once per language
/// (`Box::leak`), which is bounded.
// Not `Copy`, the `Theme` reason: a stray by-value use is a silent memcpy.
#[derive(Clone, Debug, PartialEq)]
pub struct Localization {
    pub common: CommonLabels,
    /// Names, first weekday and formats for every date and time component.
    pub date: DateLocale,
    pub pagination: PaginationLabels,
    pub avatar: AvatarLabels,
    pub burger: BurgerLabels,
    pub anchor: AnchorLabels,
    pub pin_field: PinFieldLabels,
    pub color_scheme_button: ColorSchemeButtonLabels,
    pub spotlight: SpotlightLabels,
    pub carousel: CarouselLabels,
    pub nav_link: NavLinkLabels,
    pub lightbox: LightboxLabels,
    pub floating_window: FloatingWindowLabels,
    pub notifications: NotificationsLabels,
    pub scroller: ScrollerLabels,
    pub stepper: StepperLabels,
    pub marquee: MarqueeLabels,
    pub chips: ChipsLabels,
    pub image: ImageLabels,
    pub code_block: CodeBlockLabels,
    pub color: ColorLabels,
    pub phone_field: PhoneFieldLabels,
    pub password_field: PasswordFieldLabels,
    pub number_field: NumberFieldLabels,
    pub file_field: FileFieldLabels,
    pub textarea: TextareaLabels,
}

impl Localization {
    pub const ENGLISH: Localization = Localization {
        common: CommonLabels::ENGLISH,
        date: DateLocale::ENGLISH,
        pagination: PaginationLabels::ENGLISH,
        avatar: AvatarLabels::ENGLISH,
        burger: BurgerLabels::ENGLISH,
        anchor: AnchorLabels::ENGLISH,
        pin_field: PinFieldLabels::ENGLISH,
        color_scheme_button: ColorSchemeButtonLabels::ENGLISH,
        spotlight: SpotlightLabels::ENGLISH,
        carousel: CarouselLabels::ENGLISH,
        nav_link: NavLinkLabels::ENGLISH,
        lightbox: LightboxLabels::ENGLISH,
        floating_window: FloatingWindowLabels::ENGLISH,
        notifications: NotificationsLabels::ENGLISH,
        scroller: ScrollerLabels::ENGLISH,
        stepper: StepperLabels::ENGLISH,
        marquee: MarqueeLabels::ENGLISH,
        chips: ChipsLabels::ENGLISH,
        image: ImageLabels::ENGLISH,
        code_block: CodeBlockLabels::ENGLISH,
        color: ColorLabels::ENGLISH,
        phone_field: PhoneFieldLabels::ENGLISH,
        password_field: PasswordFieldLabels::ENGLISH,
        number_field: NumberFieldLabels::ENGLISH,
        file_field: FileFieldLabels::ENGLISH,
        textarea: TextareaLabels::ENGLISH,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every hole an English template has, filled: nothing is left behind.
    #[test]
    fn every_english_template_fills_completely() {
        let english = &Localization::ENGLISH;
        let holes: &[(&str, &dyn std::fmt::Display)] = &[
            ("n", &2),
            ("m", &5),
            ("names", &"Ada"),
            ("label", &"Ada"),
            ("labels", &"Ada"),
            ("added", &"Ada"),
            ("removed", &"Grace"),
            ("alt", &"A cat"),
            ("s", &40),
            ("v", &60),
            ("value", &90),
            ("name", &"Germany"),
            ("iso", &"DE"),
            ("dial", &49),
            ("key", &"F8"),
            ("language", &"Rust"),
            ("min", &"March 5, 2026"),
            ("max", &"March 9, 2026"),
        ];
        for template in [
            english.common.remove,
            english.chips.added,
            english.chips.removed,
            english.chips.added_and_removed,
            english.image.zoom_named,
            english.color.saturation_value,
            english.color.hue_value,
            english.color.alpha_value,
            english.phone_field.country,
            english.pagination.page,
            english.pagination.current_page,
            english.avatar.count,
            english.avatar.more,
            english.carousel.indicator,
            english.carousel.slide,
            english.carousel.status,
            english.lightbox.thumbnail,
            english.notifications.region,
            english.pin_field.cell,
            english.code_block.code_named,
            english.date.on_or_after,
            english.date.on_or_before,
            english.date.between,
        ] {
            let filled = fill(template, holes);
            assert!(!filled.contains('{'), "{template} -> {filled}");
        }
    }
}
