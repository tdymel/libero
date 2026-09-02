use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Avatar, AvatarGroup, AvatarSpec, Code, Input, Text};

static AVATAR_IMAGE: Asset = asset!("/assets/avatar.svg");

const MISSING_SRC: &str = "/does-not-exist.png";

/// The six people the group demo holds, printed verbatim by `Wrap` - `people`
/// is not a control, and it is most of what a reader needs. One field per
/// line: the code block does not wrap, and a one-line struct literal runs off
/// the right of it.
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

/// `generate_code` never prints `people`, so the generated block is reopened
/// and the whole `vec![..]` spliced in - the snippet is then the one that
/// produced the preview.
fn wrap_group(_values: &DemoValues, source: &str) -> String {
    let people = indent(PEOPLE);
    match source.strip_suffix('}') {
        Some(open) if source.contains('\n') => format!("{open}{people}}}"),
        _ => format!("AvatarGroup {{\n{people}}}"),
    }
}

/// One control over the whole fallback chain: every step of it is a different
/// thing on screen, and a broken source is the only way to show that the
/// picture falls back rather than merely being absent.
fn content_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match values.str("content").as_str() {
        "picture" => vec!["src: AVATAR_IMAGE".to_string()],
        "broken" => vec![
            format!("src: {MISSING_SRC:?}"),
            r#"initials: "AL""#.to_string(),
        ],
        "initials" => vec![r#"initials: "AL""#.to_string()],
        _ => vec![],
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
                        .doc("The person this avatar stands for, announced as its accessible name."),
                    prop("src", "String")
                        .doc("The picture. Falls through to the rest of the chain once it fails to load."),
                    prop("initials", "String")
                        .doc("Drawn when there is no picture. Nothing is derived from `name`."),
                    prop("alt", "String")
                        .default("follows name")
                        .doc("Overrides the announced name. `alt: \"\"` marks the avatar decorative."),
                    prop("size", "Size")
                        .default("theme.avatar.size")
                        .doc("The square's side, which also sets the placeholder's font size."),
                    prop("radius", "ThemeAwareValue")
                        .default("theme.avatar.radius")
                        .doc("Corner radius - the radius scale, or any CSS length. The default is a circle."),
                    prop("variant", "ButtonVariant")
                        .default("tonal")
                        .doc("Placeholder chrome; invisible once a picture loads."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Placeholder tint."),
                    prop("children", "Element")
                        .doc("Anything at all in place of the initials - an icon, a glyph."),
                ]),
                props("AvatarGroup", vec![
                    prop("people", "Vec<AvatarSpec>")
                        .default("required")
                        .doc("The members, in paint order: the first is drawn on top."),
                    prop("max", "usize")
                        .doc("How many circles in total. Past that, the rest collapse into a `+N` chip."),
                    prop("spacing", "Size")
                        .default("theme.avatar_group.spacing")
                        .doc("How far each circle is pulled over the one before it."),
                    prop("size", "Size")
                        .default("theme.avatar.size")
                        .doc("Applied to every member, the chip included."),
                    prop("radius", "ThemeAwareValue")
                        .default("theme.avatar.radius")
                        .doc("Applied to every member, the chip included."),
                    prop("variant", "ButtonVariant")
                        .default("tonal")
                        .doc("Applied to every member, the chip included."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The tint a member without a `color` of its own takes."),
                ]),
                props("AvatarSpec", vec![
                    prop("name", "String")
                        .default("required")
                        .doc("The accessible name, and what the overflow chip lists."),
                    prop("src", "Option<String>")
                        .doc("The picture."),
                    prop("initials", "Option<String>")
                        .doc("Drawn when there is no picture."),
                    prop("color", "Option<ThemeAwareValue>")
                        .doc("This member's own tint, overriding the group's."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A person as a fixed square, with a fallback chain: the picture, then "
                    Code { source: "children" }
                    ", then "
                    Code { source: "initials" }
                    ", then a person glyph. Nothing is derived from "
                    Code { source: "name" }
                    " - an initial is a first grapheme cluster rather than a first character, "
                    "and which one abbreviates a name is a property of the script, so the "
                    "caller supplies it. The root is a "
                    Code { source: "span role=\"img\"" }
                    " carrying "
                    Code { source: "name" }
                    ", so a screen reader announces \"Ada Lovelace\" instead of spelling out "
                    "the two letters; "
                    Code { source: "alt: \"\"" }
                    " marks it decorative, for the common case of an avatar sitting beside the "
                    "person's visible name. It is not focusable and not interactive - wrap it "
                    "in a "
                    Code { source: "Button" }
                    " or an "
                    Code { source: "Anchor" }
                    " if it should be either."
                }
            },
            Demo {
                component: "Avatar",
                children_text: "",
                fixed: vec![r#"name: "Ada Lovelace""#.to_string()],
                controls: vec![
                    Control::toggle("content", ["picture", "broken", "initials", "glyph"])
                        .labels(["Picture", "Broken src", "Initials", "Glyph"])
                        .code(content_code),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "9999px"])
                        .default("9999px"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .default("tonal"),
                    // An unset `color` is `base_color`'s primary shade 6,
                    // which is exactly what a bare `primary` resolves to - so
                    // the default swatch is primary, not a white "unset" one.
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning"],
                    ),
                ],
                render: move |values: DemoValues| {
                    let content = values.str("content");

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
            DocSection {
                title: "Groups",
                Text {
                    Code { source: "AvatarGroup" }
                    " owns its members rather than taking them as children, which is what lets "
                    "it count them. "
                    Code { source: "max" }
                    " is the number of circles, the chip included, so the chip always stands "
                    "for at least two people. It is focusable and tooltipped, and its "
                    Code { source: "aria-label" }
                    " lists the same names, so a tooltip clipped by an "
                    Code { source: "overflow: hidden" }
                    " ancestor costs nothing but the hover."
                }
                Demo {
                    component: "AvatarGroup",
                    children_text: "",
                    controls: vec![
                        Control::slider("max", ["none", "2", "3", "4", "5", "6"])
                            .default("4")
                            .code(|_, values| match values.str("max").as_str() {
                                "none" => vec![],
                                value => vec![format!("max: {value}")],
                            }),
                        Control::slider("spacing", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("sm"),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                    ],
                    wrap: Wrap(wrap_group),
                    render: move |values: DemoValues| {
                        rsx! {
                            AvatarGroup {
                                max: values.str("max").parse::<usize>().ok(),
                                spacing: values.str("spacing"),
                                size: values.str("size"),
                                people: people(),
                            }
                        }
                    },
                }
            }
        }
    }
}
