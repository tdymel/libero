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
mod roving;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Autocomplete, Button, Dialog, Flex, NotificationOptions, Notifications, Options, Slider,
        SliderChangeEvent, Tabs, Text, Tree, TreeItem, TreeNode, TreeNodeRenderArgs,
        use_notifications,
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
    #[route("/modal")]
    ModalPage {},
    #[route("/focus-contrast")]
    FocusContrastPage {},
    #[route("/tree")]
    TreePage {},
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
    #[route("/spotlight")]
    SpotlightPage {},

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
    #[route("/broken/contrast")]
    BrokenContrast {},
    #[route("/broken/console")]
    BrokenConsole {},
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
}

/// The component the framework was **not** built for.
///
/// Everything in `Suite` assumes a static page: open a route, measure it,
/// snapshot it. A notification is transient - it appears on an action and
/// removes itself on a timer - so its accessibility tree is a function of time,
/// and its live regions say different things at different moments.
///
/// `auto_close` is pinned off. A fixture that disappears while being measured
/// is not a test of the component, it is a race, and a suite whose whole point
/// is the absence of flakiness should not contain one. Auto-close behaviour is
/// worth testing and needs a different shape: drive the clock, do not wait on
/// it.
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
        }
    }
}

use combobox::{MultiSelectPage, SelectPage, TagsFieldPage};
use overlay::{DrawerPage, MenuPage, SpotlightPage};
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
fn BrokenContrast() -> Element {
    rsx! { broken::LowContrast {} }
}
#[component]
fn BrokenConsole() -> Element {
    rsx! { broken::ConsoleError {} }
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

/// The enum is the tab strip, so it is the fixture's whole configuration.
#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Billing,
    Admin,
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
