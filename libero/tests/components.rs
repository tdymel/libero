//! One happy-path render per component: what it renders as, and the class /
//! `data-state` / custom-property triple that carries its styling.

mod common;

use common::{attributes_of, body, classes_of, has_rule_for, ids_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Anchor, AspectRatio, Box, Button, Center, Code, Container, DataList,
        DataListItem, Dialog, Divider, Drawer, Flex, Float, FocusTrap, Header, Icon, Image, Kbd,
        List, ListItem, Mark, Modal, NavLink, Option, Overlay, QrCode, ScrollArea, Select, Sidebar,
        Splitter, Text, Title, Tree, TreeNode, VisuallyHidden,
    },
    theme::{Color, Size},
};

#[test]
fn button_renders_its_class_state_and_variables() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { color: Color::Error, "Save" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["type"], "button");
    assert_eq!(attributes["data-state"], "outlined size-md radius-md");
    assert!(attributes["style"].contains("--lsx-button-color:var(--lsx-error-6);"));
    assert!(body(&html).contains(">Save<"));

    for class in classes_of(&html, "button") {
        assert!(has_rule_for(&html, &class), "{class} has no CSS rule");
    }
}

#[test]
fn flex_renders_its_children_in_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex {
                    span { "first" }
                    span { "second" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let first = body.find("first").expect("the first child");
    let second = body.find("second").expect("the second child");

    assert!(first < second);
    assert!(!classes_of(&html, "div").is_empty());
}

#[test]
fn box_renders_as_the_element_it_was_asked_for() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { component: "section", "content" }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("<section"));
    assert!(attributes_of(&html, "section").contains_key("class"));
}

#[test]
fn text_carries_its_size_as_a_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { size: Size::Lg, "body copy" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "p")["data-state"], "size-lg");
    assert!(body(&html).contains("body copy"));
}

#[test]
fn title_renders_as_its_heading_level() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Title { component: "h2", "Heading" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<h2"));
    assert!(attributes_of(&html, "h2").contains_key("data-state"));
}

#[test]
fn icon_renders_its_variant_and_colour_variables() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Icon { color: Color::Success, "★" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "span");

    assert_eq!(attributes["data-state"], "filled");
    assert!(attributes["style"].contains("--lsx-icon-color:var(--lsx-success-6);"));
}

#[test]
fn kbd_renders_its_key() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Kbd { "Ctrl" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<kbd"));
    assert!(body(&html).contains("Ctrl"));
}

#[test]
fn mark_highlights_with_its_background_variable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Mark { color: Color::Warning, "highlighted" }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "mark")["style"].contains("--lsx-mark-background"));
}

#[test]
fn visually_hidden_stays_in_the_accessibility_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { "screen reader only" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("screen reader only"));
}

#[test]
fn list_renders_its_items() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                List {
                    ListItem { "one" }
                    ListItem { "two" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<ul"));
    assert_eq!(body.matches("<li").count(), 2);
}

#[test]
fn data_list_pairs_a_label_with_its_value() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList {
                    DataListItem { label: rsx! { "Status" }, "Active" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("Status"));
    assert!(body.contains("Active"));
}

#[test]
fn select_renders_its_options_and_current_value() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Select {
                    value: "b",
                    Option { value: "a", "First" }
                    Option { value: "b", "Second" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<select"));
    assert_eq!(body.matches("<option").count(), 2);
}

#[test]
fn divider_renders_as_a_separator() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { color: Color::Grey }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "div")["style"].contains("--lsx-divider-color"));
}

#[test]
fn container_and_center_render_their_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Container {
                    Center { inline: true, "centred" }
                }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("centred"));
    assert!(body(&html).contains("--lsx-center-display-override:inline-flex;"));
}

#[test]
fn aspect_ratio_sets_its_ratio_variable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                AspectRatio { ratio: 1.5, "boxed" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("--lsx-aspect-ratio-override:1.5;"));
}

#[test]
fn action_icon_labels_itself_for_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Close", "×" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["aria-label"], "Close");
    assert_eq!(attributes["type"], "button");
}

#[test]
fn image_renders_its_source_and_alt_text() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Image { src: "/logo.png", alt: "The logo" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "img");

    assert_eq!(attributes["src"], "/logo.png");
    assert_eq!(attributes["alt"], "The logo");
}

#[test]
fn qr_code_renders_an_svg_labelled_for_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                QrCode { data: "https://example.com", aria_label: "Scan me" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "div")["aria-label"], "Scan me");
}

#[test]
fn header_renders_as_a_banner_landmark() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Header { "site header" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<header"));
    assert!(body(&html).contains("site header"));
}

#[test]
fn splitter_renders_both_panes_around_a_divider() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter { initial_size: 50.0,
                    div { "left" }
                    div { "right" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("left"));
    assert!(body.contains("right"));
    assert!(body.contains("--lsx-splitter-a:50%;"));
}

/// `touch-action: none` on the drag target is what lets a touch drag start
/// at all - without it the browser claims the gesture for scrolling and no
/// `pointermove` ever arrives. Nothing visible regresses if it goes missing,
/// so assert it reaches the rendered CSS. The drag itself needs real pointer
/// input, which SSR cannot produce.
#[test]
fn splitter_hit_target_opts_out_of_browser_touch_gestures() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter { initial_size: 50.0,
                    div { "left" }
                    div { "right" }
                }
            }
        }
    }

    assert!(render(app).contains("touch-action:none"));
}

#[test]
fn scroll_area_renders_its_content() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { "scrollable content" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("scrollable content"));
}

#[test]
fn float_renders_its_child_with_placement_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Float { placement: "top-end", "floating" }
            }
        }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];

    assert!(body(&html).contains("floating"));
    assert!(state.contains("vertical-top"), "got {state}");
    assert!(state.contains("horizontal-end"), "got {state}");
}

#[test]
fn a_modal_renders_through_the_portal_outlet() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                div { id: "in-place",
                    Modal { onclose: |_| {},
                        Dialog { "modal content" }
                    }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let in_place = body.find("in-place").expect("the in-place wrapper");
    let content = body.find("modal content").expect("the modal content");

    assert!(
        content > in_place,
        "portalled content should render at the outlet, after where it was written"
    );
}

#[test]
fn a_drawer_renders_through_the_portal_outlet() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Drawer { anchor: "right", onclose: |_| {}, "drawer content" }
            }
        }
    }

    let html = render(app);

    assert_eq!(html.matches("drawer content").count(), 1);
    assert!(
        html.contains(r#"data-state="size-md anchor-right""#),
        "got {html}"
    );
}

#[test]
fn a_sidebar_renders_in_place() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { "sidebar content" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("sidebar content"));
}

#[test]
fn a_sidebar_names_its_side_in_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { side: "right", "sidebar content" }
            }
        }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];
    assert!(state.contains("side-right"), "got {state}");
}

#[test]
fn overlay_renders_its_backdrop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Overlay {}
            }
        }
    }

    let html = render(app);
    let class = classes_of(&html, "div")
        .into_iter()
        .next()
        .expect("a class on the backdrop");

    assert!(has_rule_for(&html, &class));
}

#[test]
fn focus_trap_renders_its_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FocusTrap { "trapped" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("trapped"));
}

/// Two components with the same styling share one class and one emitted
/// rule - the registry is keyed by content, not by call site.
#[test]
fn identical_styling_is_registered_once() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { size: Size::Sm, "first" }
                Text { size: Size::Sm, "second" }
            }
        }
    }

    let html = render(app);
    let class = classes_of(&html, "p")
        .into_iter()
        .next()
        .expect("a class on the first Text");

    assert_eq!(body(&html).matches(&class).count(), 2);
    assert_eq!(html.matches(&format!(".{class}{{")).count(), 1);
}

#[test]
fn an_external_anchor_renders_a_plain_link() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", "Example" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(body(&html).contains("Example"));
}

#[test]
fn code_renders_its_source() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Code { "let x = 1;" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<code"));
    assert!(body(&html).contains("let x = 1;"));
}

#[test]
fn tree_renders_a_labelled_row_per_node() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string()),
        ];

        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(attributes_of(&html, "ul")["aria-label"], "Files");
    assert!(body.contains("Alpha"));
    assert!(body.contains("Beta"));
}

#[test]
fn nav_link_renders_a_link_with_its_active_background() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "https://example.com", active: true, "Docs" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(attributes["style"].contains("--lsx-nav-link-active-background"));
    assert!(attributes["data-state"].contains("active"));
}

/// Registering CSS used to re-render `LiberoProvider` itself - the component
/// owning `{children}` - which discarded the post-effect state of everything
/// under it. `Select` lost the `mounted` flag its `value` depends on that
/// way. The registered sheets now live in a `StyleOutlet` leaf instead, so a
/// registration dirties only that.
#[test]
fn effect_state_survives_a_stylesheet_registration() {
    #[component]
    fn Stateful() -> Element {
        let mut effect_ran = use_signal(|| false);
        use_effect(move || effect_ran.set(true));
        rsx! { Text { "effect_ran={effect_ran()}" } }
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Stateful {} } }
    }

    assert!(body(&render(app)).contains("effect_ran=true"));
}

/// `StyleOutlet` renders after `{children}`, and Dioxus renders child scopes
/// eagerly in tree order, so the creating render already carries every sheet
/// its descendants registered - no second pass needed for CSS.
#[test]
fn the_first_render_already_carries_the_component_css() {
    fn app() -> Element {
        rsx! { LiberoProvider { Button { "Press" } } }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    let class = classes_of(&html, "button")
        .into_iter()
        .find(|class| class.starts_with("lsx-"))
        .expect("the button carries a generated class");
    assert!(
        html.contains(&format!(".{class}")),
        "the class's own CSS is missing from the first render"
    );
}

/// The four components that look their own root up through `dom_api()` also
/// spread `attributes`, so a caller's `id` used to render *beside* theirs.
/// Browsers keep the first, which broke every lookup - scroll positions,
/// splitter drag, tree keyboard nav, focus trapping.
#[test]
fn a_root_id_component_renders_the_callers_id_once() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { id: "mine", "scrollable content" }
                Splitter { id: "mine", initial_size: 50.0,
                    div { "left" }
                    div { "right" }
                }
                Tree { id: "mine", aria_label: "Files", data: vec![TreeNode::new("a", "Alpha".to_string())] }
                FocusTrap { id: "mine", "trapped" }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(body.matches("id=\"mine\"").count(), 4);
    assert!(
        !body.contains("id=\"lsx-"),
        "a generated id shadowed the caller's:\n{body}"
    );
}

#[test]
fn a_root_id_component_falls_back_to_a_generated_id() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { "scrollable content" }
            }
        }
    }

    let ids = ids_of(&body(&render(app)), "div");

    assert_eq!(ids.len(), 1);
    assert!(
        ids[0].starts_with("lsx-"),
        "unexpected generated id: {}",
        ids[0]
    );
}

/// `min_size` floors *both* panes, so anything past 50 leaves `f64::clamp`
/// with `min > max` and panicked the whole subtree.
#[test]
fn splitter_survives_a_min_size_past_the_midpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter { initial_size: 50.0, min_size: 70.0,
                    div { "left" }
                    div { "right" }
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("left") && body.contains("right"));
    assert!(body.contains("aria-valuemin=\"50\""));
    assert!(body.contains("--lsx-splitter-a:50%;"));
}
