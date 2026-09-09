//! Fixture app for the E2E suite.
//!
//! One route per fixture, each rendering a single component in the smallest
//! realistic consumer that exercises it. These are **not** the docs pages: a
//! docs page carries `Demo` controls, a code block printing the same strings
//! the preview renders, and prose that changes for reasons unrelated to the
//! component. Driving those produced fourteen false failures on 2026-09-14
//! (see `codebase/testing`), so the suite drives this instead.
//!
//! Two rules every fixture follows:
//!
//! * **Size to the content, never to the viewport.** A fixture that assumes a
//!   desktop width produces measurements about the fixture rather than about
//!   the component - todo 296 is that mistake made in the docs, where a 320px
//!   minimum in a 308px box shows a scrollbar at rest.
//! * **Carry the ready marker.** `dx` answers every path with a 404 placeholder
//!   *at a success status* while its first build runs, so a page that loaded
//!   proves nothing. `data-fixture-ready` is on an element only the real app
//!   renders, and the harness waits for it.

mod broken;
mod combobox;
mod overlay;
mod picker_dialog;
mod roving;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Anchor, Autocomplete, Button, Carousel, Code, CodeBlock, Collapse, Dialog, Flex, Mark,
        NotificationData, NotificationLive, NotificationOptions, NotificationScope, Notifications,
        OptionList, Options, Paper, Slider, SliderChangeEvent, Splitter, Tabs, Text, TextField,
        Tooltip, Tree, TreeItem, TreeNode, TreeNodeRenderArgs, use_notifications,
        use_notifications_with,
    },
    hooks::{ModalScope, use_modal},
    theme::AutoClose,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[layout(Fixture)]
    #[route("/")]
    Index {},
    #[route("/autocomplete")]
    AutocompletePage {},
    #[route("/slider")]
    SliderPage {},
    #[route("/tabs")]
    TabsPage {},
    #[route("/tabs-disabled-selected")]
    TabsDisabledSelectedPage {},
    #[route("/carousel")]
    CarouselPage {},
    #[route("/modal")]
    ModalPage {},
    #[route("/focus-contrast")]
    FocusContrastPage {},
    #[route("/mark")]
    MarkPage {},
    #[route("/tree")]
    TreePage {},
    #[route("/code")]
    CodePage {},
    #[route("/radio-group")]
    RadioGroupPage {},
    #[route("/segmented-control")]
    SegmentedControlPage {},
    #[route("/menubar")]
    MenubarPage {},
    #[route("/select")]
    SelectPage {},
    #[route("/multi-select")]
    MultiSelectPage {},
    #[route("/tags-field")]
    TagsFieldPage {},
    #[route("/drawer")]
    DrawerPage {},
    #[route("/menu")]
    MenuPage {},
    #[route("/menu-open-on-mount")]
    MenuOpenOnMountPage {},
    #[route("/spotlight")]
    SpotlightPage {},
    #[route("/collapse")]
    CollapsePage {},
    #[route("/lightbox")]
    LightboxPage {},
    #[route("/floating-window")]
    FloatingWindowPage {},
    #[route("/tooltip")]
    TooltipPage {},
    #[route("/tooltip-wrapped")]
    TooltipWrappedPage {},
    #[route("/button")]
    ButtonPage {},
    #[route("/button/landing")]
    ButtonLanding {},
    #[route("/color-field")]
    ColorFieldPage {},
    #[route("/date-field")]
    DateFieldPage {},

    // A real component with one defect planted through a prop. Most plants
    // are injected by the test instead (`tests/all/planted.rs`); these are
    // the ones only a prop can make.
    #[route("/planted/segmented-control-readonly")]
    PlantedSegmentedReadonly {},

    // Deliberately broken, one per pass. See `broken.rs`: these exist so the
    // suite can prove each pass is able to fail.
    #[route("/broken/focus-ring")]
    BrokenFocusRing {},
    #[route("/broken/target-size")]
    BrokenTargetSize {},
    #[route("/broken/target-spacing")]
    BrokenTargetSpacing {},
    #[route("/broken/contrast")]
    BrokenContrast {},
    #[route("/broken/console")]
    BrokenConsole {},
    #[route("/broken/faint-field-ring")]
    BrokenFaintFieldRing {},
    #[route("/broken/console-warning")]
    BrokenConsoleWarning {},
    #[route("/broken/roving")]
    BrokenRoving {},
    #[route("/broken/focus-return")]
    BrokenFocusReturn {},
    #[route("/broken/dismissal")]
    BrokenDismissal {},
    #[route("/broken/activedescendant")]
    BrokenActiveDescendant {},
    #[route("/broken/activedescendant-missing")]
    BrokenActiveDescendantMissing {},
    #[route("/broken/static-highlight")]
    BrokenStaticHighlight {},
    #[route("/notifications")]
    NotificationsPage {},
    #[route("/notifications-clear")]
    NotificationsClearPage {},
    #[route("/splitter")]
    SplitterPage {},
}

/// The component the framework was **not** built for.
///
/// Everything in `Suite` assumes a static page: open a route, measure it,
/// snapshot it. A notification is transient - it appears on an action and
/// removes itself on a timer - so its accessibility tree is a function of time,
/// and its live regions say different things at different moments.
///
/// `#notify` and `#notify-assertive` pin `auto_close` off. A fixture that
/// disappears while being measured is not a test of the component, it is a
/// race. `#notify-timed` does close itself, after [`TIMED_AUTO_CLOSE_MS`], and
/// the test holds that timer on the page's clock and fires it by hand
/// (`tests/all/notifications.rs`), so nothing waits on real time.
#[component]
fn NotificationsPage() -> Element {
    let notify = use_notifications();

    rsx! {
        Notifications {}
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "notify",
                onclick: move |_| {
                    notify
                        .show_with(
                            "Saved to your library",
                            NotificationOptions {
                                auto_close: Some(AutoClose::Never),
                                ..Default::default()
                            },
                        );
                },
                "Notify"
            }
            Button {
                id: "notify-assertive",
                onclick: move |_| {
                    notify
                        .show_with(
                            NotificationData {
                                title: Some("Upload failed".into()),
                                message: "The file is larger than 10 MB".into(),
                                ..Default::default()
                            },
                            NotificationOptions {
                                auto_close: Some(AutoClose::Never),
                                live: NotificationLive::Assertive,
                                ..Default::default()
                            },
                        );
                },
                "Notify assertively"
            }
            Button {
                id: "notify-timed",
                onclick: move |_| {
                    notify
                        .show_with(
                            "Draft saved",
                            NotificationOptions {
                                auto_close: Some(AutoClose::After(TIMED_AUTO_CLOSE_MS)),
                                ..Default::default()
                            },
                        );
                },
                "Notify for a while"
            }
        }
    }
}

/// Two notifications, each with a "Clear all" button, so `clear()` runs with
/// focus inside one (todo 440). Its own route, so `/notifications`' baselines
/// stay as they are.
#[component]
fn NotificationsClearPage() -> Element {
    let notify = use_notifications_with(|s: NotificationScope<String>| {
        let all = use_notifications();
        rsx! {
            Paper {
                Text { "{s.args()}" }
                Button { class: "clear-all", onclick: move |_| all.clear(), "Clear all" }
            }
        }
    });

    rsx! {
        Notifications {}
        Button {
            id: "notify",
            onclick: move |_| {
                for message in ["First", "Second"] {
                    notify
                        .show_with(
                            message,
                            NotificationOptions {
                                auto_close: Some(AutoClose::Never),
                                ..Default::default()
                            },
                        );
                }
            },
            "Notify twice"
        }
    }
}

/// A delay nothing else on the page schedules, so the test's clock can hold
/// exactly this timer and pass every other one through.
const TIMED_AUTO_CLOSE_MS: u32 = 4321;

use combobox::{MultiSelectPage, SelectPage, TagsFieldPage};
use overlay::{
    DrawerPage, FloatingWindowPage, LightboxPage, MenuOpenOnMountPage, MenuPage, SpotlightPage,
};
use picker_dialog::{ColorFieldPage, DateFieldPage};
use roving::{MenubarPage, RadioGroupPage, SegmentedControlPage};

#[component]
fn PlantedSegmentedReadonly() -> Element {
    rsx! { SegmentedControlPage { readonly: true } }
}

#[component]
fn BrokenFocusRing() -> Element {
    rsx! { broken::NoFocusRing {} }
}
#[component]
fn BrokenTargetSize() -> Element {
    rsx! { broken::TinyTarget {} }
}
#[component]
fn BrokenTargetSpacing() -> Element {
    rsx! { broken::CrampedTargets {} }
}
#[component]
fn BrokenContrast() -> Element {
    rsx! { broken::LowContrast {} }
}
#[component]
fn BrokenConsole() -> Element {
    rsx! { broken::ConsoleError {} }
}
#[component]
fn BrokenFaintFieldRing() -> Element {
    rsx! { broken::FaintFieldRing {} }
}
#[component]
fn BrokenConsoleWarning() -> Element {
    rsx! { broken::ConsoleWarning {} }
}
#[component]
fn BrokenRoving() -> Element {
    rsx! { broken::ManyTabStops {} }
}
#[component]
fn BrokenFocusReturn() -> Element {
    rsx! { broken::NoFocusReturn {} }
}
#[component]
fn BrokenDismissal() -> Element {
    rsx! { broken::PhantomPanel {} }
}
#[component]
fn BrokenActiveDescendant() -> Element {
    rsx! { broken::DanglingActiveDescendant {} }
}
#[component]
fn BrokenActiveDescendantMissing() -> Element {
    rsx! { broken::MissingActiveDescendant {} }
}
#[component]
fn BrokenStaticHighlight() -> Element {
    rsx! { broken::StaticHighlight {} }
}

/// Wraps every fixture in the provider and the ready marker, and nothing else.
/// No shell, no navigation, no styling of its own - anything here would be
/// measured as though it were the component.
///
/// The marker goes **outside** the provider. `Suite` scopes axe and the
/// accessibility snapshot to it, and the provider renders its portal outlet
/// beside its children - so with the marker inside, every dialog, menu and
/// listbox was outside the scope. axe never saw one (a listbox planted with
/// `#ddd` text on white passed), and the modal's "open" baseline recorded the
/// trigger alone. Found 2026-09-19 by Olaf95.
#[component]
fn Fixture() -> Element {
    rsx! {
        div { "data-fixture-ready": "true",
            LiberoProvider {
                div { padding: "24px", Outlet::<Route> {} }
            }
        }
    }
}

#[component]
fn Index() -> Element {
    rsx! { "libero e2e fixtures" }
}

const CITIES: &[&str] = &[
    "Amsterdam",
    "Berlin",
    "Copenhagen",
    "Dublin",
    "Edinburgh",
    "Florence",
];

/// The combobox archetype, fully assembled.
///
/// `Autocomplete` rather than `Combobox` itself: `Combobox` is a wrapper whose
/// caller supplies the trigger and its aria, so a fixture built on it would be
/// testing the fixture's own wiring as much as the library's. `Autocomplete`
/// owns the whole `aria-activedescendant` contract, which is what the pass is
/// about.
#[component]
fn AutocompletePage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Autocomplete {
                label: "City",
                options: CITIES.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
}

/// The one fixture that exists for the pointer pass. Nothing else in the suite
/// reaches a drag, and the thumb is also the sharpest case for target size.
#[component]
fn SliderPage() -> Element {
    let mut volume = use_signal(|| 40.0f64);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Slider {
                aria_label: "Volume",
                value: volume(),
                min: 0.0f64,
                max: 100.0f64,
                oninput: move |e: SliderChangeEvent<f64>| volume.set(e.value()),
            }
        }
    }
}

/// A `Splitter` between two buttons, so focus has somewhere to be before a
/// drag and a Shift+Tab never sits at the document edge.
#[component]
fn SplitterPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            div { style: "height: 160px",
                Splitter {
                    initial_size: 50.0,
                    aria_label: "Resize panes",
                    panel_a: rsx! { Text { "Pane A" } },
                    panel_b: rsx! { Text { "Pane B" } },
                }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A `Tooltip` on a direct-child trigger, between two buttons so Tab has
/// somewhere to come from and to go to. `bottom`, so the test knows where the
/// gap it bridges lies.
#[component]
fn TooltipPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            Tooltip { label: rsx! { "Saves the draft" }, label_id: "save-tip", side: "bottom",
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
            Button { id: "after", "After" }
        }
    }
}

/// The trigger one element too deep: hover shows the bubble, Tab does not,
/// and a debug build says so in the console.
#[component]
fn TooltipWrappedPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            Tooltip { label: rsx! { "Saves the draft" }, label_id: "save-tip", side: "bottom",
                div {
                    Button { id: "save", "aria-describedby": "save-tip", "Save" }
                }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A plain button and a busy one, each counting its activations, and a
/// link-mode button whose route is one only this page leads to.
#[component]
fn ButtonPage() -> Element {
    let mut plain = use_signal(|| 0u32);
    let mut busy = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "plain", onclick: move |_| plain += 1, "Plain" }
            Button { id: "busy", loading: true, onclick: move |_| busy += 1, "Busy" }
            Text { id: "presses", "data-plain": "{plain}", "data-busy": "{busy}",
                "Plain {plain}, busy {busy}"
            }
            Button { id: "to-landing", to: NavigationTarget::from(Route::ButtonLanding {}), "Go to landing" }
        }
    }
}

#[component]
fn ButtonLanding() -> Element {
    rsx! { Text { id: "landing", "Landed" } }
}

/// The enum is the tab strip, so it is the fixture's whole configuration.
#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Billing,
    Admin,
}

/// The motion fixture: a `Collapse` whose open and close both run a
/// transition, so a reduced-motion test has something to switch off.
/// `keep_mounted: false`, so closing also exercises the unmount that
/// `use_presence` ties to the exit.
#[component]
fn CollapsePage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "toggle-details",
                variant: "outlined",
                onclick: move |_| open.toggle(),
                "Shipping details"
            }
            Collapse { id: "details", open: open(), keep_mounted: false,
                Text { id: "details-text", "Shipping is calculated at checkout." }
            }
        }
    }
}

/// The roving-tabindex archetype: one tab stop, arrows inside.
#[component]
fn TabsPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                value: section(),
                onchange: move |next| section.set(next),
                panel: |s: Section| rsx! {
                    Text { "panel for {s.label()}" }
                },
            }
        }
    }
}

/// Todo 403: the selected tab is disabled, which a controlled `value` allows.
/// The arrows must still part ways from it.
#[component]
fn TabsDisabledSelectedPage() -> Element {
    let mut section = use_signal(|| Section::Billing);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                value: section(),
                onchange: move |next| section.set(next),
                options: OptionList::from_options().disabling(|s| *s == Section::Billing),
                panel: |s: Section| rsx! {
                    Text { "panel for {s.label()}" }
                },
            }
        }
    }
}

/// The fixture for the one thing a carousel cannot get from structure: a key
/// pressed **inside** a slide.
///
/// `Carousel`'s track handler acts on the arrows, Home and End, and
/// `prevent_default()`s them. Its slides hold whatever a caller puts there,
/// which is code the component cannot ask to stop propagating, so the press
/// arrives at the track whatever it landed on. Three guard arms separate the
/// two cases (`carousel.rs`, `key_taken || typing_target || arrow_target`),
/// and the first slide holds one control per arm:
///
/// * a `TextField` and a `Slider` - the library's own controls. The slider
///   takes the arrows and marks them by preventing their default, which is
///   `key_taken`; the text field is left to the browser, which is
///   `typing_target`.
/// * a raw `<input type="range">` and a raw radio pair - HTML a caller wrote,
///   which nothing in the library marks and which the browser steps with the
///   arrows anyway. That is `arrow_target`.
/// * a `Button`, which **no** arm covers. It is the positive control: arrows
///   pressed on it have to move the strip, or "the strip did not move" is a
///   sentence about a carousel that never moves.
///
/// The rest of the slides are plain text. Only the resting slide is out of
/// `inert`, so a control on another one could not be focused to press a key
/// in it.
#[component]
fn CarouselPage() -> Element {
    let mut note = use_signal(|| "carousel".to_string());
    let mut volume = use_signal(|| 40.0f64);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Slide content",
                // Not the theme default (`false`). The dots are a carousel's
                // likeliest WCAG 2.5.8 failure and the `Suite` baseline cannot
                // measure a control that is not drawn (todo 381).
                indicators: true,
                slides: vec![
                    rsx! {
                        Flex { direction: "column", gap: "sm",
                            TextField {
                                label: "Note",
                                value: note(),
                                oninput: move |next| note.set(next),
                            }
                            Slider {
                                aria_label: "Volume",
                                value: volume(),
                                min: 0.0f64,
                                max: 100.0f64,
                                oninput: move |e: SliderChangeEvent<f64>| volume.set(e.value()),
                            }
                            input {
                                id: "raw-range",
                                r#type: "range",
                                min: "0",
                                max: "100",
                                step: "1",
                                value: "50",
                                "aria-label": "Raw range",
                            }
                            div { role: "radiogroup", "aria-label": "Raw choice",
                                label {
                                    input {
                                        id: "raw-radio-a",
                                        r#type: "radio",
                                        name: "raw-choice",
                                        checked: true,
                                    }
                                    " A"
                                }
                                label {
                                    input { id: "raw-radio-b", r#type: "radio", name: "raw-choice" }
                                    " B"
                                }
                            }
                            Button { id: "slide-button", variant: "outlined", "Plain button" }
                        }
                    },
                    rsx! {
                        Text { "The second slide, and nothing to press on it." }
                    },
                    rsx! {
                        Text { "The third slide, and nothing to press on it." }
                    },
                ],
            }
        }
    }
}

/// The overlay archetype: opens, traps focus, Escape closes, focus returns.
#[component]
fn ModalPage() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Text { "notes.md has changes you have not saved." }
                Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                Button { variant: "filled", onclick: move |_| s.close(), "Discard" }
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            // A stable hook of the fixture's own, rather than leaving the
            // test to guess at `button`: once the dialog is open there are
            // three buttons on the page and `querySelector` would pick whichever
            // came first.
            Button {
                id: "open-modal",
                onclick: move |_| {
                    prompt.open();
                },
                "Close editor"
            }
        }
    }
}

/// Todo 53, part one: what a `--lsx-focus-contrast` naming an undeclared
/// referent does to the ring.
///
/// The ring resolves as `var(--lsx-focus-contrast, var(--lsx-color-primary-6))`.
/// A CSS fallback applies only when the custom property is **not set at all**;
/// a property that *is* set to an invalid value is "invalid at computed-value
/// time", which is a different rule. So the question is whether the ring falls
/// back to primary or disappears - and that cannot be read off the emitted CSS,
/// only measured in a browser.
#[component]
fn FocusContrastPage() -> Element {
    rsx! {
        document::Style {
            ":root {{ --lsx-focus-contrast: var(--nothing-declares-this); }}"
        }
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "ring-probe", "Probe" }
        }
    }
}

/// Todo 53, part two: a link inside a `Mark` sits on the tint, so its ring has
/// to be drawn from the tint's contrast twin, not the primary fallback.
///
/// One `Mark` per palette colour, because the fallback's contrast against a
/// tint varies with the hue: it cleared 3:1 on some and not on others.
#[component]
fn MarkPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Text {
                Mark {
                    Anchor { id: "mark-default", to: "#", "a link inside a mark" }
                }
            }
            for (id, color) in MARK_COLORS {
                Text {
                    Mark { color: *color,
                        Anchor { id: *id, to: "#", "a link inside a mark" }
                    }
                }
            }
        }
    }
}

/// The ids `tests/all/mark.rs` tabs to, with the `Mark` colour each sits on.
/// `mark-default` sits on the theme's default and is rendered separately.
const MARK_COLORS: &[(&str, &str)] = &[
    ("mark-primary", "primary"),
    ("mark-secondary", "secondary"),
    ("mark-error", "error"),
    ("mark-info", "info"),
    ("mark-success", "success"),
];

/// The one fixture that exists for an engine rather than for a component.
///
/// The highlighter has two regex arms: the browser's `RegExp` on wasm32, and
/// the `regex` crate everywhere else - which is what **every** `cargo test`
/// runs (`codebase/highlighter-regex-engines`). So the arm users see was the
/// arm nothing tested; todo 279's fix was verified on the web by driving a
/// browser by hand, once.
///
/// This line is the one the two engines disagreed on. `\b` is ASCII in
/// JavaScript and Unicode in the `regex` crate, so `é` is a word character to
/// one and not to the other: before the fix the `if` in `éif` was a keyword on
/// the web and plain text natively, and under `fullstack` the server's markup
/// contradicted the client that hydrated it. Nothing else about the line
/// matters - the second `if` is the control, since every engine marks it.
#[component]
fn CodePage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Code { id: "non-ascii-code", language: "rust", source: NON_ASCII_SOURCE }
            CodeBlock { id: "numbered-block", language: "rust", line_numbers: true, source: "let a = 1;\nlet b = 2;" }
            // Todo 434: the leading `é` puts the scan's byte offsets apart from `RegExp`'s UTF-16 ones.
            Code { id: "nested-comment-code", language: "rust", source: "é /* a /* b */ c */ x" }
        }
    }
}

/// Kept in step with libero's own `a_keyword_after_a_non_ascii_letter_
/// tokenizes_as_it_does_on_the_web`, which asserts the same spans off the
/// `regex` arm. The pair is the point: one line, both engines.
const NON_ASCII_SOURCE: &str = "éif x; if y";

/// A deliberately awkward fixture: `Tree` is listed under the roving-tabindex
/// archetype, and this exists to find out whether that claim survives contact.
/// A tree is roving *and* hierarchical - it has expansion, levels, and rows
/// that appear and disappear - none of which a flat strip has.
#[component]
fn TreePage() -> Element {
    let data = vec![
        TreeNode::new("src", "src").children(vec![
            TreeNode::new("src/lib.rs", "lib.rs"),
            TreeNode::new("src/main.rs", "main.rs"),
        ]),
        TreeNode::new("docs", "docs").children(vec![TreeNode::new("docs/index.md", "index.md")]),
        TreeNode::new("README.md", "README.md"),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Tree {
                aria_label: "Project files",
                data,
                // Collapsed on purpose: an expandable row that is closed is the
                // case a flat archetype cannot see.
                render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                    rsx! {
                        TreeItem { tabindex: args.tabindex, "{args.data}" }
                    }
                },
            }
        }
    }
}
