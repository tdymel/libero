use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset};
use dioxus::prelude::*;
use libero::{
    components::{Code, Image, Text},
    sx::sx,
    use_theme,
};

/// `alt` is required, and the box has to be sized before `fit` means
/// anything - neither is a control, so both print as `fixed`.
const FIXED: [&str; 2] = [
    r#"alt: "A stylised landscape""#,
    r#"sx: sx().width("160px").height("160px").background("grey.1")"#,
];

const MISSING_SRC: &str = "/does-not-exist.png";

#[component]
pub fn ImagePage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Image",
            lead: rsx! {
                Text {
                    "An "
                    Code { source: "img" }
                    " with a fallback source on load error, optional rounded corners, and an "
                    "optional click-to-zoom overlay. "
                    Code { source: "fit" }
                    " maps straight onto "
                    Code { source: "object-fit" }
                    "; "
                    Code { source: "radius" }
                    " takes the radius scale or any CSS length."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Image",
                    children_text: "",
                    fixed: FIXED.map(str::to_string).to_vec(),
                    controls: vec![
                        Control::select(
                            "fit",
                            ["fill", "contain", "cover", "none", "scale-down"],
                        )
                        .default(theme.image.fit.as_str()),
                        Control::slider("radius", ["auto", "xs", "sm", "md", "lg", "xl", "50%"]),
                        Control::switch("zoomable"),
                        // Drives `src` too: a fallback only shows once the
                        // real source fails, so the switch has to break it.
                        Control::switch("broken_src").code(|_, values| {
                            let src = match values.str("broken_src").as_str() {
                                "true" => format!("{MISSING_SRC:?}"),
                                _ => "SAMPLE_IMAGE".to_string(),
                            };
                            vec![
                                format!("src: {src}"),
                                "fallback_src: FALLBACK_IMAGE".to_string(),
                            ]
                        }),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Image {
                            src: match values.str("broken_src").as_str() {
                                "true" => MISSING_SRC.to_string(),
                                _ => crate::SAMPLE_IMAGE.to_string(),
                            },
                            fallback_src: crate::FALLBACK_IMAGE.to_string(),
                            alt: "A stylised landscape",
                            fit: values.str("fit"),
                            radius: or_unset(values.str("radius")),
                            zoomable: values.str("zoomable") == "true",
                            sx: sx().width("160px").height("160px").background("grey.1"),
                        }
                    },
                }
            }
        }
    }
}
