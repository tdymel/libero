use crate::Route;
use crate::components::{DocPage, DocSection};
use crate::site::libero_version;
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Table, Text, column},
    sx::sx,
};

/// One component feature of libero's `Cargo.toml`, as the docs describe it.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Feature {
    pub name: &'static str,
    /// The category feature that turns it on with its siblings.
    pub category: &'static str,
    pub components: &'static str,
    /// The component features it turns on: what it is built from.
    pub turns_on: &'static [&'static str],
    pub default: bool,
    /// What it adds besides its own code: crates, or a notably large build.
    pub cost: &'static str,
    /// The docs pages of its components.
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
        cost: "",
        routes,
    }
}

const fn costly(feature: Feature, cost: &'static str) -> Feature {
    Feature { cost, ..feature }
}

/// Every component feature, by category, in `Cargo.toml`'s order.
#[rustfmt::skip]
pub(crate) const FEATURES: &[Feature] = &[
    feature("button-group", "buttons", "ButtonGroup", &[], true, &["/buttons/button-group"]),
    feature("copy", "buttons", "Copy", &[], true, &["/buttons/copy"]),
    feature("direction-toggle", "buttons", "DirectionToggle", &[], true, &["/buttons/direction-toggle"]),
    costly(
        feature("repository", "buttons", "Repository", &[], false, &["/buttons/repository"]),
        "The Simple Icons brand marks (pictogram-icons-simple).",
    ),
    feature("share", "buttons", "Share", &[], true, &["/buttons/share"]),
    feature("theme-switcher", "buttons", "ThemeSwitcher", &["menu"], true, &["/buttons/theme-switcher"]),
    costly(
        feature("tldr", "buttons", "Tldr", &["chip", "menu"], false, &["/buttons/tldr"]),
        "The Lobe Icons brand marks (pictogram-icons-lobe).",
    ),
    feature("toolbar", "buttons", "Toolbar, ToolbarGroup, ToolbarSeparator", &[], true, &["/buttons/toolbar"]),
    feature("accordion", "data-display", "Accordion", &[], true, &["/data-display/accordion"]),
    feature("audio", "data-display", "Audio", &["slider", "menu"], false, &["/data-display/audio"]),
    feature("avatar", "data-display", "Avatar, AvatarGroup", &[], true, &["/data-display/avatar"]),
    feature("badge", "data-display", "Badge", &[], true, &["/data-display/badge"]),
    feature("carousel", "data-display", "Carousel", &[], true, &["/data-display/carousel"]),
    costly(
        feature("charts", "data-display", "LineChart, BarChart, PieChart and the other charts", &["skeleton"], false, &["/data-display/chart"]),
        "charts-rs, about 240 KB brotli on the web.",
    ),
    feature("data-list", "data-display", "DataList", &[], true, &["/data-display/data-list"]),
    feature("image", "data-display", "Image", &["lightbox"], true, &["/data-display/image"]),
    feature("image-list", "data-display", "ImageList", &["image"], true, &["/data-display/image-list"]),
    feature("indicator", "data-display", "Indicator", &[], true, &["/data-display/indicator"]),
    feature("kanban", "data-display", "Kanban", &["sortable", "menu"], false, &["/data-display/kanban"]),
    feature("list", "data-display", "List", &[], true, &["/data-display/list"]),
    feature("marquee", "data-display", "Marquee", &[], false, &["/data-display/marquee"]),
    costly(
        feature("qr-code", "data-display", "QrCode", &[], false, &["/data-display/qr-code"]),
        "The fast_qr encoder.",
    ),
    feature("sortable", "data-display", "Sortable, use_sortable", &[], true, &["/data-display/sortable"]),
    costly(
        feature("table", "data-display", "Table, column and the table toolbar buttons", &["badge", "progress-bar", "skeleton", "checkbox", "native-select", "select", "text-field", "pagination", "menu", "sortable", "date-time"], true, &["/data-display/table"]),
        "The largest component: its filters and editors bring the date, select and text fields.",
    ),
    feature("timeline", "data-display", "Timeline", &[], true, &["/data-display/timeline"]),
    feature("video", "data-display", "Video", &["slider", "menu"], false, &["/data-display/video"]),
    feature("circular-progress", "feedback", "CircularProgress", &[], true, &["/feedback/circular-progress"]),
    feature("notifications", "feedback", "Notifications, use_notifications", &[], true, &["/feedback/notifications"]),
    feature("progress-bar", "feedback", "ProgressBar", &["slider"], true, &["/feedback/progress-bar"]),
    feature("skeleton", "feedback", "Skeleton", &[], true, &["/feedback/skeleton"]),
    feature("autocomplete", "form", "Autocomplete", &["combobox"], true, &["/form/autocomplete"]),
    feature("cascader", "form", "Cascader", &["combobox"], false, &["/form/cascader"]),
    feature("checkbox", "form", "Checkbox", &[], true, &["/form/checkbox"]),
    feature("chip", "form", "Chip", &[], true, &["/form/chip"]),
    feature("color", "form", "ColorField, ColorPicker, ColorSwatch, HueSlider, AlphaSlider", &["slider"], false, &["/form/color-field", "/form/color-picker"]),
    feature("combobox", "form", "Combobox, use_combobox", &[], true, &["/form/combobox"]),
    feature("date-time", "form", "ChronoField, ChronoPicker, DateField, DatePicker, TimeField, TimePicker and the other date and time fields and pickers", &["tabs"], true, &["/form/chrono-field", "/form/chrono-picker"]),
    feature("file-field", "form", "FileField", &["chip", "image-cropper", "select"], true, &["/form/file-field"]),
    feature("image-cropper", "form", "ImageCropper", &["slider"], true, &["/form/image-cropper"]),
    feature("native-select", "form", "NativeSelect", &["combobox", "select"], true, &["/form/native-select"]),
    feature("number-field", "form", "NumberField", &[], true, &["/form/number-field"]),
    feature("password-field", "form", "PasswordField", &["text-field"], true, &["/form/password-field"]),
    feature("phone-field", "form", "PhoneField", &["combobox"], false, &["/form/phone-field"]),
    feature("pin-field", "form", "PinField", &[], true, &["/form/pin-field"]),
    feature("radio", "form", "Radio", &[], true, &[]),
    feature("radio-group", "form", "RadioGroup", &["radio"], true, &["/form/radio-group"]),
    feature("rating", "form", "Rating", &[], true, &["/form/rating"]),
    feature("rich-text-editor", "form", "RichTextEditor, RichTextView", &["toolbar", "text-field", "menu", "code"], false, &["/form/rich-text-editor"]),
    feature("segmented-control", "form", "SegmentedControl", &[], true, &["/form/segmented-control"]),
    feature("select", "form", "Select, MultiSelect", &["combobox", "chip"], true, &["/form/select", "/form/multi-select"]),
    feature("slider", "form", "Slider, RangeSlider", &[], true, &["/form/slider", "/form/range-slider"]),
    feature("switch", "form", "Switch", &[], true, &["/form/switch"]),
    feature("tags-field", "form", "TagsField", &["combobox", "chip", "select"], true, &["/form/tags-field"]),
    feature("text-field", "form", "TextField", &[], true, &["/form/text-field"]),
    feature("textarea", "form", "Textarea", &[], true, &["/form/textarea"]),
    feature("bottom-navigation", "navigation", "BottomNavigation", &["nav-link"], true, &["/navigation/bottom-navigation"]),
    feature("burger", "navigation", "Burger", &[], true, &["/navigation/burger"]),
    feature("menubar", "navigation", "Menubar", &["menu"], true, &["/navigation/menubar"]),
    feature("nav-link", "navigation", "NavLink", &["tree"], true, &["/navigation/nav-link"]),
    feature("pagination", "navigation", "Pagination", &[], true, &["/navigation/pagination"]),
    feature("stepper", "navigation", "Stepper", &[], true, &["/navigation/stepper"]),
    feature("tabs", "navigation", "Tabs", &[], true, &["/navigation/tabs"]),
    feature("tree", "navigation", "Tree", &["list"], true, &["/navigation/tree"]),
    feature("drawer", "overlay", "use_drawer", &[], true, &["/overlay/drawer"]),
    feature("floating-window", "overlay", "use_floating_window", &["menu"], false, &["/overlay/floating-window"]),
    feature("hover-card", "overlay", "HoverCard", &[], true, &["/overlay/hover-card"]),
    feature("lightbox", "overlay", "use_lightbox", &["carousel"], true, &["/overlay/lightbox"]),
    feature("menu", "overlay", "Menu, use_menu", &[], true, &["/overlay/menu"]),
    feature("spotlight", "overlay", "use_spotlight", &["combobox"], false, &["/overlay/spotlight"]),
    feature("tour", "overlay", "use_tour", &[], false, &["/overlay/tour"]),
    feature("code", "typography", "Code, CodeBlock", &["copy"], true, &["/typography/code", "/typography/code-block"]),
];

/// The category features, each turning on every component feature of its category.
pub(crate) const CATEGORIES: [&str; 7] = [
    "buttons",
    "data-display",
    "feedback",
    "form",
    "navigation",
    "overlay",
    "typography",
];

/// Component pages with no feature: the component is always compiled.
pub(crate) const ALWAYS: &[&str] = &[
    "/accessibility/focus-trap",
    "/accessibility/visually-hidden",
    "/buttons/action-icon",
    "/buttons/button",
    "/data-display/icon",
    "/data-display/icon-provider",
    "/data-display/pictogram",
    "/feedback/alert",
    "/feedback/loader",
    "/form/fieldset",
    "/form/form",
    "/layout/aspect-ratio",
    "/layout/box",
    "/layout/center",
    "/layout/collapse",
    "/layout/container",
    "/layout/divider",
    "/layout/flex",
    "/layout/float",
    "/layout/grid",
    "/layout/header",
    "/layout/paper",
    "/layout/scroll-area",
    "/layout/scroller",
    "/layout/sidebar",
    "/layout/splitter",
    "/layout/transition",
    "/navigation/anchor",
    "/overlay/dialog",
    "/overlay/modal",
    "/overlay/overlay",
    "/overlay/popover",
    "/overlay/shortcut-help",
    "/overlay/tooltip",
    "/typography/blockquote",
    "/typography/kbd",
    "/typography/mark",
    "/typography/text",
    "/typography/title",
];

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
] }"#
);

const ALWAYS_COMPILED: &str = "The hooks (bar use_drawer, use_floating_window, use_lightbox and the \
    sortable hooks, which come with their component), the layout components (Box, Flex, Grid, Paper, ScrollArea, Header, Sidebar, \
    Splitter and the rest), Text, Title, Kbd, Mark, Blockquote, Button, ActionIcon, Icon, Pictogram, \
    Alert, Loader, Anchor, Dialog, Modal, Overlay, Tooltip, ShortcutHelp, use_popover, FocusTrap, \
    VisuallyHidden, Form and Fieldset. Most components are built from them.";

#[component]
pub fn FeaturesPage() -> Element {
    let categories: Vec<(&str, String)> = CATEGORIES
        .iter()
        .map(|category| {
            let members: Vec<&str> = FEATURES
                .iter()
                .filter(|feature| feature.category == *category)
                .map(|feature| feature.name)
                .collect();
            (*category, members.join(", "))
        })
        .collect();

    rsx! {
        DocPage {
            title: "Features",
            markdown: "/md/features.md",
            lead: rsx! {
                Text {
                    "Each component sits behind a Cargo feature, so an app compiles only the components it uses. "
                    "The default turns the common ones on, so an app that names no feature still builds."
                }
            },

            DocSection {
                title: "Recommended",
                Text {
                    "Turn "
                    Code { source: "default-features" }
                    " off and list the components you use, one feature each. A component's feature turns on "
                    "the features it is built from, so "
                    Code { source: "table" }
                    " alone gives a working "
                    Code { source: "Table" }
                    ". The compiler names a missing one: a component behind an off feature is not in "
                    Code { source: "libero::components" }
                    "."
                }
                CodeBlock { source: RECOMMENDED, language: "toml" }
                Text {
                    "Assistants writing code with libero should do the same: start from "
                    Code { source: "default-features = false" }
                    " and add the feature of each component they use."
                }
            }

            DocSection {
                title: "Always compiled",
                Text { "{ALWAYS_COMPILED}" }
            }

            DocSection {
                title: "Categories",
                Text {
                    "A category feature turns on every component feature of its category, the ones off by default too. "
                    Code { source: "all-components" }
                    " turns on every category."
                }
                Table {
                    aria_label: "Category features",
                    scroll: true,
                    data: categories,
                    columns: vec![
                        column("Category")
                            .value(|row: &(&str, String)| row.0)
                            .render(|row: &(&str, String)| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                        column("Turns on").value(|row: &(&str, String)| row.1.clone()),
                    ],
                }
            }

            DocSection {
                title: "Components",
                Text {
                    "Off by default are the rare or heavy ones: charts, media players, the kanban board, "
                    "the rich text editor, QR codes and the brand-mark buttons among them. "
                    "A feature on by default may be there only because another default one turns it on."
                }
                Table {
                    aria_label: "Component features",
                    scroll: true,
                    data: FEATURES.to_vec(),
                    columns: vec![
                        column("Feature")
                            .value(|row: &Feature| row.name)
                            .render(|row: &Feature| rsx! { Code { source: row.name, sx: sx().white_space("nowrap") } }),
                        column("Components").value(|row: &Feature| row.components),
                        column("Turns on").value(|row: &Feature| row.turns_on.join(", ")),
                        column("Default").value(|row: &Feature| if row.default { "on" } else { "off" }),
                        column("Cost").value(|row: &Feature| row.cost),
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

    /// The table here is libero's `Cargo.toml`: names, what each turns on, the default.
    #[test]
    fn the_features_match_libero_cargo_toml() {
        let cargo = cargo_features();
        let defaults = &cargo["default"];
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
            if defaults.iter().any(|item| item == feature.name) != feature.default {
                problems.push(format!("{}: default is {}", feature.name, !feature.default));
            }
            if !cargo[feature.category]
                .iter()
                .any(|item| item == feature.name)
            {
                problems.push(format!("{}: not in `{}`", feature.name, feature.category));
            }
            // "Off by default" means not compiled by default, so `default` lists what it turns on.
            for name in feature.turns_on.iter().filter(|_| feature.default) {
                if !defaults.iter().any(|item| item == name) {
                    problems.push(format!(
                        "{}: turns on {name}, not in `default`",
                        feature.name
                    ));
                }
            }
        }
        for category in CATEGORIES {
            for member in &cargo[category] {
                if !FEATURES.iter().any(|feature| feature.name == member) {
                    problems.push(format!("{category}: {member} has no row here"));
                }
            }
        }
        assert_eq!(cargo["all-components"], CATEGORIES);
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
