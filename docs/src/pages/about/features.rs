use crate::Route;
use crate::components::{DocPage, DocSection};
use crate::site::libero_version;
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Table, Text, column},
    sx::sx,
};

/// One component or hook feature of libero's `Cargo.toml`, as the docs describe it.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Feature {
    pub name: &'static str,
    /// The group feature that turns it on with its siblings.
    pub category: &'static str,
    /// The components or hooks it compiles.
    pub components: &'static str,
    /// The features it turns on: what it is built from.
    pub turns_on: &'static [&'static str],
    /// On through `default`, listed there or turned on by a feature that is.
    pub default: bool,
    /// What it adds besides its own code, or what it needs another feature for.
    pub note: &'static str,
    /// The docs pages of its components or hooks.
    pub routes: &'static [&'static str],
}

const fn feature(
    name: &'static str,
    category: &'static str,
    components: &'static str,
    turns_on: &'static [&'static str],
    default: bool,
    routes: &'static [&'static str],
) -> Feature {
    Feature {
        name,
        category,
        components,
        turns_on,
        default,
        note: "",
        routes,
    }
}

const fn noted(feature: Feature, note: &'static str) -> Feature {
    Feature { note, ..feature }
}

/// Every component and hook feature, by group, in `Cargo.toml`'s order.
#[rustfmt::skip]
pub(crate) const FEATURES: &[Feature] = &[
    feature("focus-trap", "accessibility", "FocusTrap", &["use-focus-return", "visually-hidden"], true, &["/accessibility/focus-trap"]),
    feature("visually-hidden", "accessibility", "VisuallyHidden", &["box"], true, &["/accessibility/visually-hidden"]),
    feature("action-icon", "buttons", "ActionIcon", &["loader", "tooltip", "kbd", "anchor", "paper", "pictogram", "use-hotkeys", "use-id"], true, &["/buttons/action-icon"]),
    feature("button", "buttons", "Button", &["loader", "anchor"], true, &["/buttons/button"]),
    feature("button-group", "buttons", "ButtonGroup", &["box"], false, &["/buttons/button-group"]),
    feature("copy", "buttons", "Copy", &["action-icon", "use-clipboard"], false, &["/buttons/copy"]),
    feature("direction-toggle", "buttons", "DirectionToggle", &["action-icon", "use-direction"], false, &["/buttons/direction-toggle"]),
    noted(
        feature("repository", "buttons", "Repository", &["action-icon", "use-local-storage"], false, &["/buttons/repository"]),
        "The Simple Icons brand marks (pictogram-icons-simple).",
    ),
    feature("share", "buttons", "Share", &["action-icon", "icon", "button", "use-share"], false, &["/buttons/share"]),
    feature("theme-switcher", "buttons", "ThemeSwitcher", &["menu", "action-icon", "use-color-scheme"], false, &["/buttons/theme-switcher"]),
    noted(
        feature("tldr", "buttons", "Tldr", &["chip", "menu", "icon", "action-icon"], false, &["/buttons/tldr"]),
        "The Lobe Icons brand marks (pictogram-icons-lobe).",
    ),
    feature("toolbar", "buttons", "Toolbar, ToolbarGroup, ToolbarSeparator", &["box", "use-hotkeys"], false, &["/buttons/toolbar"]),
    feature("accordion", "data-display", "Accordion", &["collapse", "use-icon", "use-id"], false, &["/data-display/accordion"]),
    feature("audio", "data-display", "Audio", &["slider", "menu", "button", "modal", "action-icon", "shortcut-help", "use-fullscreen", "use-media"], false, &["/data-display/audio"]),
    feature("avatar", "data-display", "Avatar, AvatarGroup", &["tooltip", "use-icon", "use-id", "visually-hidden"], false, &["/data-display/avatar"]),
    feature("badge", "data-display", "Badge", &["box"], false, &["/data-display/badge"]),
    feature("carousel", "data-display", "Carousel", &["action-icon", "scroll-area"], false, &["/data-display/carousel"]),
    noted(
        feature("charts", "data-display", "LineChart, BarChart, PieChart and the other charts", &["skeleton", "mark", "focus-trap", "use-accessibility", "use-popover", "use-debounce", "use-id", "use-icon"], false, &["/data-display/chart"]),
        "charts-rs, about 240 KB brotli on the web.",
    ),
    feature("data-list", "data-display", "DataList", &["box"], false, &["/data-display/data-list"]),
    feature("icon", "data-display", "Icon", &["pictogram", "visually-hidden"], true, &["/data-display/icon"]),
    noted(
        feature("image", "data-display", "Image", &["box"], false, &["/data-display/image"]),
        "Its zoomable prop needs lightbox on as well; without it the picture renders unzoomable.",
    ),
    feature("image-list", "data-display", "ImageList", &["image", "grid"], false, &["/data-display/image-list"]),
    feature("indicator", "data-display", "Indicator", &["box"], false, &["/data-display/indicator"]),
    feature("kanban", "data-display", "Kanban", &["sortable", "menu", "focus-trap"], false, &["/data-display/kanban"]),
    feature("list", "data-display", "List", &["box"], false, &["/data-display/list"]),
    feature("marquee", "data-display", "Marquee", &["action-icon"], false, &["/data-display/marquee"]),
    feature("pictogram", "data-display", "Pictogram", &["use-icon"], true, &["/data-display/pictogram"]),
    noted(
        feature("qr-code", "data-display", "QrCode", &["box"], false, &["/data-display/qr-code"]),
        "The fast_qr encoder.",
    ),
    feature("sortable", "data-display", "Sortable, use_sortable", &["use-drag", "use-icon", "use-id", "use-media-query", "use-timers", "visually-hidden"], false, &["/data-display/sortable"]),
    noted(
        feature("table", "data-display", "Table, column and the table toolbar buttons", &["badge", "progress-bar", "skeleton", "checkbox", "native-select", "select", "text-field", "pagination", "menu", "sortable", "icon", "action-icon", "button", "collapse", "use-debounce"], false, &["/data-display/table"]),
        "The largest component: its filters and editors bring the select and text fields. With date-time on, a date filter uses DateField.",
    ),
    feature("timeline", "data-display", "Timeline", &["visually-hidden"], false, &["/data-display/timeline"]),
    feature("video", "data-display", "Video", &["slider", "menu", "button", "modal", "action-icon", "shortcut-help", "use-fullscreen", "use-media"], false, &["/data-display/video"]),
    feature("alert", "feedback", "Alert", &["action-icon"], true, &["/feedback/alert"]),
    feature("circular-progress", "feedback", "CircularProgress", &["box", "use-accessibility"], false, &["/feedback/circular-progress"]),
    feature("loader", "feedback", "Loader", &["box"], true, &["/feedback/loader"]),
    feature("notifications", "feedback", "Notifications, use_notifications", &["alert", "button", "float"], false, &["/feedback/notifications"]),
    feature("progress-bar", "feedback", "ProgressBar", &["slider"], false, &["/feedback/progress-bar"]),
    feature("skeleton", "feedback", "Skeleton", &["box"], false, &["/feedback/skeleton"]),
    feature("autocomplete", "form", "Autocomplete", &["combobox", "action-icon"], false, &["/form/autocomplete"]),
    feature("cascader", "form", "Cascader", &["combobox", "action-icon", "use-media-query"], false, &["/form/cascader"]),
    feature("checkbox", "form", "Checkbox", &["use-icon", "use-id", "visually-hidden"], true, &["/form/checkbox"]),
    feature("chip", "form", "Chip", &["action-icon", "anchor"], true, &["/form/chip"]),
    feature("color", "form", "ColorField, ColorPicker, ColorSwatch, HueSlider, AlphaSlider", &["slider", "action-icon"], false, &["/form/color-field", "/form/color-picker"]),
    feature("combobox", "form", "Combobox, use_combobox", &["loader", "paper", "scroll-area", "use-id", "use-popover", "visually-hidden", "use-icon"], true, &["/form/combobox"]),
    feature("date-time", "form", "ChronoField, ChronoPicker, DateField, DatePicker, TimeField, TimePicker and the other date and time fields and pickers", &["tabs", "action-icon", "use-drag", "use-media-query"], false, &["/form/chrono-field", "/form/chrono-picker"]),
    feature("fieldset", "form", "Fieldset", &["box", "use-icon", "use-id", "visually-hidden"], true, &["/form/fieldset"]),
    noted(
        feature("file-field", "form", "FileField", &["chip", "select", "alert", "modal", "action-icon", "button", "dialog", "loader"], false, &["/form/file-field"]),
        "Its crop prop needs image-cropper on as well; without it the file passes uncropped.",
    ),
    feature("form-root", "form", "Form, use_form", &["alert"], true, &["/form/form"]),
    feature("image-cropper", "form", "ImageCropper", &["slider", "action-icon"], false, &["/form/image-cropper"]),
    feature("native-select", "form", "NativeSelect", &["combobox", "select"], false, &["/form/native-select"]),
    feature("number-field", "form", "NumberField", &["action-icon"], false, &["/form/number-field"]),
    feature("password-field", "form", "PasswordField", &["text-field", "action-icon"], false, &["/form/password-field"]),
    feature("phone-field", "form", "PhoneField", &["combobox"], false, &["/form/phone-field"]),
    feature("pin-field", "form", "PinField", &["use-icon", "visually-hidden", "use-id"], false, &["/form/pin-field"]),
    feature("radio", "form", "Radio", &["use-accessibility", "use-icon", "visually-hidden", "use-id"], false, &[]),
    feature("radio-group", "form", "RadioGroup", &["radio"], false, &["/form/radio-group"]),
    feature("rating", "form", "Rating", &["use-icon", "visually-hidden", "use-id", "use-drag"], false, &["/form/rating"]),
    feature("rich-text-editor", "form", "RichTextEditor, RichTextView", &["toolbar", "text-field", "menu", "code", "mark", "modal", "action-icon", "button", "dialog", "shortcut-help", "text", "flex", "use-history"], false, &["/form/rich-text-editor"]),
    feature("segmented-control", "form", "SegmentedControl", &["button", "use-id"], false, &["/form/segmented-control"]),
    feature("select", "form", "Select, MultiSelect", &["combobox", "chip", "action-icon"], true, &["/form/select", "/form/multi-select"]),
    feature("slider", "form", "Slider, RangeSlider", &["tooltip", "use-icon", "visually-hidden", "use-id", "use-drag"], false, &["/form/slider", "/form/range-slider"]),
    feature("switch", "form", "Switch", &["use-accessibility", "use-icon", "visually-hidden", "use-id"], true, &["/form/switch"]),
    feature("tags-field", "form", "TagsField", &["combobox", "chip", "select", "action-icon"], false, &["/form/tags-field"]),
    feature("text-field", "form", "TextField", &["use-icon", "visually-hidden", "use-id"], true, &["/form/text-field"]),
    feature("textarea", "form", "Textarea", &["use-debounce", "use-icon", "visually-hidden", "use-id"], false, &["/form/textarea"]),
    feature("aspect-ratio", "layout", "AspectRatio", &["box"], true, &["/layout/aspect-ratio"]),
    feature("box", "layout", "Box", &["use-element"], true, &["/layout/box"]),
    feature("center", "layout", "Center", &["box"], true, &["/layout/center"]),
    feature("collapse", "layout", "Collapse", &["box", "use-timers"], true, &["/layout/collapse"]),
    feature("container", "layout", "Container", &["box"], true, &["/layout/container"]),
    feature("divider", "layout", "Divider", &["use-id", "use-icon", "visually-hidden"], true, &["/layout/divider"]),
    feature("flex", "layout", "Flex", &["box"], true, &["/layout/flex"]),
    feature("float", "layout", "Float", &["box"], true, &["/layout/float"]),
    feature("grid", "layout", "Grid, GridItem, GridZone", &["box"], true, &["/layout/grid"]),
    feature("header", "layout", "Header", &["paper", "use-id", "use-icon", "visually-hidden"], true, &["/layout/header"]),
    feature("paper", "layout", "Paper", &["box"], true, &["/layout/paper"]),
    feature("scroll-area", "layout", "ScrollArea, use_scroll_area", &["box", "use-drag"], true, &["/layout/scroll-area"]),
    feature("scroller", "layout", "Scroller, use_scroller", &["scroll-area", "use-icon", "use-id", "visually-hidden"], true, &["/layout/scroller"]),
    feature("sidebar", "layout", "Sidebar", &["scroll-area"], true, &["/layout/sidebar"]),
    feature("splitter", "layout", "Splitter", &["use-drag", "use-id", "use-icon", "visually-hidden"], true, &["/layout/splitter"]),
    feature("transition", "layout", "Transition", &["box", "use-timers"], true, &["/layout/transition"]),
    feature("anchor", "navigation", "Anchor", &["use-icon", "visually-hidden"], true, &["/navigation/anchor"]),
    feature("burger", "navigation", "Burger", &["action-icon"], false, &["/navigation/burger"]),
    feature("menubar", "navigation", "Menubar", &["menu"], false, &["/navigation/menubar"]),
    feature("nav-link", "navigation", "NavLink", &["tree", "anchor", "collapse", "use-id"], false, &["/navigation/nav-link"]),
    feature("navigation-bar", "navigation", "NavigationBar", &["nav-link", "float"], false, &["/navigation/navigation-bar"]),
    feature("pagination", "navigation", "Pagination", &["action-icon"], false, &["/navigation/pagination"]),
    feature("stepper", "navigation", "Stepper", &["collapse", "use-icon", "use-id", "visually-hidden"], false, &["/navigation/stepper"]),
    feature("tabs", "navigation", "Tabs", &["use-id", "use-icon", "visually-hidden"], false, &["/navigation/tabs"]),
    feature("tree", "navigation", "Tree", &["list", "icon", "use-popover"], false, &["/navigation/tree"]),
    feature("dialog", "overlay", "Dialog", &["action-icon", "title"], true, &["/overlay/dialog"]),
    feature("drawer", "overlay", "use_drawer", &["dialog", "modal", "float", "use-swipe"], false, &["/overlay/drawer"]),
    feature("floating-window", "overlay", "use_floating_window", &["menu", "action-icon", "button", "title", "float", "use-drag"], false, &["/overlay/floating-window"]),
    feature("hover-card", "overlay", "HoverCard", &["paper", "scroll-area", "use-popover", "use-timers"], false, &["/overlay/hover-card"]),
    feature("lightbox", "overlay", "use_lightbox", &["carousel", "modal", "action-icon", "dialog"], false, &["/overlay/lightbox"]),
    feature("menu", "overlay", "Menu, use_menu", &["anchor", "kbd", "divider", "paper", "use-popover", "use-timers"], false, &["/overlay/menu"]),
    feature("modal", "overlay", "Modal, use_modal", &["overlay-backdrop", "focus-trap", "use-back"], true, &["/overlay/modal"]),
    feature("overlay-backdrop", "overlay", "Overlay", &["box"], true, &["/overlay/overlay"]),
    feature("shortcut-help", "overlay", "ShortcutHelp", &["dialog", "kbd", "use-element", "use-hotkeys"], false, &["/overlay/shortcut-help"]),
    feature("spotlight", "overlay", "use_spotlight", &["combobox", "dialog", "modal", "kbd", "loader"], false, &["/overlay/spotlight"]),
    feature("tooltip", "overlay", "Tooltip", &["use-timers", "box", "use-popover"], true, &["/overlay/tooltip"]),
    feature("tour", "overlay", "use_tour", &["dialog", "button", "focus-trap", "use-accessibility", "use-local-storage", "use-back"], false, &["/overlay/tour"]),
    feature("blockquote", "typography", "Blockquote", &["box"], true, &["/typography/blockquote"]),
    feature("code", "typography", "Code, CodeBlock", &["copy"], false, &["/typography/code", "/typography/code-block"]),
    feature("kbd", "typography", "Kbd", &["box"], true, &["/typography/kbd"]),
    feature("mark", "typography", "Mark", &["box"], true, &["/typography/mark"]),
    feature("text", "typography", "Text", &["box"], true, &["/typography/text"]),
    feature("title", "typography", "Title", &["box"], true, &["/typography/title"]),
    feature("use-accessibility", "all-hooks", "use_accessibility", &[], true, &["/accessibility/use-accessibility"]),
    feature("use-back", "all-hooks", "use_back", &["use-popover"], true, &["/hooks/use-back"]),
    feature("use-clipboard", "all-hooks", "use_clipboard", &[], true, &[]),
    feature("use-color-scheme", "all-hooks", "use_color_scheme", &[], true, &[]),
    feature("use-countdown", "all-hooks", "use_countdown, format_duration", &[], false, &["/hooks/use-countdown"]),
    feature("use-debounce", "all-hooks", "use_debounced_value, use_debounced_callback, use_throttled_value, use_throttled_callback", &["use-timers"], true, &["/hooks/use-debounce"]),
    feature("use-direction", "all-hooks", "use_direction", &[], false, &[]),
    feature("use-drag", "all-hooks", "use_drag", &["use-element"], true, &["/hooks/use-drag"]),
    feature("use-element", "all-hooks", "use_element", &[], true, &["/hooks/use-element"]),
    feature("use-focus-return", "all-hooks", "use_focus_return", &["use-element"], true, &["/accessibility/use-focus-return"]),
    feature("use-fullscreen", "all-hooks", "use_fullscreen", &["use-back"], false, &["/hooks/use-fullscreen"]),
    feature("use-geolocation", "all-hooks", "use_geolocation", &[], false, &["/hooks/use-geolocation"]),
    feature("use-history", "all-hooks", "use_history", &["use-timers"], false, &["/hooks/use-history"]),
    feature("use-hotkeys", "all-hooks", "use_hotkeys", &["use-popover"], true, &["/hooks/use-hotkeys"]),
    feature("use-icon", "all-hooks", "use_icon", &[], true, &[]),
    feature("use-id", "all-hooks", "use_id", &[], true, &["/accessibility/use-id"]),
    feature("use-indexed-db", "all-hooks", "use_indexed_db", &["use-local-storage"], false, &["/hooks/use-indexed-db"]),
    feature("use-intersection", "all-hooks", "use_intersection, use_in_viewport", &["use-element"], false, &["/hooks/use-intersection"]),
    feature("use-long-press", "all-hooks", "use_long_press", &["use-timers"], false, &["/hooks/use-long-press"]),
    feature("use-media", "all-hooks", "use_media", &["use-element"], false, &["/hooks/use-media"]),
    feature("use-media-query", "all-hooks", "use_media_query, use_is_mobile", &[], true, &["/hooks/use-media-query"]),
    feature("use-popover", "all-hooks", "use_popover", &["use-focus-return"], true, &["/overlay/popover"]),
    feature("use-push-subscription", "all-hooks", "use_push_subscription", &[], false, &[]),
    feature("use-share", "all-hooks", "use_share", &[], false, &[]),
    feature("use-local-storage", "all-hooks", "use_local_storage, use_session_storage", &[], true, &["/hooks/use-local-storage"]),
    feature("use-swipe", "all-hooks", "use_swipe, use_edge_swipe", &["use-direction", "use-drag"], false, &["/hooks/use-swipe"]),
    feature("use-system-notification", "all-hooks", "use_system_notification", &[], false, &["/hooks/use-system-notification"]),
    feature("use-timers", "all-hooks", "use_timeout, use_interval", &[], true, &["/hooks/use-timers"]),
    feature("use-user-media", "all-hooks", "use_user_media, use_user_media_devices", &[], false, &["/hooks/use-user-media"]),
];

/// The group features of `all-components`, each turning on every component feature of its group.
pub(crate) const CATEGORIES: [&str; 9] = [
    "accessibility",
    "buttons",
    "data-display",
    "feedback",
    "form",
    "layout",
    "navigation",
    "overlay",
    "typography",
];

/// The group of every hook feature.
pub(crate) const HOOKS: &str = "all-hooks";

/// libero's features that are no component or hook: name and what it does.
#[rustfmt::skip]
pub(crate) const OTHER: &[(&str, &str)] = &[
    ("native", "Reaches elements through Blitz under dioxus-native: subtree queries, focus, the clipboard, file dialogs."),
    ("desktop", "The desktop WebView on Linux: system notifications over the session bus, the save dialog."),
    ("fcm", "use_fcm, Firebase Cloud Messaging on Android. Ships firebase-messaging into the app."),
    ("collation", "Table sorts text by the app's language off the web too, as the browser's Intl.Collator does."),
    ("full-polymorphism", "The 28 rarer HTML tags for a component's component prop (head, iframe, video, template, ...)."),
    ("icons-bootstrap, icons-material, icons-phosphor, icons-tabler", "One pictogram set each for IconProvider."),
    ("code-lang-*", "One grammar each for Code and CodeBlock, from code-lang-bash to code-lang-yaml."),
    ("held-clock, fake-file-dialog", "Test harnesses only: a held timer clock and a scripted file dialog. Never in an app."),
];

/// Component pages with no feature: the component is always compiled.
pub(crate) const ALWAYS: &[&str] = &["/data-display/icon-provider"];

/// What a page's feature line says.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PageFeature {
    Feature(&'static Feature),
    Always,
}

/// The feature of the page at `route`; `None` for a page that is not a component's.
pub(crate) fn page_feature(route: &str) -> Option<PageFeature> {
    if ALWAYS.contains(&route) {
        return Some(PageFeature::Always);
    }
    FEATURES
        .iter()
        .find(|feature| feature.routes.contains(&route))
        .map(PageFeature::Feature)
}

/// `` `a`, `b` `` for a list of feature names.
#[cfg(test)]
fn names(features: &[&str]) -> String {
    features
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A page's `Feature:` line in its markdown mirror.
#[cfg(test)]
pub(crate) fn md_line(page: PageFeature) -> String {
    match page {
        PageFeature::Always => "Feature: none, always compiled ([features.md](features.md))".into(),
        PageFeature::Feature(feature) => {
            let default = if feature.default { "on" } else { "off" };
            let turns_on = match feature.turns_on {
                [] => String::new(),
                list => format!(", turns on {}", names(list)),
            };
            format!(
                "Feature: `{}` ({default} by default{turns_on}; [features.md](features.md))",
                feature.name
            )
        }
    }
}

/// The feature line under a component page's lead.
#[component]
pub(crate) fn FeatureNote(route: String) -> Element {
    let Some(page) = page_feature(&route) else {
        return rsx! {};
    };
    let link = rsx! { Anchor { to: Route::FeaturesPage {}, "Features" } };
    let turns_on = match page {
        PageFeature::Feature(feature) if !feature.turns_on.is_empty() => {
            format!(". Turns on {}", feature.turns_on.join(", "))
        }
        _ => String::new(),
    };
    match page {
        PageFeature::Always => rsx! {
            Text { size: "sm",
                "Always compiled, no Cargo feature needed. See "
                {link}
                "."
            }
        },
        PageFeature::Feature(feature) => rsx! {
            Text { size: "sm",
                "Cargo feature "
                Code { source: feature.name }
                if feature.default { ", on by default" } else { ", off by default" }
                "{turns_on}. See "
                {link}
                "."
            }
        },
    }
}

// snippet: ignore - a Cargo.toml fragment, not Rust
const RECOMMENDED: &str = concat!(
    r#"libero = { version = ""#,
    libero_version!(),
    r#"", default-features = false, features = [
    "table",
    "select",
    "date-time",
    "use-clipboard",
] }"#
);

/// libero's `default`, as `Cargo.toml` lists it.
#[rustfmt::skip]
pub(crate) const DEFAULT: &[&str] = &[
    "layout", "text", "title", "anchor", "kbd", "mark", "blockquote", "button", "action-icon", "icon",
    "alert", "loader", "tooltip", "dialog", "modal", "form-root", "fieldset", "text-field", "checkbox",
    "switch", "select", "use-media-query", "use-color-scheme", "use-clipboard", "use-debounce",
    "use-timers", "use-hotkeys", "use-local-storage",
];

fn feature_table(label: &'static str, first: &'static str, rows: Vec<Feature>) -> Element {
    rsx! {
        Table {
            aria_label: label,
            scroll: true,
            data: rows,
            columns: vec![
                column("Feature")
                    .value(|row: &Feature| row.name)
                    .render(|row: &Feature| rsx! { Code { source: row.name, sx: sx().white_space("nowrap") } }),
                column(first).value(|row: &Feature| row.components),
                column("Turns on").value(|row: &Feature| row.turns_on.join(", ")),
                column("Default").value(|row: &Feature| if row.default { "on" } else { "off" }),
                column("Notes").value(|row: &Feature| row.note),
            ],
        }
    }
}

#[component]
pub fn FeaturesPage() -> Element {
    let categories: Vec<(&str, String)> = CATEGORIES
        .iter()
        .chain([&HOOKS])
        .map(|category| {
            let members: Vec<&str> = FEATURES
                .iter()
                .filter(|feature| feature.category == *category)
                .map(|feature| feature.name)
                .collect();
            (*category, members.join(", "))
        })
        .collect();
    let of = |hooks: bool| -> Vec<Feature> {
        FEATURES
            .iter()
            .filter(|feature| (feature.category == HOOKS) == hooks)
            .copied()
            .collect()
    };

    rsx! {
        DocPage {
            title: "Features",
            markdown: "/md/features.md",
            lead: rsx! {
                Text {
                    "Every component and every hook sits behind its own Cargo feature. Turn on only what you use: "
                    "the wasm gets smaller and the builds faster. An assistant that manages your Cargo.toml can keep the list exact."
                }
            },

            DocSection {
                title: "Recommended",
                Text {
                    "Turn "
                    Code { source: "default-features" }
                    " off and list the components and hooks you use. A feature turns on the features it is built from, so "
                    Code { source: "table" }
                    " alone gives a working "
                    Code { source: "Table" }
                    ". The compiler names a missing one: an item behind an off feature is not in "
                    Code { source: "libero" }
                    "."
                }
                CodeBlock { source: RECOMMENDED, language: "toml" }
            }

            DocSection {
                title: "Default",
                Text {
                    "The default turns on the layout components, the common components and the common hooks: "
                    "{DEFAULT.join(\", \")}. With "
                    Code { source: "default-features = false" }
                    " and no features, libero compiles no component and no hook: only the provider, styling, theme and platform layer."
                }
            }

            DocSection {
                title: "Groups",
                Text {
                    "A group feature turns on every feature of its group. "
                    Code { source: "all-components" }
                    " turns on every component group, "
                    Code { source: "all-hooks" }
                    " every hook."
                }
                Table {
                    aria_label: "Group features",
                    scroll: true,
                    data: categories,
                    columns: vec![
                        column("Group")
                            .value(|row: &(&str, String)| row.0)
                            .render(|row: &(&str, String)| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                        column("Turns on").value(|row: &(&str, String)| row.1.clone()),
                    ],
                }
            }

            DocSection {
                title: "Components",
                Text {
                    Code { source: "form-root" }
                    " is Form and use_form, "
                    Code { source: "overlay-backdrop" }
                    " is Overlay. "
                    Code { source: "table" }
                    "'s date filter, FileField's "
                    Code { source: "crop" }
                    " and Image's "
                    Code { source: "zoomable" }
                    " use another feature only when it is on (Notes)."
                }
                {feature_table("Component features", "Components", of(false))}
            }

            DocSection {
                title: "Hooks",
                Text {
                    "One feature per hook module. The hooks of a component (use_modal, use_drawer, use_lightbox, use_sortable, ...) come with its feature."
                }
                {feature_table("Hook features", "Hooks", of(true))}
            }

            DocSection {
                title: "Other features",
                Table {
                    aria_label: "Other features",
                    scroll: true,
                    data: OTHER.to_vec(),
                    columns: vec![
                        column("Feature")
                            .value(|row: &(&str, &str)| row.0)
                            .render(|row: &(&str, &str)| rsx! { Code { source: row.0 } }),
                        column("Does").value(|row: &(&str, &str)| row.1),
                    ],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// libero's `[features]`: each name and the features it lists.
    fn cargo_features() -> BTreeMap<String, Vec<String>> {
        let toml = include_str!("../../../../libero/Cargo.toml");
        let section = toml.split("\n[features]\n").nth(1).unwrap();
        let mut features = BTreeMap::new();
        let mut current: Option<(String, String)> = None;
        for line in section.lines() {
            let line = line.split('#').next().unwrap().trim();
            match &mut current {
                Some((_, list)) => list.push_str(line),
                None => {
                    if let Some((name, rest)) = line.split_once(" = ") {
                        current = Some((name.to_string(), rest.to_string()));
                    }
                }
            }
            if let Some((name, list)) = current.take_if(|(_, list)| list.ends_with(']')) {
                let items = list
                    .split('"')
                    .skip(1)
                    .step_by(2)
                    .map(str::to_string)
                    .collect();
                features.insert(name, items);
            }
        }
        features
    }

    /// Every feature `default` turns on, directly or through another.
    fn default_closure(cargo: &BTreeMap<String, Vec<String>>) -> Vec<String> {
        let mut on: Vec<String> = Vec::new();
        let mut todo = cargo["default"].clone();
        while let Some(name) = todo.pop() {
            if name.starts_with("dep:") || on.contains(&name) {
                continue;
            }
            todo.extend(cargo.get(&name).cloned().unwrap_or_default());
            on.push(name);
        }
        on
    }

    /// The tables here are libero's `Cargo.toml`: names, what each turns on, the default.
    #[test]
    fn the_features_match_libero_cargo_toml() {
        let cargo = cargo_features();
        let on = default_closure(&cargo);
        let mut problems = Vec::new();
        for feature in FEATURES {
            let Some(listed) = cargo.get(feature.name) else {
                problems.push(format!("{}: not in Cargo.toml", feature.name));
                continue;
            };
            let turns_on: Vec<&str> = listed
                .iter()
                .filter(|item| !item.starts_with("dep:"))
                .map(String::as_str)
                .collect();
            if turns_on != feature.turns_on {
                problems.push(format!(
                    "{}: Cargo.toml turns on {turns_on:?}",
                    feature.name
                ));
            }
            if on.iter().any(|item| item == feature.name) != feature.default {
                problems.push(format!("{}: default is {}", feature.name, !feature.default));
            }
            if !cargo[feature.category]
                .iter()
                .any(|item| item == feature.name)
            {
                problems.push(format!("{}: not in `{}`", feature.name, feature.category));
            }
        }
        for category in CATEGORIES.iter().chain([&HOOKS]) {
            for member in &cargo[*category] {
                if !FEATURES.iter().any(|feature| feature.name == member) {
                    problems.push(format!("{category}: {member} has no row here"));
                }
            }
        }
        assert_eq!(cargo["all-components"], CATEGORIES);
        assert_eq!(cargo["default"], DEFAULT);
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
