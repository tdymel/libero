use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Dialog, Flex, FloatingWindowOptions, Input, Kbd, Text, WindowRect},
    hooks::{ModalScope, use_floating_window, use_modal},
    sx::sx,
};

/// The three windows the preview opens: the name its handle has in the
/// printed code, its title, where it first appears, and its body.
const WINDOWS: [(&str, &str, &str, &str); 3] = [
    (
        "inspector",
        "Inspector",
        "bottom-end",
        r#"Text { "Drag the title bar, or focus it and use the arrow keys." }
Button { onclick: move |_| window.close(), "Done" }"#,
    ),
    (
        "notes",
        "Notes",
        "top-end",
        r#"Text { "A modal opened from here covers every window." }
Button { onclick: move |_| { confirm.open(); }, "Open a modal" }"#,
    ),
    (
        "layers",
        "Layers",
        "bottom-start",
        r#"Text { "Click a window to bring it to the front." }
Button { onclick: move |_| window.close(), "Done" }"#,
    ),
];

const CONFIRM: &str = r#"let confirm = use_modal(|s: ModalScope<()>| rsx! {
    Dialog { title: "A modal",
        Text { "Modals and their overlay sit above every floating window." }
        Button { onclick: move |_| s.close(), "Close" }
    }
});
"#;

const READOUT: &str = r#"if let Some(rect) = last() {
    Text {
        {format!("Last reported: {:.0}, {:.0} - {:.0} x {:.0}", rect.x, rect.y, rect.width, rect.height)}
    }
}"#;

/// One `use_floating_window` call, with the options the controls set.
fn hook_code(
    values: &DemoValues,
    (handle, title, placement, body): (&str, &str, &str, &str),
) -> String {
    let mut options = vec![
        format!("title: Some({title:?}.into()),"),
        format!("placement: {placement:?}.into(),"),
    ];
    if values.str("resizable") == "true" {
        options.push("resizable: true,".into());
        options.push(
            r#"sx: sx().min_width("16rem").min_height("8rem").max_width("40rem").into(),"#.into(),
        );
    }
    if values.str("pinned") == "true" {
        options.push("pinned: true,".into());
    }
    if values.str("onmove") == "true" {
        options.push("onmove: Some(report),".into());
        options.push("onresize: Some(report),".into());
    }
    options.push("..Default::default()".into());
    // Notes opens the modal and never closes itself, so it ignores its handle.
    let param = if handle == "notes" {
        "move |_|"
    } else {
        "|window|"
    };
    format!(
        "let {handle} = use_floating_window(\n    FloatingWindowOptions {{\n{}    }},\n    {param} rsx! {{\n{}    }},\n);\n",
        indent(&indent(&options.join("\n"))),
        indent(&indent(body)),
    )
}

/// The code of the example opened last: its hook, what the hook's body needs,
/// and its trigger. The preview's buttons pick the example.
fn wrap_example(values: &DemoValues, _: &str) -> String {
    let example = values.str("example");
    let shown: Vec<_> = WINDOWS
        .into_iter()
        .filter(|(handle, ..)| example == "all" || example == *handle)
        .collect();

    let mut code = String::new();
    if values.str("onmove") == "true" {
        code.push_str("let mut last = use_signal(|| None::<WindowRect>);\n");
        code.push_str("let report = use_callback(move |rect: WindowRect| last.set(Some(rect)));\n");
    }
    if shown.iter().any(|(handle, ..)| *handle == "notes") {
        code.push_str(CONFIRM);
    }
    for window in &shown {
        code.push_str(&hook_code(values, *window));
    }

    let trigger = match shown.as_slice() {
        [(handle, title, ..)] => {
            format!("Button {{ onclick: move |_| {handle}.toggle(), {title:?} }}")
        }
        _ => "Button {\n    onclick: move |_| {\n        inspector.open();\n        notes.open();\n        layers.open();\n    },\n    \"Open all three\"\n}".to_string(),
    };
    let readout = if values.str("onmove") == "true" {
        indent(READOUT)
    } else {
        String::new()
    };
    format!("{code}\nrsx! {{\n{}{readout}}}", indent(&trigger))
}

/// The hooks need a scope of their own: `Demo` calls `render` from its own.
#[component]
fn WindowDemo(resizable: bool, pinned: bool, report: bool, values: DemoValues) -> Element {
    let mut last = use_signal(|| None::<WindowRect>);
    let record = use_callback(move |rect: WindowRect| last.set(Some(rect)));
    let options = move |title: &str, placement: &str| FloatingWindowOptions {
        title: Some(title.into()),
        placement: placement.into(),
        resizable,
        pinned,
        sx: if resizable {
            sx().min_width("16rem")
                .min_height("8rem")
                .max_width("40rem")
                .into()
        } else {
            Input::None
        },
        onmove: report.then_some(record),
        onresize: report.then_some(record),
        ..Default::default()
    };

    let confirm = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "A modal",
                Text { "Modals and their overlay sit above every floating window." }
                Button { onclick: move |_| s.close(), "Close" }
            }
        }
    });
    let inspector = use_floating_window(options("Inspector", "bottom-end"), |window| {
        rsx! {
            Text { "Drag the title bar, or focus it and use the arrow keys." }
            Button { onclick: move |_| window.close(), "Done" }
        }
    });
    let notes = use_floating_window(options("Notes", "top-end"), move |_| {
        rsx! {
            Text { "A modal opened from here covers every window." }
            Button {
                onclick: move |_| {
                    confirm.open();
                },
                "Open a modal"
            }
        }
    });
    let layers = use_floating_window(options("Layers", "bottom-start"), |window| {
        rsx! {
            Text { "Click a window to bring it to the front." }
            Button { onclick: move |_| window.close(), "Done" }
        }
    });

    // Every button also picks the example the code block prints.
    let show = move |example: &'static str| {
        let values = values.clone();
        move || values.set("example", example)
    };

    rsx! {
        Flex { direction: "column", align: "center", gap: "md",
            Flex {
                direction: "row",
                justify: "center",
                gap: "sm",
                wrap: "wrap",
                Button {
                    variant: "outlined",
                    onclick: {
                        let show = show("inspector");
                        move |_| {
                            show();
                            inspector.toggle();
                        }
                    },
                    "Inspector"
                }
                Button {
                    variant: "outlined",
                    onclick: {
                        let show = show("notes");
                        move |_| {
                            show();
                            notes.toggle();
                        }
                    },
                    "Notes"
                }
                Button {
                    variant: "outlined",
                    onclick: {
                        let show = show("layers");
                        move |_| {
                            show();
                            layers.toggle();
                        }
                    },
                    "Layers"
                }
                Button {
                    variant: "outlined",
                    onclick: {
                        let show = show("all");
                        move |_| {
                            show();
                            inspector.open();
                            notes.open();
                            layers.open();
                        }
                    },
                    "Open all three"
                }
            }
            if report {
                if let Some(rect) = last() {
                    Text {
                        {
                            format!(
                                "Last reported: {:.0}, {:.0} - {:.0} x {:.0}",
                                rect.x,
                                rect.y,
                                rect.width,
                                rect.height,
                            )
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn FloatingWindowPage() -> Element {
    rsx! {
        DocPage {
            title: "Floating window",
            source: "libero/src/hooks/floating_window.rs",
            markdown: "/md/floating_window.md",
            properties: vec![
                props("FloatingWindowOptions", vec![
                    prop("title", "Option<String>").doc("The title bar's heading, and the window's accessible name."),
                    prop("aria_label", "Option<String>").doc("Names the window instead of `title`."),
                    prop("placement", "Input<Placement>").default("center-center").doc("Where it first appears, from `theme.floating_window.placement`. Once dragged it stays where it was put, clamped into the viewport."),
                    prop("resizable", "bool").default("false").doc("Draws the corner resize handle."),
                    prop("pinned", "bool").default("false").doc("No drag and no keyboard move."),
                    prop("z_index", "Input<ThemeAwareValue>").doc("Overrides the stacking. Unset, windows stack from `theme.z_index.window`, below overlays and modals."),
                    prop("sx", "Input<Sx>").doc("On the window. `min_width`/`max_width`/`min_height`/`max_height` here clamp a resize. The viewport cap always applies on top, so a `max_width` of `40rem` is still no wider than a phone."),
                    prop("onmove", "Option<Callback<WindowRect>>").doc("After a drag or a keyboard move, in viewport pixels."),
                    prop("onresize", "Option<Callback<WindowRect>>").doc("After a resize, by pointer or keyboard."),
                ]),
                props("FloatingWindowHandle", vec![
                    prop("open()", "()").doc("Shows it and remembers where focus was. Does nothing when open."),
                    prop("close()", "()").doc("Hides it and returns focus to whatever opened it."),
                    prop("toggle()", "()").doc("One or the other."),
                    prop("is_open()", "bool").doc("A subscribed read."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A non-modal window over the page: a title bar that drags, an optional corner "
                    "resize handle, and a close button. "
                    Code { source: "use_floating_window" }
                    " owns whether it exists and hands back a "
                    Code { source: "Copy" }
                    " handle; the window owns where it is and how big. No overlay and no focus "
                    "trap, so the page stays usable. Windows sit on the viewport, not in this "
                    "preview: the buttons open them over the whole page, and the code block "
                    "follows the last one you pressed."
                }
            },
            Demo {
                component: "FloatingWindowOptions",
                children_text: "",
                controls: vec![
                    // Set by the preview's buttons, never by the panel.
                    Control::toggle("example", ["inspector", "notes", "layers", "all"])
                        .hidden_when(|_| true),
                    Control::switch("resizable"),
                    Control::switch("pinned"),
                    Control::switch("onmove"),
                ],
                render: move |values: DemoValues| rsx! {
                    WindowDemo {
                        resizable: values.str("resizable") == "true",
                        pinned: values.str("pinned") == "true",
                        report: values.str("onmove") == "true",
                        values: values.clone(),
                    }
                },
                wrap: Wrap(wrap_example),
            }
            DocSection {
                title: "Keyboard",
                Text {
                    "A window takes focus when it opens. " Kbd { "Tab" }
                    " reaches the title bar, where " Kbd { "←" } " " Kbd { "↑" } " "
                    Kbd { "→" } " " Kbd { "↓" } " move it 10px and "
                    Kbd { "Shift" } " + arrow 1px. On the resize handle the arrows resize, and "
                    Kbd { "Home" } " " Kbd { "End" }
                    " ask for the smallest and largest size allowed. " Kbd { "Esc" }
                    " closes the window and returns focus to its trigger."
                }
            }
        }
    }
}
