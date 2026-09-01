use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use crate::icons::CheckmarkIcon;
use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Code, GridSpan, Image, ImageBar, ImageItem, ImageList, Input, Text},
    sx::sx,
    use_theme,
};

/// Six pictures with six **different intrinsic heights**. Equal ones would
/// make `masonry` look identical to `standard` and demonstrate nothing - the
/// mistake the `Grid` page made with `dense` and three equal cards.
static GALLERY: [Asset; 6] = [
    asset!("/assets/gallery/1.svg"),
    asset!("/assets/gallery/2.svg"),
    asset!("/assets/gallery/3.svg"),
    asset!("/assets/gallery/4.svg"),
    asset!("/assets/gallery/5.svg"),
    asset!("/assets/gallery/6.svg"),
];

const TITLES: [&str; 6] = ["Breakfast", "Burger", "Camera", "Coffee", "Hats", "Honey"];
const AUTHORS: [&str; 6] = [
    "@rgbagirl",
    "@amirali",
    "@bkristastucchio",
    "@nolanissac",
    "@hjrc33",
    "@arwinneil",
];

/// What the `bar` control really changes: not a prop on `ImageList` - there is
/// none - but the `ImageBar` each `ImageItem` carries. A control whose effect
/// the code block cannot honestly write as `bar: "below"` has to write the
/// thing it actually does: the block has to be what a caller would type.
fn items_code(bar: &str) -> String {
    let bar = match bar {
        "none" => String::new(),
        "bottom" => "\n            .bar(ImageBar::new(p.title).subtitle(p.author))".to_string(),
        position => format!(
            "\n            .bar(\n                ImageBar::new(p.title)\n                    \
             .subtitle(p.author)\n                    .position(BarPosition::{}),\n            )",
            match position {
                "top" => "Top",
                _ => "Below",
            }
        ),
    };

    format!(
        "items: photos\n    .iter()\n    .map(|p| {{\n        ImageItem::new(rsx! {{\n            \
         Image {{ src: p.url.clone(), alt: p.alt.clone(), fit: \"cover\" }}\n        }}){bar}\n    }})\n    \
         .collect()"
    )
}

/// Real pages, so the Linking section's cells navigate rather than 404.
const LINK_TARGETS: [&str; 3] = [
    "/layout/grid",
    "/layout/aspect-ratio",
    "/data-display/image",
];

fn picture(index: usize) -> Element {
    rsx! {
        Image {
            src: GALLERY[index].to_string(),
            alt: format!("{} - a stylised landscape", TITLES[index]),
            fit: "cover",
        }
    }
}

fn items(bar: &str) -> Vec<ImageItem> {
    (0..GALLERY.len())
        .map(|index| {
            let item = ImageItem::new(picture(index));
            match bar {
                "none" => item,
                position => item.bar(
                    ImageBar::new(TITLES[index])
                        .subtitle(AUTHORS[index])
                        .action(rsx! {
                            ActionIcon {
                                // A `<button>` inherits no colour of its own, so an
                                // action on the bar's scrim has to be told - the
                                // `MultiSelect` precedent.
                                variant: "standard",
                                sx: sx().color("inherit"),
                                size: "sm",
                                aria_label: format!("Select {}", TITLES[index]),
                                CheckmarkIcon {}
                            }
                        })
                        .position(position.into()),
                ),
            }
        })
        .collect()
}

#[component]
pub fn ImageListPage() -> Element {
    let theme = use_theme();
    let defaults = &theme.image_list;

    rsx! {
        DocPage {
            title: "ImageList",
            source: "libero/src/components/layout/image_list/image_list.rs",
            markdown: "/md/image-list.md",
            properties: vec![
                props("ImageList", vec![
                    prop("items", "Vec<ImageItem>").doc("One cell each, in render order."),
                    prop("cols", "u8")
                        .default(defaults.cols.to_string())
                        .doc("Columns. Snapped to a divisor of twelve - 1, 2, 3, 4, 6 or 12 - because a cell is a span of a `GridZone`'s twelve tracks. One value, not one per breakpoint: use `sx().breakpoint(..)` until per-breakpoint props land."),
                    prop("variant", "ImageListVariant")
                        .default(defaults.variant.as_str())
                        .doc("`standard` gives every cell the same height; `masonry` keeps each picture's own and packs them with no dead space."),
                    prop("gap", "Size")
                        .default(defaults.gap.as_str())
                        .doc("Between cells."),
                    prop("radius", "Size")
                        .default(defaults.radius.as_str())
                        .doc("Each cell's corner radius."),
                    prop("ratio", "f32")
                        .default("1.0, from theme.aspect_ratio")
                        .doc("Cell aspect ratio, e.g. `16.0 / 9.0`. Ignored by `masonry`, where the picture's own height is the point."),
                ]),
                props("ImageItem", vec![
                    prop("new(content)", "Element").doc("The cell's content - an `Image` with `fit: \"cover\"`, usually."),
                    prop("span(span)", "GridSpan").doc("This cell's width, overriding the one `cols` derives. The same twelfths a `GridItem` takes."),
                    prop("bar(bar)", "ImageBar").doc("The caption strip."),
                    prop("to(target)", "NavigationTarget").doc("Makes the cell a link. The anchor wraps the bar's title, or the picture when there is no bar, and a stretched `::after` extends the hit area over the tile."),
                ]),
                props("ImageBar", vec![
                    prop("new(title)", "OptionLabel").doc("The caption. Sibling content, not a label for the image."),
                    prop("subtitle(subtitle)", "OptionLabel").doc("A second, dimmer line."),
                    prop("action(action)", "Element").doc("A control at the end of the bar. Stays clickable on a cell with a `to`, because it is a sibling of the anchor."),
                    prop("position(position)", "BarPosition").doc("`bottom`, `top` or `below` - overriding the theme's, per cell."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A grid of pictures, each with an optional caption bar. Renders a "
                    Code { source: "<ul role=\"list\">" }
                    " of "
                    Code { source: "<li>" }
                    "s, so a gallery is announced with a count - and every cell is a "
                    Code { source: "GridItem" }
                    " of a "
                    Code { source: "GridZone" }
                    ", so "
                    Code { source: "cols" }
                    " is a span of the library's own twelve tracks rather than a second "
                    "grid with its own track count."
                }
                Text {
                    Code { source: "masonry" }
                    " is that zone's measuring engine, not a CSS multi-column - so the "
                    "reading order and the visual order agree, which is where MUI's own "
                    "masonry parts company with its DOM. Each picture's accessible name is "
                    "its own "
                    Code { source: "alt" }
                    "; the bar is sibling content and never becomes one."
                }
            },
            Demo {
                component: "ImageList",
                children_text: "",
                controls: vec![
                    Control::slider("cols", ["1", "2", "3", "4", "6"])
                        .default(defaults.cols.to_string())
                        // Unquoted: `cols` is a `u8`, not a string.
                        .code(|control, values| {
                            let value = values.str("cols");
                            match value == control.default {
                                true => vec![],
                                false => vec![format!("cols: {value}u8")],
                            }
                        }),
                    Control::toggle("variant", ["standard", "masonry"])
                        .default(defaults.variant.as_str()),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(defaults.gap.as_str()),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(defaults.radius.as_str()),
                    // `masonry` takes the picture's own height, so `ratio` does
                    // not exist in that state - an ignored prop must not print.
                    Control::toggle("ratio", ["1", "4:3", "16:9"])
                        .hidden_when(|values| values.str("variant") == "masonry")
                        .code(|_, values| match values.str("ratio").as_str() {
                            "4:3" => vec!["ratio: 4.0 / 3.0".to_string()],
                            "16:9" => vec!["ratio: 16.0 / 9.0".to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("bar", ["none", "bottom", "top", "below"])
                        .default(defaults.bar_position.as_str())
                        .code(|_, values| vec![items_code(&values.str("bar"))]),
                ],
                render: move |values: DemoValues| rsx! {
                    ImageList {
                        cols: values.str("cols").parse::<u8>().unwrap_or(defaults.cols),
                        variant: values.str("variant"),
                        gap: values.str("gap"),
                        radius: values.str("radius"),
                        // Left unset under `masonry`, which warns on a ratio
                        // it is going to ignore - the control is hidden there,
                        // so the prop must not be set behind its back either.
                        ratio: match (
                            values.str("variant").as_str(),
                            values.str("ratio").as_str(),
                        ) {
                            ("masonry", _) => Input::None,
                            (_, "4:3") => Input::Value(4.0 / 3.0),
                            (_, "16:9") => Input::Value(16.0 / 9.0),
                            _ => Input::None,
                        },
                        items: items(&values.str("bar")),
                    }
                },
            }
            DocSection {
                title: "Spans",
                Text {
                    "A cell's width is a per-item value, so no control can drive it: "
                    Code { source: "ImageItem::span" }
                    " takes the same twelfths a "
                    Code { source: "GridItem" }
                    " does, and overrides whatever "
                    Code { source: "cols" }
                    " derived. The first picture below is "
                    Code { source: "GridSpan::Half" }
                    " in a four-column list."
                }
                ImageList {
                    cols: 4u8,
                    items: (0..GALLERY.len())
                        .map(|index| {
                            let item = ImageItem::new(picture(index))
                                .bar(ImageBar::new(TITLES[index]));
                            match index {
                                0 => item.span(GridSpan::Half),
                                _ => item,
                            }
                        })
                        .collect::<Vec<_>>(),
                }
            }
            DocSection {
                title: "Linking a cell",
                Text {
                    Code { source: "ImageItem::to" }
                    " makes the whole tile a hit target. The anchor wraps the bar's title - "
                    "or the picture, when there is no bar - and a stretched "
                    Code { source: "::after {{ inset: 0 }}" }
                    " extends the hit area over the cell. The bar's "
                    Code { source: "action" }
                    " stays clickable because it is a sibling of the anchor rather than "
                    "inside it: a "
                    Code { source: "<button>" }
                    " inside an "
                    Code { source: "<a>" }
                    " is invalid HTML. So the accessible name is the title's text, or the "
                    "picture's "
                    Code { source: "alt" }
                    " - the first two cells below have a bar and the third does not."
                }
                ImageList {
                    cols: 3u8,
                    items: LINK_TARGETS
                        .iter()
                        .enumerate()
                        .map(|(index, target)| {
                            let item = ImageItem::new(picture(index)).to(*target);
                            match index {
                                2 => item,
                                _ => item.bar(
                                    ImageBar::new(TITLES[index])
                                        .subtitle(AUTHORS[index])
                                        .action(rsx! {
                                            ActionIcon {
                                                // A `<button>` inherits no colour of its own, so an
                                                // action on the bar's scrim has to be told - the
                                                // `MultiSelect` precedent.
                                                variant: "standard",
                                                sx: sx().color("inherit"),
                                                size: "sm",
                                                aria_label: format!("Select {}", TITLES[index]),
                                                CheckmarkIcon {}
                                            }
                                        }),
                                ),
                            }
                        })
                        .collect::<Vec<_>>(),
                }
            }
        }
    }
}
