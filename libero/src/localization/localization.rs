use super::*;

/// Every string libero shows a reader, one group per component plus [`CommonLabels`].
/// [`ENGLISH`](Self::ENGLISH) by default, [`GERMAN`](Self::GERMAN) ships too; dates are [`Formats`].
///
/// Handed to `LiberoProvider { localization }`; a per-instance prop (`aria_label`, ...)
/// wins over it. Override with struct update, one group or one string at a time:
///
/// ```
/// use libero::localization::{CommonLabels, Localization, PaginationLabels};
///
/// static WORDS: Localization = Localization {
///     common: CommonLabels { close: "Zumachen", ..CommonLabels::GERMAN },
///     pagination: PaginationLabels { page: "Gehe zu Seite {n}", ..PaginationLabels::GERMAN },
///     ..Localization::GERMAN
/// };
/// assert_eq!(WORDS.common.close, "Zumachen");
/// ```
///
/// Templates take named holes (`{n}`, `{names}`), so a language can reorder them.
/// A catalogue loaded at runtime is leaked once per language (`Box::leak`).
///
/// Docs: <https://libero-ui.dev/about/localization>
// Not `Copy`, the `Theme` reason: a stray by-value use is a silent memcpy.
#[derive(Clone, Debug, PartialEq)]
pub struct Localization {
    /// The language as a BCP 47 tag (`"en"`, `"de"`); the outermost provider writes it
    /// to the root's `lang`, so a screen reader speaks the words in it.
    pub lang: &'static str,
    pub common: CommonLabels,
    pub form: FormLabels,
    /// Names and labels for every date and time component.
    pub date: DateLocale,
    pub pagination: PaginationLabels,
    pub avatar: AvatarLabels,
    pub burger: BurgerLabels,
    pub anchor: AnchorLabels,
    pub pin_field: PinFieldLabels,
    pub theme_switcher: ThemeSwitcherLabels,
    pub repository: RepositoryLabels,
    pub tldr: TldrLabels,
    pub direction_toggle: DirectionToggleLabels,
    pub spotlight: SpotlightLabels,
    pub carousel: CarouselLabels,
    pub nav_link: NavLinkLabels,
    pub lightbox: LightboxLabels,
    pub floating_window: FloatingWindowLabels,
    pub notifications: NotificationsLabels,
    pub scroller: ScrollerLabels,
    pub sortable: SortableLabels,
    pub kanban: KanbanLabels,
    pub stepper: StepperLabels,
    pub marquee: MarqueeLabels,
    pub media: MediaLabels,
    pub chips: ChipsLabels,
    pub combobox: ComboboxLabels,
    pub cascader: CascaderLabels,
    pub tags_field: TagsFieldLabels,
    pub image: ImageLabels,
    pub code_block: CodeBlockLabels,
    pub copy: CopyLabels,
    pub color: ColorLabels,
    pub phone_field: PhoneFieldLabels,
    pub password_field: PasswordFieldLabels,
    pub number_field: NumberFieldLabels,
    pub file_field: FileFieldLabels,
    pub image_cropper: ImageCropperLabels,
    pub textarea: TextareaLabels,
    pub slider: SliderLabels,
    pub rating: RatingLabels,
    pub menu: MenuLabels,
    pub table: TableLabels,
    pub shortcut_help: ShortcutHelpLabels,
    pub rich_text_editor: RichTextEditorLabels,
    pub tour: TourLabels,
}

impl Localization {
    /// The default.
    pub const ENGLISH: Localization = Localization {
        lang: "en",
        common: CommonLabels::ENGLISH,
        form: FormLabels::ENGLISH,
        date: DateLocale::ENGLISH,
        pagination: PaginationLabels::ENGLISH,
        avatar: AvatarLabels::ENGLISH,
        burger: BurgerLabels::ENGLISH,
        anchor: AnchorLabels::ENGLISH,
        pin_field: PinFieldLabels::ENGLISH,
        theme_switcher: ThemeSwitcherLabels::ENGLISH,
        repository: RepositoryLabels::ENGLISH,
        tldr: TldrLabels::ENGLISH,
        direction_toggle: DirectionToggleLabels::ENGLISH,
        spotlight: SpotlightLabels::ENGLISH,
        carousel: CarouselLabels::ENGLISH,
        nav_link: NavLinkLabels::ENGLISH,
        lightbox: LightboxLabels::ENGLISH,
        floating_window: FloatingWindowLabels::ENGLISH,
        notifications: NotificationsLabels::ENGLISH,
        scroller: ScrollerLabels::ENGLISH,
        sortable: SortableLabels::ENGLISH,
        kanban: KanbanLabels::ENGLISH,
        stepper: StepperLabels::ENGLISH,
        marquee: MarqueeLabels::ENGLISH,
        media: MediaLabels::ENGLISH,
        chips: ChipsLabels::ENGLISH,
        combobox: ComboboxLabels::ENGLISH,
        cascader: CascaderLabels::ENGLISH,
        tags_field: TagsFieldLabels::ENGLISH,
        image: ImageLabels::ENGLISH,
        code_block: CodeBlockLabels::ENGLISH,
        copy: CopyLabels::ENGLISH,
        color: ColorLabels::ENGLISH,
        phone_field: PhoneFieldLabels::ENGLISH,
        password_field: PasswordFieldLabels::ENGLISH,
        number_field: NumberFieldLabels::ENGLISH,
        file_field: FileFieldLabels::ENGLISH,
        image_cropper: ImageCropperLabels::ENGLISH,
        textarea: TextareaLabels::ENGLISH,
        slider: SliderLabels::ENGLISH,
        rating: RatingLabels::ENGLISH,
        menu: MenuLabels::ENGLISH,
        table: TableLabels::ENGLISH,
        shortcut_help: ShortcutHelpLabels::ENGLISH,
        rich_text_editor: RichTextEditorLabels::ENGLISH,
        tour: TourLabels::ENGLISH,
    };

    /// Hand it to `LiberoProvider { localization }`; German dates are `Formats::GERMAN`.
    pub const GERMAN: Localization = Localization {
        lang: "de",
        common: CommonLabels::GERMAN,
        form: FormLabels::GERMAN,
        date: DateLocale::GERMAN,
        pagination: PaginationLabels::GERMAN,
        avatar: AvatarLabels::GERMAN,
        burger: BurgerLabels::GERMAN,
        anchor: AnchorLabels::GERMAN,
        pin_field: PinFieldLabels::GERMAN,
        theme_switcher: ThemeSwitcherLabels::GERMAN,
        repository: RepositoryLabels::GERMAN,
        tldr: TldrLabels::GERMAN,
        direction_toggle: DirectionToggleLabels::GERMAN,
        spotlight: SpotlightLabels::GERMAN,
        carousel: CarouselLabels::GERMAN,
        nav_link: NavLinkLabels::GERMAN,
        lightbox: LightboxLabels::GERMAN,
        floating_window: FloatingWindowLabels::GERMAN,
        notifications: NotificationsLabels::GERMAN,
        scroller: ScrollerLabels::GERMAN,
        sortable: SortableLabels::GERMAN,
        kanban: KanbanLabels::GERMAN,
        stepper: StepperLabels::GERMAN,
        marquee: MarqueeLabels::GERMAN,
        media: MediaLabels::GERMAN,
        chips: ChipsLabels::GERMAN,
        combobox: ComboboxLabels::GERMAN,
        cascader: CascaderLabels::GERMAN,
        tags_field: TagsFieldLabels::GERMAN,
        image: ImageLabels::GERMAN,
        code_block: CodeBlockLabels::GERMAN,
        copy: CopyLabels::GERMAN,
        color: ColorLabels::GERMAN,
        phone_field: PhoneFieldLabels::GERMAN,
        password_field: PasswordFieldLabels::GERMAN,
        number_field: NumberFieldLabels::GERMAN,
        file_field: FileFieldLabels::GERMAN,
        image_cropper: ImageCropperLabels::GERMAN,
        textarea: TextareaLabels::GERMAN,
        slider: SliderLabels::GERMAN,
        rating: RatingLabels::GERMAN,
        menu: MenuLabels::GERMAN,
        table: TableLabels::GERMAN,
        shortcut_help: ShortcutHelpLabels::GERMAN,
        rich_text_editor: RichTextEditorLabels::GERMAN,
        tour: TourLabels::GERMAN,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every template with a hole, in one order for every language.
    fn templates(words: &Localization) -> [&'static str; 44] {
        [
            words.media.position,
            words.tour.progress,
            words.common.remove,
            words.image_cropper.value,
            words.rating.value,
            words.cascader.back,
            words.cascader.select,
            words.chips.added,
            words.chips.removed,
            words.chips.added_and_removed,
            words.tags_field.duplicate,
            words.tags_field.full,
            words.tags_field.not_allowed,
            words.image.zoom_named,
            words.color.saturation_value,
            words.color.hue_value,
            words.color.alpha_value,
            words.phone_field.country,
            words.pagination.page,
            words.pagination.current_page,
            words.avatar.count,
            words.avatar.more,
            words.carousel.indicator,
            words.carousel.slide,
            words.carousel.status,
            words.carousel.status_range,
            words.carousel.status_wrap,
            words.lightbox.thumbnail,
            words.lightbox.zoomed,
            words.notifications.region,
            words.pin_field.cell,
            words.sortable.item,
            words.sortable.lifted,
            words.sortable.moved,
            words.sortable.dropped,
            words.sortable.cancelled,
            words.kanban.moved,
            words.code_block.code_named,
            words.tldr.prompt,
            words.date.on_or_after,
            words.date.on_or_before,
            words.date.between,
            words.date.duration_at_least,
            words.date.duration_at_most,
        ]
    }

    /// A template's hole names, sorted.
    fn holes_of(template: &str) -> Vec<&str> {
        let mut holes: Vec<&str> = template
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect();
        holes.sort_unstable();
        holes
    }

    /// Every hole a template has, filled: nothing is left behind.
    #[test]
    fn every_template_fills_completely() {
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
            ("count", &5),
            ("name", &"Germany"),
            ("column", &"Done"),
            ("iso", &"DE"),
            ("dial", &49),
            ("key", &"F8"),
            ("language", &"Rust"),
            ("min", &"March 5, 2026"),
            ("max", &"March 9, 2026"),
            ("from", &1),
            ("to", &3),
            ("url", &"https://libero-ui.dev"),
            ("width", &40),
            ("height", &30),
            ("x", &10),
            ("y", &20),
            ("time", &"1:05"),
            ("duration", &"4:56"),
        ];
        for words in [&Localization::ENGLISH, &Localization::GERMAN] {
            for template in templates(words) {
                let filled = fill(template, holes);
                assert!(!filled.contains('{'), "{template} -> {filled}");
            }
        }
    }

    /// German has every hole English has, and no other: no value goes unsaid.
    #[test]
    fn german_fills_the_same_holes_as_english() {
        let english = templates(&Localization::ENGLISH);
        let german = templates(&Localization::GERMAN);
        for (english, german) in english.into_iter().zip(german) {
            assert_eq!(holes_of(german), holes_of(english), "{english} / {german}");
        }
    }
}
