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
}

/// A `CodeBlock`'s copy button and header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeBlockLabels {
    pub copy: &'static str,
    /// Announced once the copy landed.
    pub copied: &'static str,
    pub copy_failed: &'static str,
    /// The header's language name when there is no known language.
    pub unrecognized_language: &'static str,
    /// Read before a `diff` line that starts with `+`; the `+` is drawing only.
    pub added: &'static str,
    /// Read before a `diff` line that starts with `-`.
    pub removed: &'static str,
    /// The scroll region's name when the code overflows and has no known language.
    pub code: &'static str,
    /// The scroll region's name with a known language; `{language}` is its name.
    pub code_named: &'static str,
}

impl CodeBlockLabels {
    pub const ENGLISH: Self = Self {
        copy: "Copy code",
        copied: "Copied",
        copy_failed: "Copy failed",
        unrecognized_language: "Unrecognized language",
        added: "Added",
        removed: "Removed",
        code: "Code",
        code_named: "{language} code",
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
}

impl PhoneFieldLabels {
    pub const ENGLISH: Self = Self {
        search: "Search countries",
        country: "Country: {name}, {iso} +{dial}",
    };
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
}

/// What a `ColorSchemeButton` announces itself with. The three `to_*` name
/// what a press *does*, as the glyph beside them shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSchemeButtonLabels {
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

impl ColorSchemeButtonLabels {
    pub const ENGLISH: Self = Self {
        to_light: "Switch to the light theme",
        to_dark: "Switch to the dark theme",
        to_system: "Follow the system theme",
        group: "Theme",
        picker: "Choose a theme",
        themes: "Themes",
    };
}

/// Every string a `Spotlight` says to a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpotlightLabels {
    /// Names the dialog, unless `SpotlightOptions::aria_label` does.
    pub label: &'static str,
    pub placeholder: &'static str,
    /// Shown, and announced, when a query matches nothing.
    pub nothing_found: &'static str,
    /// Announced while `SpotlightOptions::loading` is set.
    pub loading: &'static str,
}

impl SpotlightLabels {
    pub const ENGLISH: Self = Self {
        label: "Command palette",
        placeholder: "Search...",
        nothing_found: "Nothing found",
        loading: "Searching",
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
    /// The autoplay button's name. It stays the same whether the slideshow
    /// runs or not: `aria-pressed` carries the state.
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
}

/// A `Lightbox`'s strings. Its close button reads [`CommonLabels::close`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightboxLabels {
    pub label: &'static str,
    /// Names the thumbnail strip, which is a region of its own.
    pub thumbnails: &'static str,
    /// Names a thumbnail: `{n}` is the slide number.
    pub thumbnail: &'static str,
}

impl LightboxLabels {
    pub const ENGLISH: Self = Self {
        label: "Gallery",
        thumbnails: "Thumbnails",
        thumbnail: "Go to slide {n}",
    };
}

/// A `FloatingWindow`'s handles. Its close button reads
/// [`CommonLabels::close`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatingWindowLabels {
    /// Names the title bar, which is the keyboard move handle.
    pub move_handle: &'static str,
    /// The move handle's description: how to move it.
    pub move_hint: &'static str,
    /// Names the corner resize handle.
    pub resize_handle: &'static str,
}

impl FloatingWindowLabels {
    pub const ENGLISH: Self = Self {
        move_handle: "Move window",
        move_hint: "Use arrow keys to move the window",
        resize_handle: "Resize window",
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarqueeLabels {
    /// The pause toggle's name. It stays the same whether the marquee is
    /// running or not: `aria-pressed` carries the state.
    pub pause: &'static str,
}

impl MarqueeLabels {
    pub const ENGLISH: Self = Self { pause: "Pause" };
}

/// A `FileField`'s own words. A chip's x reads [`CommonLabels::remove`].
///
/// ```
/// use libero::localization::FileFieldLabels;
///
/// const GERMAN: FileFieldLabels = FileFieldLabels {
///     any_of: |group| match group {
///         "image" => "Bilder".to_string(),
///         "video" => "Videos".to_string(),
///         group => format!("{group}-Dateien"),
///     },
///     ..FileFieldLabels::ENGLISH
/// };
/// assert_eq!((GERMAN.any_of)("image"), "Bilder");
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
    /// A card's file size units, bytes to terabytes, in powers of 1000.
    pub size_units: [&'static str; 5],
}

/// `FileFieldLabels::ENGLISH.any_of`. A named fn, so every copy compares equal.
fn english_any_of(group: &str) -> String {
    format!("{group}s")
}

/// `TextareaLabels::ENGLISH.characters_left`. A named fn, so every copy holds
/// the same address and compares equal.
fn english_characters_left(n: usize) -> String {
    match n {
        1 => "1 character left".to_string(),
        n => format!("{n} characters left"),
    }
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
    /// const GERMAN: TextareaLabels = TextareaLabels {
    ///     characters_left: |n| match n {
    ///         1 => "Noch 1 Zeichen".to_string(),
    ///         n => format!("Noch {n} Zeichen"),
    ///     },
    /// };
    /// assert_eq!((GERMAN.characters_left)(3), "Noch 3 Zeichen");
    /// assert_eq!((TextareaLabels::ENGLISH.characters_left)(1), "1 character left");
    /// ```
    pub characters_left: fn(usize) -> String,
}

impl TextareaLabels {
    pub const ENGLISH: Self = Self {
        characters_left: english_characters_left,
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
}
