use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Avatar, AvatarGroup, AvatarSpec, Code, Input, Text};

static AVATAR_IMAGE: Asset = asset!("/assets/avatar.svg");

const MISSING_SRC: &str = "/does-not-exist.png";

/// The group demo's six people, printed verbatim by `Wrap`. One field per line: the code
/// block does not wrap.
// snippet: item const AVATAR_IMAGE: &str = "/ada.png";
// snippet: in AvatarGroup { .. }
const PEOPLE: &str = r#"people: vec![
    AvatarSpec {
        name: "Ada Lovelace".into(),
        src: Some(AVATAR_IMAGE.to_string()),
        ..Default::default()
    },
    AvatarSpec {
        name: "Grace Hopper".into(),
        initials: Some("GH".into()),
        ..Default::default()
    },
    AvatarSpec {
        name: "Katherine Johnson".into(),
        initials: Some("KJ".into()),
        color: Some("secondary".into()),
        ..Default::default()
    },
    AvatarSpec::from("Radia Perlman"),
    AvatarSpec::from("Barbara Liskov"),
    AvatarSpec::from("Margaret Hamilton"),
]"#;

fn people() -> Vec<AvatarSpec> {
    vec![
        AvatarSpec {
            name: "Ada Lovelace".into(),
            src: Some(AVATAR_IMAGE.to_string()),
            ..Default::default()
        },
        AvatarSpec {
            name: "Grace Hopper".into(),
            initials: Some("GH".into()),
            ..Default::default()
        },
        AvatarSpec {
            name: "Katherine Johnson".into(),
            initials: Some("KJ".into()),
            color: Some("secondary".into()),
            ..Default::default()
        },
        AvatarSpec::from("Radia Perlman"),
        AvatarSpec::from("Barbara Liskov"),
        AvatarSpec::from("Margaret Hamilton"),
    ]
}

/// In group mode, renames the block to `AvatarGroup` and splices in `people`, which
/// `generate_code` never prints.
fn wrap_group(values: &DemoValues, source: &str) -> String {
    if !grouped(values) {
        return source.to_string();
    }
    let source = source.replacen("Avatar {", "AvatarGroup {", 1);
    let people = indent(PEOPLE);
    match source.strip_suffix('}') {
        Some(open) if source.contains('\n') => format!("{open}{people}}}"),
        _ => format!("AvatarGroup {{\n{people}}}"),
    }
}

fn grouped(values: &DemoValues) -> bool {
    values.str("component") == "group"
}

/// One control over the fallback chain; a broken source shows the fallback happening.
/// Prints `name` too, which a group does not take.
fn content_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    let name = r#"name: "Ada Lovelace""#.to_string();
    match values.str("content").as_str() {
        "picture" => vec![name, "src: AVATAR_IMAGE".to_string()],
        "broken" => vec![
            name,
            format!("src: {MISSING_SRC:?}"),
            r#"initials: "AL""#.to_string(),
        ],
        "initials" => vec![name, r#"initials: "AL""#.to_string()],
        _ => vec![name],
    }
}

#[component]
pub fn AvatarPage() -> Element {
    rsx! {
        DocPage {
            title: "Avatar",
            source: "libero/src/components/data_display/avatar/avatar.rs",
            markdown: "/md/avatar.md",
            properties: vec![
                props("Avatar", vec![
                    prop("name", "String")
                        .default("required")
                        .doc("The person this avatar stands for, read as its accessible name."),
                    prop("src", "Option<String>")
                        .default("None")
                        .doc("The picture. Falls back to the rest of the chain when it fails to load."),
                    prop("initials", "Option<String>")
                        .default("None")
                        .doc("Drawn when there is no picture. Nothing is derived from `name`."),
                    prop("alt", "Option<String>")
                        .default("None")
                        .doc("Replaces the announced name. `alt: \"\"` marks the avatar decorative."),
                    prop("size", "Size")
                        .default("md")
                        .doc("The side of the square, which also sets the placeholder's font size."),
                    prop("radius", "Size")
                        .default("xxl")
                        .doc("A step on the avatar's own radius scale, `2px` to `32px`. The default `xxl` is a circle."),
                    prop("variant", "Variant")
                        .default("tonal")
                        .doc("The placeholder's look. Hidden once a picture loads."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The placeholder's tint."),
                    prop("children", "Option<Element>")
                        .default("None")
                        .doc("Anything in place of the initials, such as an icon."),
                ]),
                props("AvatarGroup", vec![
                    prop("people", "Vec<AvatarSpec>")
                        .default("required")
                        .doc("The members. The first is drawn on top."),
                    prop("max", "Option<usize>")
                        .default("None")
                        .doc("How many circles in total, the `+N` chip included, so the chip always stands for at least two people."),
                    prop("spacing", "Size")
                        .default("sm")
                        .doc("How far each circle overlaps the one before it."),
                    prop("size", "Size")
                        .default("md")
                        .doc("For every member and the chip."),
                    prop("radius", "Size")
                        .default("xxl")
                        .doc("For every member and the chip."),
                    prop("variant", "Variant")
                        .default("tonal")
                        .doc("For every member and the chip."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The tint of every member without a `color` of its own."),
                ]),
                props("AvatarSpec", vec![
                    prop("name", "String")
                        .default("required")
                        .doc("The accessible name, and what the chip lists."),
                    prop("src", "Option<String>")
                        .default("None")
                        .doc("The picture."),
                    prop("initials", "Option<String>")
                        .default("None")
                        .doc("Drawn when there is no picture."),
                    prop("color", "Option<ThemeAwareValue>")
                        .default("None")
                        .doc("This member's own tint."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "`name` is the accessible name, so a screen reader says \"Ada Lovelace\" rather than the initials.",
                    "A decorative avatar is hidden whole.",
                    "The group's `+N` chip is focusable, and its label lists the hidden names. Its words come from the `avatar` labels of the localization.",
                ])
                .must([
                    "Pass `alt: \"\"` where the name shows beside the avatar, or it is read twice.",
                    "Put nothing focusable in a decorative avatar.",
                ]),
            lead: rsx! {
                Text {
                    "A person as a fixed square. It shows the picture, else "
                    Code { source: "children" }
                    ", else "
                    Code { source: "initials" }
                    ", else a person glyph. A picture that fails to load falls back too. "
                    "Nothing is derived from "
                    Code { source: "name" }
                    ", since the letters that abbreviate a name depend on the script. The "
                    "avatar is not focusable. Wrap it in a "
                    Code { source: "Button" }
                    " or an "
                    Code { source: "Anchor" }
                    " to make it interactive."
                }
            },
            Demo {
                component: "Avatar",
                children_text: "",
                wrap: Wrap(wrap_group),
                controls: vec![
                    // Not a prop: one avatar, or a group of six. A group
                    // takes the same size and chrome for every member.
                    Control::toggle("component", ["avatar", "group"])
                        .labels(["Avatar", "AvatarGroup"])
                        .code(|_, _| vec![]),
                    Control::toggle("content", ["picture", "broken", "initials", "glyph"])
                        .labels(["Picture", "Broken src", "Initials", "Glyph"])
                        .hidden_when(grouped)
                        .code(content_code),
                    Control::slider("max", ["none", "2", "3", "4", "5", "6"])
                        .default("4")
                        .hidden_when(|values| !grouped(values))
                        .code(|_, values| match values.str("max").as_str() {
                            "none" => vec![],
                            value => vec![format!("max: {value}")],
                        }),
                    Control::slider("spacing", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm")
                        .hidden_when(|values| !grouped(values)),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("xxl"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"])
                    .default("tonal"),
                    // Unset `color` resolves to primary, so the default swatch is primary.
                    Control::color("color"),
                ],
                render: move |values: DemoValues| {
                    let content = values.str("content");

                    if grouped(&values) {
                        return rsx! {
                            AvatarGroup {
                                max: values.str("max").parse::<usize>().ok(),
                                spacing: values.str("spacing"),
                                size: values.str("size"),
                                radius: values.str("radius"),
                                variant: values.str("variant"),
                                color: Input::from(values.str("color")),
                                people: people(),
                            }
                        };
                    }
                    rsx! {
                        Avatar {
                            name: "Ada Lovelace",
                            src: match content.as_str() {
                                "picture" => Some(AVATAR_IMAGE.to_string()),
                                "broken" => Some(MISSING_SRC.to_string()),
                                _ => None,
                            },
                            initials: (content == "initials" || content == "broken")
                                .then(|| "AL".to_string()),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            variant: values.str("variant"),
                            color: Input::from(values.str("color")),
                        }
                    }
                },
            }
        }
    }
}
