//! Compiles every ` ```rust ` block in `docs/public/md/*.md` as a doc-test, so a
//! printed example that stops compiling fails `cargo test -p libero`. Only built
//! under `cfg(doctest)`; run just these with `cargo test -p libero --doc md_examples`.
//!
//! A complete example that names a placeholder - the reader's `Route`, a
//! `Photo`, an icon, a `submit` function - still compiles: the stand-ins go
//! at the end of the block as hidden `# ` lines, rustdoc's own convention,
//! so a reader sees the example and the test sees a whole program. Only a
//! fragment on purpose (a prop list, a block that continues an earlier one)
//! is fenced `rust,ignore`. A bare ` ``` ` fence is Rust to rustdoc too, so
//! plain text is fenced `text`.
//! `tests/all/md_examples.rs` checks that every md file is listed here.
//!
//! `PageSnippets` does the same for the code the docs pages print, from
//! `const`s and from each `Demo`'s generated code block, which the docs
//! crate's `snippets.rs` test writes into one md file. That file is gitignored,
//! so `cargo test -p docs page_snippets` runs first: in a fresh clone the
//! doc-tests fail on the missing file, and the line rustc quotes names the
//! command. Failing, not skipping: a skip would leave the printed code
//! unchecked behind a green run.

macro_rules! md_pages {
    ($($page:ident => $file:literal,)*) => {
        $(
            #[doc = include_str!(concat!("../../docs/public/md/", $file, ".md"))]
            pub struct $page;
        )*
    };
}

/// The code the docs pages print, as `docs/src/snippets.rs` writes it out.
#[doc = include_str!("../tests/page_snippets.md")] // Missing? Run `cargo test -p docs page_snippets` first.
pub struct PageSnippets;

md_pages! {
    Accessibility => "accessibility",
    Accordion => "accordion",
    ActionIcon => "action_icon",
    Alert => "alert",
    Anchor => "anchor",
    AspectRatio => "aspect_ratio",
    Autocomplete => "autocomplete",
    Avatar => "avatar",
    Badge => "badge",
    Blockquote => "blockquote",
    Box => "box",
    Burger => "burger",
    Button => "button",
    ButtonGroup => "button_group",
    Carousel => "carousel",
    Cascader => "cascader",
    Center => "center",
    Checkbox => "checkbox",
    Chip => "chip",
    Code => "code",
    CodeBlock => "code_block",
    Collapse => "collapse",
    ColorField => "color_field",
    ColorPicker => "color_picker",
    Combobox => "combobox",
    Container => "container",
    CopyButton => "copy_button",
    Credits => "credits",
    DataList => "data_list",
    ChronoField => "chrono_field",
    ChronoPicker => "chrono_picker",
    Dialog => "dialog",
    DirectionToggle => "direction_toggle",
    Divider => "divider",
    Drawer => "drawer",
    Fieldset => "fieldset",
    FileField => "file_field",
    Flex => "flex",
    Float => "float",
    FloatingWindow => "floating_window",
    FocusTrap => "focus_trap",
    Form => "form",
    FormGettingStarted => "form_getting_started",
    GettingStarted => "getting_started",
    Grid => "grid",
    Header => "header",
    HoverCard => "hover_card",
    Icon => "icon",
    Hooks => "hooks",
    Image => "image",
    ImageList => "image_list",
    Index => "index",
    Indicator => "indicator",
    Kbd => "kbd",
    Lightbox => "lightbox",
    List => "list",
    Loader => "loader",
    Localization => "localization",
    Mark => "mark",
    Marquee => "marquee",
    Menu => "menu",
    Menubar => "menubar",
    Modal => "modal",
    MultiSelect => "multi_select",
    NativeSelect => "native_select",
    NavLink => "nav_link",
    Notifications => "notifications",
    NumberField => "number_field",
    Overlay => "overlay",
    Pagination => "pagination",
    Paper => "paper",
    PasswordField => "password_field",
    Philosophy => "philosophy",
    PhoneField => "phone_field",
    Pictogram => "pictogram",
    PinField => "pin_field",
    Platform => "platform",
    Popover => "popover",
    ProgressBar => "progress_bar",
    QrCode => "qr_code",
    RadioGroup => "radio_group",
    RangeSlider => "range_slider",
    RepoButton => "repo_button",
    ScrollArea => "scroll_area",
    Scroller => "scroller",
    SegmentedControl => "segmented_control",
    Select => "select",
    Sidebar => "sidebar",
    Skeleton => "skeleton",
    Slider => "slider",
    Splitter => "splitter",
    Spotlight => "spotlight",
    Stepper => "stepper",
    Styling => "styling",
    Switch => "switch",
    Table => "table",
    Tabs => "tabs",
    TagsField => "tags_field",
    Text => "text",
    TextField => "text_field",
    Textarea => "textarea",
    ThemeToggle => "theme_toggle",
    Theming => "theming",
    Timeline => "timeline",
    Title => "title",
    Tldr => "tldr",
    Tooltip => "tooltip",
    Tree => "tree",
    UseAccessibility => "use_accessibility",
    UseDebounce => "use_debounce",
    UseDrag => "use_drag",
    UseElement => "use_element",
    UseFocusReturn => "use_focus_return",
    UseHotkeys => "use_hotkeys",
    UseId => "use_id",
    UseIntersection => "use_intersection",
    UseLongPress => "use_long_press",
    UseMediaQuery => "use_media_query",
    UseStylesheet => "use_stylesheet",
    UseThemeSet => "use_theme_set",
    UseTimers => "use_timers",
    VisuallyHidden => "visually_hidden",
}
