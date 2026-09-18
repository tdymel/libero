use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Image, Kbd, Text},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
    sx::{StaticSx, sx},
};

/// The `ImageList` page's six pictures, in different sizes, so the stage
/// visibly fits each one.
static GALLERY: [Asset; 6] = [
    asset!("/assets/gallery/1.svg"),
    asset!("/assets/gallery/2.svg"),
    asset!("/assets/gallery/3.svg"),
    asset!("/assets/gallery/4.svg"),
    asset!("/assets/gallery/5.svg"),
    asset!("/assets/gallery/6.svg"),
];

const TITLES: [&str; 6] = ["Breakfast", "Burger", "Camera", "Coffee", "Hats", "Honey"];

/// Every `LightboxOptions` switch, in the order the struct declares them.
const SWITCHES: [&str; 5] = [
    "zoom",
    "thumbnails",
    "captions",
    "controls",
    "close_on_swipe_down",
];

static GRID_SX: StaticSx = StaticSx::new(|| {
    sx().display("grid")
        .grid_template_columns("repeat(3, 96px)")
        .gap("xs")
});

static THUMB_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("96px")
        .height("96px")
        .padding("0")
        .border_width("0")
        .cursor("zoom-in")
});

/// The options, the items and the trigger are the whole example, so the code
/// block is written out rather than generated from props.
fn wrap_hook(values: &DemoValues, _: &str) -> String {
    let changed: String = SWITCHES
        .iter()
        .filter(|name| values.str(name) != "true")
        .map(|name| format!("        {name}: false,\n"))
        .collect();
    format!(
        "let lightbox = use_lightbox(LightboxOptions {{\n\
         {changed}    ..LightboxOptions::default()\n\
         }});\n\
         let items: Vec<LightboxItem> = photos\n    \
             .iter()\n    \
             .map(|p| LightboxItem::new(&p.src, &p.alt).caption(&p.title))\n    \
             .collect();\n\n\
         rsx! {{\n    \
             for (index, photo) in photos.iter().enumerate() {{\n        \
                 Box {{\n            \
                     component: \"button\",\n            \
                     r#type: \"button\",\n            \
                     aria_label: \"Open {{photo.alt}}\",\n            \
                     onclick: {{\n                \
                         let items = items.clone();\n                \
                         move |_| {{ lightbox.open_with((items.clone(), index)); }}\n            \
                     }},\n            \
                     Image {{ src: \"{{photo.src}}\", alt: \"\", fit: \"cover\" }}\n        \
                 }}\n    \
             }}\n\
         }}"
    )
}

/// The hook needs its own scope and reads its options once, so the page keys
/// this on them and a switch remounts it.
#[component]
fn LightboxDemo(
    zoom: bool,
    thumbnails: bool,
    captions: bool,
    controls: bool,
    close_on_swipe_down: bool,
) -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        zoom,
        thumbnails,
        captions,
        controls,
        close_on_swipe_down,
        ..LightboxOptions::default()
    });
    let items: Vec<LightboxItem> = GALLERY
        .iter()
        .zip(TITLES)
        .map(|(src, title)| LightboxItem::new(src.to_string(), title).caption(title))
        .collect();

    rsx! {
        Box { framework_sx: &GRID_SX,
            for (index, (src, title)) in GALLERY.iter().zip(TITLES).enumerate() {
                Box {
                    key: "{index}",
                    component: "button",
                    r#type: "button",
                    framework_sx: &THUMB_SX,
                    aria_label: "Open {title}",
                    onclick: {
                        let items = items.clone();
                        move |_| {
                            lightbox.open_with((items.clone(), index));
                        }
                    },
                    Image { src: src.to_string(), alt: "", fit: "cover" }
                }
            }
        }
    }
}

#[component]
pub fn LightboxPage() -> Element {
    rsx! {
        DocPage {
            title: "Lightbox",
            source: "libero/src/components/overlay/use_lightbox.rs",
            markdown: "/md/lightbox.md",
            properties: vec![
                props("LightboxOptions", vec![
                    prop("zoom", "bool").default("true").doc("Lets the zoom buttons, the wheel, a double-click, `z`, `+` and `-` zoom, and a drag, a click or the arrows pan. On desktop and mobile a drag stops once the pointer leaves the picture."),
                    prop("max_zoom", "Option<f64>").default("8.0").doc("Upper scale bound. Unset, the theme's."),
                    prop("thumbnails", "bool").default("true").doc("The strip under the stage. Never shown for one picture."),
                    prop("captions", "bool").default("true").doc("Shows each item's caption."),
                    prop("controls", "bool").default("true").doc("The previous and next arrows."),
                    prop("preload", "usize").default("1").doc("Pictures on each side loaded at once. The rest load lazily."),
                    prop("close_on_swipe_down", "bool").default("true").doc("A downward touch swipe closes. Off while zoomed."),
                    prop("aria_label", "Option<String>").default("\"Gallery\"").doc("Names the dialog. Unset, the localization's label."),
                ]).without_base_props(),
                props("LightboxItem", vec![
                    prop("src", "String").default("required").doc("The picture."),
                    prop("thumbnail_src", "Option<String>").default("follows src").doc("A smaller source for the strip."),
                    prop("alt", "String").default("required").doc("The picture's text alternative."),
                    prop("caption", "Option<String>").doc("Shown under the stage."),
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "A modal image viewer. "
                    Code { source: "use_lightbox" }
                    " is "
                    Code { source: "use_modal" }
                    " with a gallery around it. Each opening carries its pictures and where to "
                    "start, and focus returns to the thumbnail that opened it. "
                    Code { source: "Image {{ zoomable }}" }
                    " is this viewer with one picture."
                }
                Text {
                    "Double-click or press "
                    Kbd { "z" }
                    " to step through 2x, 4x and 8x and back to fitted. The zoom buttons, "
                    Kbd { "+" }
                    " and "
                    Kbd { "-" }
                    " zoom in finer steps. Scroll to zoom at the cursor. Drag, click or use the arrows "
                    "to pan. At the edge of a pan the arrows move to the next picture, so the "
                    "keyboard never gets stuck."
                }
            },
            // snippet: item struct Photo { src: String, alt: String, title: String }
            // snippet: let photos: Vec<Photo> = Vec::new();
            Demo {
                component: "LightboxOptions",
                children_text: "",
                controls: SWITCHES.iter().map(|name| Control::switch(name).default("true")).collect(),
                render: move |values: DemoValues| {
                    let on = |name: &str| values.str(name) == "true";
                    let key = SWITCHES.map(|name| values.str(name)).join("-");
                    rsx! {
                        LightboxDemo {
                            key: "{key}",
                            zoom: on("zoom"),
                            thumbnails: on("thumbnails"),
                            captions: on("captions"),
                            controls: on("controls"),
                            close_on_swipe_down: on("close_on_swipe_down"),
                        }
                    }
                },
                wrap: Wrap(wrap_hook),
            }
            DocSection { title: "Accessibility",
                Text {
                    "Give every picture its own "
                    Code { source: "alt" }
                    ". With "
                    Code { source: "zoom" }
                    " on, the picture showing is a tab stop that takes the zoom and pan keys. "
                    "Its description lists them, and a status message reads each new zoom "
                    "level. In the thumbnail strip, the arrows, "
                    Kbd { "Home" }
                    " and "
                    Kbd { "End" }
                    " move along the strip and change the picture with it. "
                    Kbd { "Esc" }
                    " closes the viewer."
                }
            }
        }
    }
}
