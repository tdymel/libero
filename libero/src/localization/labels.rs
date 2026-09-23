//! One group per component, plus `CommonLabels` for the words many share.
//! Templates take named holes, filled by [`fill`](super::fill).

/// Words many components share. A component with its own wording has a group
/// of its own instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommonLabels {
    /// Every close button: `Alert`, `Notifications`, `Lightbox`,
    /// `FloatingWindow`, `Dialog`.
    pub close: &'static str,
    /// A clearable field's x.
    pub clear: &'static str,
    /// An x that drops one item - a chip, a file. `{label}` names the item.
    pub remove: &'static str,
    /// What a loader announces while options are fetched.
    pub loading: &'static str,
    /// A search box's placeholder.
    pub search: &'static str,
}

impl CommonLabels {
    pub const ENGLISH: Self = Self {
        close: "Close",
        clear: "Clear",
        remove: "Remove {label}",
        loading: "Loading",
        search: "Search",
    };

    pub const GERMAN: Self = Self {
        close: "Schließen",
        clear: "Leeren",
        remove: "{label} entfernen",
        loading: "Wird geladen",
        search: "Suchen",
    };
}

/// Every list with a search or typed filter: `Select`, `MultiSelect`, `Cascader`,
/// `Autocomplete`, `TagsField`. Its loader reads [`CommonLabels::loading`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboboxLabels {
    /// Shown, and announced, when typed text matches no option.
    pub nothing_found: &'static str,
}

impl ComboboxLabels {
    pub const ENGLISH: Self = Self {
        nothing_found: "No results",
    };

    pub const GERMAN: Self = Self {
        nothing_found: "Keine Ergebnisse",
    };
}

/// A `Cascader` on a narrow screen, which shows one level at a time.
/// `{label}` is the parent option's label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CascaderLabels {
    /// The header button that returns to the parent's level.
    pub back: &'static str,
    /// Under `any_level`, the first row, which picks the parent itself.
    pub select: &'static str,
}

impl CascaderLabels {
    pub const ENGLISH: Self = Self {
        back: "Back to {label}",
        select: "Select {label}",
    };

    pub const GERMAN: Self = Self {
        back: "Zurück zu {label}",
        select: "{label} auswählen",
    };
}

/// What a `TagsField` announces when it refuses a tag. `{labels}` are the
/// refused tags joined by `, `; the draft keeps the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagsFieldLabels {
    /// The tag is already in the list.
    pub duplicate: &'static str,
    /// The list holds `max_tags` already.
    pub full: &'static str,
    /// `tag_rules` turned it down.
    pub not_allowed: &'static str,
}

impl TagsFieldLabels {
    pub const ENGLISH: Self = Self {
        duplicate: "Already added: {labels}",
        full: "Tag limit reached, not added: {labels}",
        not_allowed: "Not allowed: {labels}",
    };

    pub const GERMAN: Self = Self {
        duplicate: "Bereits hinzugefügt: {labels}",
        full: "Höchstzahl erreicht, nicht hinzugefügt: {labels}",
        not_allowed: "Nicht erlaubt: {labels}",
    };
}

/// What a field holding chips announces when its list changes. `{labels}`,
/// `{added}` and `{removed}` are labels joined by `, `.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipsLabels {
    pub added: &'static str,
    pub removed: &'static str,
    /// One change that both added and removed.
    pub added_and_removed: &'static str,
}

impl ChipsLabels {
    pub const ENGLISH: Self = Self {
        added: "Added {labels}",
        removed: "Removed {labels}",
        added_and_removed: "Added {added}. Removed {removed}",
    };

    pub const GERMAN: Self = Self {
        added: "{labels} hinzugefügt",
        removed: "{labels} entfernt",
        added_and_removed: "{added} hinzugefügt. {removed} entfernt",
    };
}

/// A zoomable `Image`'s button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageLabels {
    /// Names the button of a picture without `alt`.
    pub zoom: &'static str,
    /// Names it after the picture: `{alt}`.
    pub zoom_named: &'static str,
}

impl ImageLabels {
    pub const ENGLISH: Self = Self {
        zoom: "Zoom in",
        zoom_named: "Zoom in: {alt}",
    };

    pub const GERMAN: Self = Self {
        zoom: "Vergrößern",
        zoom_named: "Vergrößern: {alt}",
    };
}

/// What a `CopyButton` is named and announces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyButtonLabels {
    /// The button's name, unless its `aria_label` replaces it.
    pub copy: &'static str,
    /// Announced once the copy landed.
    pub copied: &'static str,
    pub copy_failed: &'static str,
}

impl CopyButtonLabels {
    pub const ENGLISH: Self = Self {
        copy: "Copy",
        copied: "Copied",
        copy_failed: "Copy failed",
    };

    pub const GERMAN: Self = Self {
        copy: "Kopieren",
        copied: "Kopiert",
        copy_failed: "Kopieren fehlgeschlagen",
    };
}

/// A `CodeBlock`'s copy button and header. The copy announcements are
/// [`CopyButtonLabels`]'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeBlockLabels {
    /// The copy button's name.
    pub copy: &'static str,
    /// Read before a `diff` line that starts with `+`; the `+` is drawing only.
    pub added: &'static str,
    /// Read before a `diff` line that starts with `-`.
    pub removed: &'static str,
    /// The scroll region's name when the code overflows and has no known language.
    pub code: &'static str,
    /// The scroll region's name with a known language; `{language}` is its name.
    pub code_named: &'static str,
    /// The header's language name for `language: "text"`, no highlighting.
    pub plain_text: &'static str,
}

impl CodeBlockLabels {
    pub const ENGLISH: Self = Self {
        copy: "Copy code",
        added: "Added",
        removed: "Removed",
        code: "Code",
        code_named: "{language} code",
        plain_text: "Plain text",
    };

    pub const GERMAN: Self = Self {
        copy: "Code kopieren",
        added: "Hinzugefügt",
        removed: "Entfernt",
        code: "Code",
        code_named: "{language}-Code",
        plain_text: "Nur-Text",
    };
}

/// `ColorPicker`, its sliders and `ColorField`. The channel names stand in when
/// the matching `*_label` prop is unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorLabels {
    pub saturation: &'static str,
    pub hue: &'static str,
    pub alpha: &'static str,
    /// The panel's value: `{s}` saturation and `{v}` brightness, in percent.
    pub saturation_value: &'static str,
    /// The hue slider's value: `{value}` in degrees.
    pub hue_value: &'static str,
    /// The alpha slider's value: `{value}` in percent.
    pub alpha_value: &'static str,
    /// Names `ColorField`'s dropdown.
    pub choose: &'static str,
    /// Names `ColorField`'s eye dropper button.
    pub eye_dropper: &'static str,
    /// `ColorField`'s error while its typed text is no color.
    pub invalid: &'static str,
}

impl ColorLabels {
    pub const ENGLISH: Self = Self {
        saturation: "Saturation",
        hue: "Hue",
        alpha: "Alpha",
        saturation_value: "Saturation {s}%, brightness {v}%",
        hue_value: "{value} degrees",
        alpha_value: "{value}%",
        choose: "Choose color",
        eye_dropper: "Pick a color from the screen",
        invalid: "Not a valid color",
    };

    pub const GERMAN: Self = Self {
        saturation: "Sättigung",
        hue: "Farbton",
        alpha: "Deckkraft",
        saturation_value: "Sättigung {s} %, Helligkeit {v} %",
        hue_value: "{value} Grad",
        alpha_value: "{value} %",
        choose: "Farbe wählen",
        eye_dropper: "Farbe vom Bildschirm aufnehmen",
        invalid: "Keine gültige Farbe",
    };
}

/// `PhoneField`'s country picker. Its search placeholder is
/// [`CommonLabels::search`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneFieldLabels {
    /// Names the country search box.
    pub search: &'static str,
    /// Names the country button: `{name}`, `{iso}` and `{dial}`.
    pub country: &'static str,
    /// Country names by upper-case ISO code; a code missing here keeps its
    /// English name. `country_label` on the field wins over both.
    ///
    /// ```
    /// use libero::localization::PhoneFieldLabels;
    ///
    /// static LABELS: PhoneFieldLabels = PhoneFieldLabels {
    ///     country_names: &[("DE", "Deutschland"), ("AT", "Österreich")],
    ///     ..PhoneFieldLabels::GERMAN
    /// };
    /// ```
    pub country_names: &'static [(&'static str, &'static str)],
}

impl PhoneFieldLabels {
    pub const ENGLISH: Self = Self {
        search: "Search countries",
        country: "Country: {name}, {iso} +{dial}",
        country_names: &[],
    };

    pub const GERMAN: Self = Self {
        search: "Länder durchsuchen",
        country: "Land: {name}, {iso} +{dial}",
        country_names: super::country_names::GERMAN,
    };

    /// The name `country_names` gives `iso`, if any.
    pub(crate) fn country_name(&self, iso: &str) -> Option<&'static str> {
        self.country_names
            .iter()
            .find(|(code, _)| code.eq_ignore_ascii_case(iso))
            .map(|(_, name)| *name)
    }
}

/// `PasswordField`'s reveal button, when its props are unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswordFieldLabels {
    /// Names the button in both states; `aria-pressed` carries the state.
    pub show: &'static str,
}

impl PasswordFieldLabels {
    pub const ENGLISH: Self = Self {
        show: "Show password",
    };

    pub const GERMAN: Self = Self {
        show: "Passwort anzeigen",
    };
}

/// `NumberField`'s steppers, when its props are unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberFieldLabels {
    pub decrease: &'static str,
    pub increase: &'static str,
}

impl NumberFieldLabels {
    pub const ENGLISH: Self = Self {
        decrease: "Decrease",
        increase: "Increase",
    };

    pub const GERMAN: Self = Self {
        decrease: "Verringern",
        increase: "Erhöhen",
    };
}

/// `RangeSlider`'s thumbs, when its props are unset. Each follows the field's
/// label: "Price Minimum".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliderLabels {
    pub minimum: &'static str,
    pub maximum: &'static str,
}

impl SliderLabels {
    pub const ENGLISH: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
    };

    pub const GERMAN: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
    };
}

/// A `Menu` item's drawn shortcut hint. The spoken one is `aria-keyshortcuts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuLabels {
    /// Stands in for `Control`: "Control+S" draws "Ctrl+S".
    pub control: &'static str,
    /// Stands in for `Shift`: "Control+Shift+S" draws "Strg+Umschalt+S".
    pub shift: &'static str,
    pub alt: &'static str,
    pub meta: &'static str,
}

impl MenuLabels {
    pub const ENGLISH: Self = Self {
        control: "Ctrl",
        shift: "Shift",
        alt: "Alt",
        meta: "Meta",
    };
    pub const GERMAN: Self = Self {
        control: "Strg",
        shift: "Umschalt",
        alt: "Alt",
        meta: "Meta",
    };
}

/// Every string `Pagination` puts in front of a reader.
///
/// The `<nav>`'s own name is **not** here. `aria_label` is a required prop, so
/// the caller supplies it and localises it themselves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationLabels {
    /// A page the reader is not on. `{n}` is the page number.
    pub page: &'static str,
    /// The current page. `aria-current` already says "current", so repeating
    /// "go to" on the page you are on is a lie.
    pub current_page: &'static str,
    pub previous: &'static str,
    pub next: &'static str,
    pub first: &'static str,
    pub last: &'static str,
}

impl PaginationLabels {
    pub const ENGLISH: Self = Self {
        page: "Go to page {n}",
        current_page: "Page {n}",
        previous: "Go to previous page",
        next: "Go to next page",
        first: "Go to first page",
        last: "Go to last page",
    };

    pub const GERMAN: Self = Self {
        page: "Zu Seite {n}",
        current_page: "Seite {n}",
        previous: "Zur vorherigen Seite",
        next: "Zur nächsten Seite",
        first: "Zur ersten Seite",
        last: "Zur letzten Seite",
    };
}

/// `AvatarGroup`'s overflow chip. `{n}` is the hidden count, `{names}` their
/// names joined by `, `.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarLabels {
    /// The chip's visible text: `+3`.
    pub count: &'static str,
    /// The chip's accessible name: `3 more: Ada, Grace, Linus`.
    pub more: &'static str,
}

impl AvatarLabels {
    pub const ENGLISH: Self = Self {
        count: "+{n}",
        more: "{n} more: {names}",
    };

    pub const GERMAN: Self = Self {
        count: "+{n}",
        more: "{n} weitere: {names}",
    };
}

/// The words a burger announces itself with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurgerLabels {
    /// Names the button while `open` is unset: it opens something.
    pub open: &'static str,
    /// Names the button in both states once `open` is set: `aria-expanded`
    /// carries the state.
    pub toggle: &'static str,
}

impl BurgerLabels {
    pub const ENGLISH: Self = Self {
        open: "Open navigation",
        toggle: "Toggle navigation",
    };

    pub const GERMAN: Self = Self {
        open: "Navigation öffnen",
        toggle: "Navigation umschalten",
    };
}

/// An `Anchor`'s or link `Chip`'s cue for a link that opens a new tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorLabels {
    /// Read after the link text, hidden from sight: the icon shows it.
    pub new_tab: &'static str,
}

impl AnchorLabels {
    pub const ENGLISH: Self = Self {
        new_tab: "(opens in a new tab)",
    };

    pub const GERMAN: Self = Self {
        new_tab: "(öffnet in einem neuen Tab)",
    };
}

/// A `PinField`'s cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinFieldLabels {
    /// Names a cell: `{n}` is its one-based position, `{m}` the count.
    pub cell: &'static str,
}

impl PinFieldLabels {
    pub const ENGLISH: Self = Self {
        cell: "Character {n} of {m}",
    };

    pub const GERMAN: Self = Self {
        cell: "Zeichen {n} von {m}",
    };
}

/// What a `ThemeToggle` announces itself with. The three `to_*` name
/// what a press *does*, as the glyph beside them shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeToggleLabels {
    /// Names the button when a press pins the light scheme.
    pub to_light: &'static str,
    /// Names the button when a press pins the dark scheme.
    pub to_dark: &'static str,
    /// Names the button when a press hands the choice back to the platform.
    pub to_system: &'static str,
    /// Names the pair of buttons, when a theme picker makes it a pair.
    pub group: &'static str,
    /// Names the picker's own button.
    pub picker: &'static str,
    /// Names the group of theme sets inside the picker's menu.
    pub themes: &'static str,
}

impl ThemeToggleLabels {
    pub const ENGLISH: Self = Self {
        to_light: "Switch to the light theme",
        to_dark: "Switch to the dark theme",
        to_system: "Follow the system theme",
        group: "Theme",
        picker: "Choose a theme",
        themes: "Themes",
    };

    pub const GERMAN: Self = Self {
        to_light: "Zum hellen Design wechseln",
        to_dark: "Zum dunklen Design wechseln",
        to_system: "Dem Systemdesign folgen",
        group: "Design",
        picker: "Design wählen",
        themes: "Designs",
    };
}

/// A `RepoButton`'s name: the host and repo, plus the star count once one shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct RepoButtonLabels {
    /// The count as heard after the host and repo: the exact count, for the
    /// plural, and the shortened one on screen (`1.2k`). A fn, for plural forms.
    ///
    /// ```
    /// use libero::localization::RepoButtonLabels;
    ///
    /// assert_eq!((RepoButtonLabels::ENGLISH.stars)(1_234, "1.2k"), "1.2k stars");
    /// assert_eq!((RepoButtonLabels::GERMAN.stars)(1, "1"), "1 Stern");
    /// ```
    pub stars: fn(u64, &str) -> String,
}

/// `RepoButtonLabels::ENGLISH.stars`. A named fn, so every copy compares equal.
fn english_stars(count: u64, shown: &str) -> String {
    match count {
        1 => format!("{shown} star"),
        _ => format!("{shown} stars"),
    }
}

/// `RepoButtonLabels::GERMAN.stars`.
fn german_stars(count: u64, shown: &str) -> String {
    match count {
        1 => format!("{shown} Stern"),
        _ => format!("{shown} Sterne"),
    }
}

impl RepoButtonLabels {
    pub const ENGLISH: Self = Self {
        stars: english_stars,
    };

    pub const GERMAN: Self = Self {
        stars: german_stars,
    };
}

/// A `Tldr` menu's words. Provider names are brand names and stay literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TldrLabels {
    /// The trigger's visible text.
    pub label: &'static str,
    /// Names the icon-only trigger.
    pub icon_only: &'static str,
    /// Names the group of providers in the menu.
    pub group: &'static str,
    /// What the assistant is asked; `{url}` is the page's address.
    pub prompt: &'static str,
}

impl TldrLabels {
    pub const ENGLISH: Self = Self {
        label: "TLDR",
        icon_only: "Summarize with AI",
        group: "Summarize with",
        prompt: "Summarize and analyze the key insights from {url}. If you cannot access this URL \
                 please fall back to your general knowledge.",
    };

    pub const GERMAN: Self = Self {
        label: "TLDR",
        icon_only: "Mit KI zusammenfassen",
        group: "Zusammenfassen mit",
        prompt: "Fasse die wichtigsten Erkenntnisse aus {url} zusammen und analysiere sie. Wenn \
                 du diese URL nicht öffnen kannst, greife auf dein allgemeines Wissen zurück.",
    };
}

/// A `DirectionToggle`'s name, which says what a press does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectionToggleLabels {
    /// Names the button while the text runs left to right.
    pub to_rtl: &'static str,
    /// Names the button while the text runs right to left.
    pub to_ltr: &'static str,
}

impl DirectionToggleLabels {
    pub const ENGLISH: Self = Self {
        to_rtl: "Switch to right-to-left text",
        to_ltr: "Switch to left-to-right text",
    };

    pub const GERMAN: Self = Self {
        to_rtl: "Zu Text von rechts nach links wechseln",
        to_ltr: "Zu Text von links nach rechts wechseln",
    };
}

/// Every string a `Spotlight` says to a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpotlightLabels {
    /// Names the dialog, unless `SpotlightOptions::aria_label` does.
    pub label: &'static str,
    /// Names the search box.
    pub search: &'static str,
    pub placeholder: &'static str,
    /// Shown, and announced, when a query matches nothing.
    pub nothing_found: &'static str,
    /// Announced while `SpotlightOptions::loading` is set.
    pub loading: &'static str,
}

impl SpotlightLabels {
    pub const ENGLISH: Self = Self {
        label: "Command palette",
        search: "Search commands",
        placeholder: "Search...",
        nothing_found: "Nothing found",
        loading: "Searching",
    };

    pub const GERMAN: Self = Self {
        label: "Befehlspalette",
        search: "Befehle durchsuchen",
        placeholder: "Suchen …",
        nothing_found: "Nichts gefunden",
        loading: "Suche läuft",
    };
}

/// A `Carousel`'s strings. `{n}` is a one-based slide number, `{m}` the count.
/// A caller also passes `aria_label`/`slide_label` per instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarouselLabels {
    pub label: &'static str,
    pub previous: &'static str,
    pub next: &'static str,
    /// Names a dot: `{n}`, the first slide it shows.
    pub indicator: &'static str,
    /// Names the dot strip's group.
    pub indicators: &'static str,
    /// A slide group's accessible name: `{n}` and `{m}`.
    pub slide: &'static str,
    /// What the live region reads when the slide settles: `{n}` and `{m}`.
    pub status: &'static str,
    /// The live region above one slide per view: the slides showing,
    /// `{from}` to `{to}`, of `{n}`.
    pub status_range: &'static str,
    /// The autoplay button's name in both states: `aria-pressed` carries the state.
    pub pause: &'static str,
}

impl CarouselLabels {
    pub const ENGLISH: Self = Self {
        label: "Carousel",
        previous: "Previous slide",
        next: "Next slide",
        indicator: "Go to slide {n}",
        indicators: "Choose slide",
        slide: "{n} of {m}",
        status: "Slide {n} of {m}",
        status_range: "Slides {from}–{to} of {n}",
        pause: "Pause slideshow",
    };

    pub const GERMAN: Self = Self {
        label: "Karussell",
        previous: "Vorherige Folie",
        next: "Nächste Folie",
        indicator: "Zu Folie {n}",
        indicators: "Folie wählen",
        slide: "{n} von {m}",
        status: "Folie {n} von {m}",
        status_range: "Folien {from}–{to} von {n}",
        pause: "Diashow anhalten",
    };
}

/// A `NavLink` with nested links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavLinkLabels {
    /// Names the button that shows the nested links. The link's own name
    /// follows it, so it reads "Show links, Docs".
    pub show_links: &'static str,
}

impl NavLinkLabels {
    pub const ENGLISH: Self = Self {
        show_links: "Show links",
    };

    pub const GERMAN: Self = Self {
        show_links: "Links anzeigen",
    };
}

/// A `Lightbox`'s strings. Its close button reads [`CommonLabels::close`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightboxLabels {
    pub label: &'static str,
    /// Names the thumbnail strip, which is a region of its own.
    pub thumbnails: &'static str,
    /// Names a thumbnail: `{n}` is the slide number.
    pub thumbnail: &'static str,
    /// The zoomable picture's description: its keys.
    pub keys: &'static str,
    /// Announced after a zoom: `{n}` is the scale in percent of the fitted size.
    pub zoomed: &'static str,
    /// Announced once a zoom is back to the fitted size.
    pub fitted: &'static str,
    /// Names the toolbar's zoom-in button.
    pub zoom_in: &'static str,
    /// Names the toolbar's zoom-out button.
    pub zoom_out: &'static str,
}

impl LightboxLabels {
    pub const ENGLISH: Self = Self {
        label: "Gallery",
        thumbnails: "Thumbnails",
        thumbnail: "Go to slide {n}",
        keys: "Z, plus or minus to zoom. Arrow keys pan a zoomed picture, or change the picture.",
        zoomed: "Zoomed to {n}%",
        fitted: "Zoom reset",
        zoom_in: "Zoom in",
        zoom_out: "Zoom out",
    };

    pub const GERMAN: Self = Self {
        label: "Galerie",
        thumbnails: "Vorschaubilder",
        thumbnail: "Zu Bild {n}",
        keys: "Z, Plus oder Minus zum Zoomen. Pfeiltasten verschieben ein vergrößertes Bild oder wechseln das Bild.",
        zoomed: "Auf {n} % gezoomt",
        fitted: "Zoom zurückgesetzt",
        zoom_in: "Vergrößern",
        zoom_out: "Verkleinern",
    };
}

/// A `FloatingWindow`'s handles and its title-bar menu. Its close button reads
/// [`CommonLabels::close`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatingWindowLabels {
    /// Names the title bar, which is the keyboard move handle, and the group
    /// of move buttons.
    pub move_handle: &'static str,
    /// The move handle's description: how to move it.
    pub move_hint: &'static str,
    /// Names the corner resize handle, and the group of resize buttons.
    pub resize_handle: &'static str,
    /// The resize handle's value text: `{width}` and `{height}` in pixels.
    pub size: &'static str,
    /// Names the title bar's menu button.
    pub menu: &'static str,
    /// The menu item that shows the move buttons.
    pub move_item: &'static str,
    /// The menu item that shows the resize buttons.
    pub resize_item: &'static str,
    /// The menu item that puts the window back where and how it opened.
    pub reset_item: &'static str,
    pub move_up: &'static str,
    pub move_down: &'static str,
    pub move_left: &'static str,
    pub move_right: &'static str,
    pub narrower: &'static str,
    pub wider: &'static str,
    pub shorter: &'static str,
    pub taller: &'static str,
    /// Hides the move or resize buttons again.
    pub done: &'static str,
}

impl FloatingWindowLabels {
    pub const ENGLISH: Self = Self {
        move_handle: "Move window",
        move_hint: "Use arrow keys to move the window",
        resize_handle: "Resize window",
        size: "{width} by {height} pixels",
        menu: "Window menu",
        move_item: "Move",
        resize_item: "Resize",
        reset_item: "Reset position and size",
        move_up: "Move up",
        move_down: "Move down",
        move_left: "Move left",
        move_right: "Move right",
        narrower: "Narrower",
        wider: "Wider",
        shorter: "Shorter",
        taller: "Taller",
        done: "Done",
    };

    pub const GERMAN: Self = Self {
        move_handle: "Fenster verschieben",
        move_hint: "Mit den Pfeiltasten das Fenster verschieben",
        resize_handle: "Fenstergröße ändern",
        size: "{width} mal {height} Pixel",
        menu: "Fenstermenü",
        move_item: "Verschieben",
        resize_item: "Größe ändern",
        reset_item: "Position und Größe zurücksetzen",
        move_up: "Nach oben",
        move_down: "Nach unten",
        move_left: "Nach links",
        move_right: "Nach rechts",
        narrower: "Schmaler",
        wider: "Breiter",
        shorter: "Niedriger",
        taller: "Höher",
        done: "Fertig",
    };
}

/// The `Notifications` host's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationsLabels {
    /// Names the region around the stacks: `{key}` is the key that focuses
    /// the newest notification.
    pub region: &'static str,
}

impl NotificationsLabels {
    pub const ENGLISH: Self = Self {
        region: "Notifications ({key})",
    };

    pub const GERMAN: Self = Self {
        region: "Benachrichtigungen ({key})",
    };
}

/// A `Scroller`'s two controls. "Backward"/"forward" rather than
/// "left"/"right", so the names do not lie under a right-to-left page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollerLabels {
    pub backward: &'static str,
    pub forward: &'static str,
}

impl ScrollerLabels {
    pub const ENGLISH: Self = Self {
        backward: "Scroll backward",
        forward: "Scroll forward",
    };

    pub const GERMAN: Self = Self {
        backward: "Zurückscrollen",
        forward: "Vorwärtsscrollen",
    };
}

/// A `Sortable` item's drag handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortableLabels {
    pub handle: &'static str,
}

impl SortableLabels {
    pub const ENGLISH: Self = Self { handle: "Reorder" };

    pub const GERMAN: Self = Self {
        handle: "Neu anordnen",
    };
}

/// Read by a screen reader after a step's label. The marker's glyph is
/// drawing only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperLabels {
    pub completed: &'static str,
    /// So an error is not colour alone.
    pub error: &'static str,
}

impl StepperLabels {
    pub const ENGLISH: Self = Self {
        completed: "Completed",
        error: "Error",
    };

    pub const GERMAN: Self = Self {
        completed: "Abgeschlossen",
        error: "Fehler",
    };
}

/// A `Marquee`'s pause toggle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarqueeLabels {
    /// The toggle's name in both states: `aria-pressed` carries the state.
    pub pause: &'static str,
}

impl MarqueeLabels {
    pub const ENGLISH: Self = Self { pause: "Pause" };
    pub const GERMAN: Self = Self { pause: "Anhalten" };
}

/// A `FileField`'s own words. A chip's x reads [`CommonLabels::remove`].
///
/// ```
/// use libero::localization::FileFieldLabels;
///
/// const WORDS: FileFieldLabels = FileFieldLabels {
///     any_of: |group| match group {
///         "image" => "Fotos".to_string(),
///         group => (FileFieldLabels::GERMAN.any_of)(group),
///     },
///     ..FileFieldLabels::GERMAN
/// };
/// assert_eq!((WORDS.any_of)("image"), "Fotos");
/// assert_eq!((WORDS.any_of)("video"), "Videos");
/// assert_eq!((FileFieldLabels::ENGLISH.any_of)("audio"), "audios");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct FileFieldLabels {
    /// The `Input` variant's Browse button, heard after the field's label.
    pub browse: &'static str,
    /// The Browse button's description on a `required` field.
    pub required: &'static str,
    /// The dropzone's prompt when no `placeholder` or children are given.
    pub drop_file: &'static str,
    /// The same, for a `multiple` field.
    pub drop_files: &'static str,
    /// Names a `type/*` entry of `accept` in the dropzone's hint: `image`
    /// reads as "images". A fn rather than a template, for plural forms.
    pub any_of: fn(&str) -> String,
    /// A card's file size units, bytes to terabytes, in powers of 1000. The
    /// decimal separator is [`Formats::decimal_separator`](super::Formats).
    pub size_units: [&'static str; 5],
}

/// `FileFieldLabels::ENGLISH.any_of`. A named fn, so every copy compares equal.
fn english_any_of(group: &str) -> String {
    format!("{group}s")
}

/// `FileFieldLabels::GERMAN.any_of`: a German plural rarely adds a letter.
fn german_any_of(group: &str) -> String {
    match group {
        "image" => "Bilder".to_string(),
        "video" => "Videos".to_string(),
        "audio" => "Audiodateien".to_string(),
        "text" => "Textdateien".to_string(),
        "font" => "Schriftarten".to_string(),
        group => format!("{group}-Dateien"),
    }
}

/// `TextareaLabels::ENGLISH.characters_left`. A named fn, so every copy holds
/// the same address and compares equal.
fn english_characters_left(n: usize) -> String {
    match n {
        1 => "1 character left".to_string(),
        n => format!("{n} characters left"),
    }
}

/// `TextareaLabels::GERMAN.characters_left`. "Zeichen" is its own plural.
fn german_characters_left(n: usize) -> String {
    format!("Noch {n} Zeichen")
}

/// A `Textarea`'s `counter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct TextareaLabels {
    /// Announced near `maxlength`: how many characters are left. A fn rather
    /// than a template, for a language's plural forms.
    ///
    /// ```
    /// use libero::localization::TextareaLabels;
    ///
    /// const WORDS: TextareaLabels = TextareaLabels {
    ///     characters_left: |n| match n {
    ///         1 => "Nur noch 1 Zeichen".to_string(),
    ///         n => format!("Nur noch {n} Zeichen"),
    ///     },
    /// };
    /// assert_eq!((WORDS.characters_left)(3), "Nur noch 3 Zeichen");
    /// assert_eq!((TextareaLabels::GERMAN.characters_left)(1), "Noch 1 Zeichen");
    /// assert_eq!((TextareaLabels::ENGLISH.characters_left)(1), "1 character left");
    /// ```
    pub characters_left: fn(usize) -> String,
}

impl TextareaLabels {
    pub const ENGLISH: Self = Self {
        characters_left: english_characters_left,
    };

    pub const GERMAN: Self = Self {
        characters_left: german_characters_left,
    };
}

impl FileFieldLabels {
    pub const ENGLISH: Self = Self {
        browse: "Browse files",
        required: "Required",
        drop_file: "Drop a file here, or click to pick",
        drop_files: "Drop files here, or click to pick",
        any_of: english_any_of,
        size_units: ["B", "kB", "MB", "GB", "TB"],
    };

    pub const GERMAN: Self = Self {
        browse: "Dateien durchsuchen",
        required: "Pflichtfeld",
        drop_file: "Datei hier ablegen oder zum Auswählen klicken",
        drop_files: "Dateien hier ablegen oder zum Auswählen klicken",
        any_of: german_any_of,
        size_units: ["B", "kB", "MB", "GB", "TB"],
    };
}
