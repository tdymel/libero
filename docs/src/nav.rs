use std::collections::HashSet;

use dioxus::prelude::*;
use libero::{
    components::{
        Flex, NavLink, Sidebar, SpotlightAction, States, Tree, TreeLabel, TreeNode,
        TreeNodeRenderArgs, default_tree_render,
    },
    hooks::ElementHandle,
    platform::ElementApi,
    sx::{Sx, sx},
    theme::{ColorCss, ColorShade, SIDEBAR_SIZE, Size},
};

use crate::Route;

// Below `Sm` an off-canvas panel toggled by `open` (`visibility` drops closed links from tab
// order); from `Sm` up the sticky sidebar, ignoring `open`. With `drawer` off-canvas everywhere.
fn nav_responsive_sx(open: bool, drawer: bool) -> Sx {
    // Only closing delays `visibility`, so the panel slides away instead of vanishing.
    let transition = if open {
        "transform 200ms ease, visibility 0s"
    } else {
        "transform 200ms ease, visibility 0s 200ms"
    };

    // "ColorSchemeButton", the longest label, needs 4px past `Sm` to stay on one row.
    let width = format!("calc({} + 8px)", SIDEBAR_SIZE.value(Size::Sm));
    // Absolute in the row below the header, not fixed at the header's height:
    // Blitz lays a fixed box out like an absolute one, which put it a header lower.
    let base = sx()
        .position("absolute")
        .top("0")
        .height("100%")
        .width("100%")
        // Overlapping the content, so it needs its own background.
        .background("surface")
        // `auto` would paint below any positioned content; well under `Modal`'s 1000.
        .z_index("10")
        .transform(if open {
            "translateX(0)"
        } else {
            "translateX(-100%)"
        })
        .visibility(if open { "visible" } else { "hidden" })
        .transition(transition)
        .media("(prefers-reduced-motion: reduce)", sx().transition("none"));
    if drawer {
        return base.breakpoint(Size::Sm, sx().width(width));
    }
    base.breakpoint(
        Size::Sm,
        sx().position("sticky")
            .top("0")
            .height("100%")
            .width(width)
            .transform("none")
            .visibility("visible"),
    )
}

#[derive(Clone, PartialEq)]
struct NavEntry {
    label: &'static str,
}

impl TreeLabel for NavEntry {
    fn tree_label(&self) -> String {
        self.label.to_string()
    }
}

fn page(route: Route, label: &'static str) -> TreeNode<NavEntry> {
    TreeNode::new(route.to_string(), NavEntry { label })
}

// A synthetic id (never a real route path, which always starts with `/`) -
// just avoids any chance of colliding with one.
fn group(
    id: &'static str,
    label: &'static str,
    children: Vec<TreeNode<NavEntry>>,
) -> TreeNode<NavEntry> {
    TreeNode::new(format!("group:{id}"), NavEntry { label }).children(children)
}

/// One palette action per page, grouped under its sidebar section - what the
/// docs search opens on until a real search index exists.
pub fn page_actions() -> Vec<SpotlightAction> {
    pages()
        .into_iter()
        .map(|(path, label, group)| {
            let mut action = SpotlightAction::new(label).onclick(move |_| {
                if let Ok(route) = path.parse::<Route>() {
                    navigator().push(route);
                }
            });
            // The section is a keyword too: "Accessibility" finds its "Overview".
            action.keywords = group.map(str::to_string).into_iter().collect();
            action.group = group.map(str::to_string);
            action
        })
        .collect()
}

/// The pages before and after `route` in sidebar order, for the docs pager.
pub fn neighbours(route: &Route) -> [Option<(Route, &'static str)>; 2] {
    let pages = pages();
    let path = route.to_string();
    let Some(at) = pages.iter().position(|(id, ..)| *id == path) else {
        return [None, None];
    };
    let pick = |index: Option<usize>| {
        let (id, label, _) = pages.get(index?)?;
        Some((id.parse::<Route>().ok()?, *label))
    };
    [pick(at.checked_sub(1)), pick(Some(at + 1))]
}

/// The sidebar label of `route`'s page.
pub fn page_label(route: &Route) -> Option<&'static str> {
    let path = route.to_string();
    pages()
        .into_iter()
        .find(|(id, ..)| *id == path)
        .map(|(_, label, _)| label)
}

// Every page as (path, label, group label), in sidebar order.
fn pages() -> Vec<(String, &'static str, Option<&'static str>)> {
    fn walk(
        nodes: Vec<TreeNode<NavEntry>>,
        group: Option<&'static str>,
        out: &mut Vec<(String, &'static str, Option<&'static str>)>,
    ) {
        for node in nodes {
            if node.children.is_empty() {
                out.push((node.id, node.data.label, group));
            } else {
                walk(node.children, Some(node.data.label), out);
            }
        }
    }
    let mut out = Vec::new();
    walk(nav_tree(), None, &mut out);
    out
}

fn nav_tree() -> Vec<TreeNode<NavEntry>> {
    vec![
        group(
            "about",
            "About",
            vec![
                page(Route::GettingStarted {}, "Getting started"),
                page(Route::PhilosophyPage {}, "Philosophy"),
                page(Route::StylingPage {}, "Styling"),
                page(Route::ThemingPage {}, "Theming"),
                page(Route::LocalizationPage {}, "Localization"),
                page(Route::PlatformPage {}, "Platform"),
            ],
        ),
        // The two building blocks first, then arranging, sizing, and the app shell.
        group(
            "layout",
            "Layout",
            vec![
                page(Route::BoxPage {}, "Box"),
                page(Route::PaperPage {}, "Paper"),
                page(Route::FlexPage {}, "Flex"),
                page(Route::GridPage {}, "Grid"),
                page(Route::CenterPage {}, "Center"),
                page(Route::ContainerPage {}, "Container"),
                page(Route::AspectRatioPage {}, "AspectRatio"),
                page(Route::DividerPage {}, "Divider"),
                page(Route::CollapsePage {}, "Collapse"),
                page(Route::FloatPage {}, "Float"),
                page(Route::HeaderPage {}, "Header"),
                page(Route::SidebarPage {}, "Sidebar"),
                page(Route::SplitterPage {}, "Splitter"),
                page(Route::ScrollAreaPage {}, "ScrollArea"),
                page(Route::ScrollerPage {}, "Scroller"),
            ],
        ),
        group(
            "buttons",
            "Buttons",
            vec![
                page(Route::ButtonPage {}, "Button"),
                page(Route::ActionIconPage {}, "ActionIcon"),
                page(Route::CopyButtonPage {}, "CopyButton"),
                page(Route::DirectionTogglePage {}, "DirectionToggle"),
                page(Route::RepoButtonPage {}, "RepoButton"),
                page(Route::ThemeTogglePage {}, "ThemeToggle"),
            ],
        ),
        // Fields built on `use_field`, ordered for reading: guide, containers, then fields
        // from the most common on, related ones together.
        group(
            "form",
            "Form",
            vec![
                page(Route::FormGettingStartedPage {}, "Getting started"),
                page(Route::FormPage {}, "Form"),
                page(Route::FieldsetPage {}, "Fieldset"),
                page(Route::TextFieldPage {}, "TextField"),
                page(Route::TextareaPage {}, "Textarea"),
                page(Route::PasswordFieldPage {}, "PasswordField"),
                page(Route::PhoneFieldPage {}, "PhoneField"),
                page(Route::NumberFieldPage {}, "NumberField"),
                page(Route::PinFieldPage {}, "PinField"),
                page(Route::AutocompletePage {}, "Autocomplete"),
                page(Route::SelectPage {}, "Select"),
                page(Route::MultiSelectPage {}, "MultiSelect"),
                page(Route::TagsFieldPage {}, "TagsField"),
                page(Route::CascaderPage {}, "Cascader"),
                page(Route::NativeSelectPage {}, "NativeSelect"),
                page(Route::ComboboxPage {}, "Combobox"),
                page(Route::CheckboxPage {}, "Checkbox"),
                page(Route::ChipPage {}, "Chip"),
                page(Route::SwitchPage {}, "Switch"),
                page(Route::RadioGroupPage {}, "RadioGroup"),
                page(Route::SegmentedControlPage {}, "SegmentedControl"),
                page(Route::SliderPage {}, "Slider"),
                page(Route::RangeSliderPage {}, "RangeSlider"),
                page(Route::ColorFieldPage {}, "ColorField"),
                page(Route::ColorPickerPage {}, "ColorPicker"),
                page(Route::ChronoFieldPage {}, "ChronoField"),
                page(Route::ChronoPickerPage {}, "ChronoPicker"),
                page(Route::FileFieldPage {}, "FileField"),
            ],
        ),
        // Links, then section switchers, then step and tree navigation.
        group(
            "navigation",
            "Navigation",
            vec![
                page(Route::AnchorPage {}, "Anchor"),
                page(Route::NavLinkPage {}, "NavLink"),
                page(Route::BurgerPage {}, "Burger"),
                page(Route::TabsPage {}, "Tabs"),
                page(Route::MenubarPage {}, "Menubar"),
                page(Route::PaginationPage {}, "Pagination"),
                page(Route::StepperPage {}, "Stepper"),
                page(Route::TreePage {}, "Tree"),
            ],
        ),
        // The backdrop, the modal family, the popups anchored to a trigger,
        // then the specialised windows.
        group(
            "overlay",
            "Overlay",
            vec![
                page(Route::OverlayPage {}, "Overlay"),
                page(Route::ModalPage {}, "Modal"),
                page(Route::DialogPage {}, "Dialog"),
                page(Route::DrawerPage {}, "Drawer"),
                page(Route::PopoverPage {}, "Popover"),
                page(Route::TooltipPage {}, "Tooltip"),
                page(Route::HoverCardPage {}, "HoverCard"),
                page(Route::MenuPage {}, "Menu"),
                page(Route::SpotlightPage {}, "Spotlight"),
                page(Route::LightboxPage {}, "Lightbox"),
                page(Route::FloatingWindowPage {}, "FloatingWindow"),
            ],
        ),
        group(
            "feedback",
            "Feedback",
            vec![
                page(Route::AlertPage {}, "Alert"),
                page(Route::NotificationsPage {}, "Notifications"),
                page(Route::LoaderPage {}, "Loader"),
                page(Route::ProgressBarPage {}, "ProgressBar"),
                page(Route::SkeletonPage {}, "Skeleton"),
            ],
        ),
        // Small markers, pictures, collections, then structured data.
        group(
            "data-display",
            "Data display",
            vec![
                page(Route::IconPage {}, "Icon"),
                page(Route::BadgePage {}, "Badge"),
                page(Route::IndicatorPage {}, "Indicator"),
                page(Route::AvatarPage {}, "Avatar"),
                page(Route::ImagePage {}, "Image"),
                page(Route::ImageListPage {}, "ImageList"),
                page(Route::CarouselPage {}, "Carousel"),
                page(Route::ListPage {}, "List"),
                page(Route::DataListPage {}, "DataList"),
                page(Route::TablePage {}, "Table"),
                page(Route::TimelinePage {}, "Timeline"),
                page(Route::AccordionPage {}, "Accordion"),
                page(Route::MarqueePage {}, "Marquee"),
                page(Route::QrCodePage {}, "QrCode"),
            ],
        ),
        group(
            "typography",
            "Typography",
            vec![
                page(Route::TitlePage {}, "Title"),
                page(Route::TextPage {}, "Text"),
                page(Route::MarkPage {}, "Mark"),
                page(Route::CodePage {}, "Code"),
                page(Route::KbdPage {}, "Kbd"),
                page(Route::CodeBlockPage {}, "CodeBlock"),
                page(Route::BlockquotePage {}, "Blockquote"),
            ],
        ),
        group(
            "accessibility",
            "Accessibility",
            vec![
                page(Route::AccessibilityPage {}, "Overview"),
                page(Route::FocusTrapPage {}, "FocusTrap"),
                page(Route::VisuallyHiddenPage {}, "VisuallyHidden"),
            ],
        ),
        // The overview, the primitives, then the app-wide ones. A hook a
        // component or guide page already shows well has no page of its own.
        group(
            "hooks",
            "Hooks",
            vec![
                page(Route::HooksPage {}, "Overview"),
                page(Route::UseIdPage {}, "use_id"),
                page(Route::UseElementPage {}, "use_element"),
                page(Route::UseFocusReturnPage {}, "use_focus_return"),
                page(Route::UseDragPage {}, "use_drag"),
                page(Route::UseThemeSetPage {}, "use_theme_set"),
                page(Route::UseStylesheetPage {}, "use_stylesheet"),
                page(Route::UseAccessibilityPage {}, "use_accessibility"),
            ],
        ),
    ]
}

fn ancestor_group(data: &[TreeNode<NavEntry>], target: &str) -> Option<String> {
    data.iter()
        .find(|node| node.children.iter().any(|child| child.id == target))
        .map(|node| node.id.clone())
}

#[component]
pub fn DocsNav(
    open: Signal<bool>,
    burger: ElementHandle,
    /// An off-canvas drawer at every width, not a sidebar from `Sm` up.
    #[props(default)]
    drawer: bool,
) -> Element {
    let data = nav_tree();
    let current_path = use_route::<Route>().to_string();
    let group = ancestor_group(&data, &current_path);

    // `Tree`'s open sections. A page in a closed section (the search reaches
    // any) opens it; only a new page does, so the reader can close it again.
    let mut expanded = use_signal(|| group.iter().cloned().collect::<HashSet<_>>());
    use_effect(use_reactive!(|group| {
        if let Some(group) = group
            && !expanded.peek().contains(&group)
        {
            expanded.write().insert(group);
        }
    }));

    rsx! {
        Sidebar {
            // What the header's `Burger` names in its `aria-controls`.
            id: "docs-nav",
            side: "start",
            component: "nav",
            sx: nav_responsive_sx(open(), drawer),
            Flex {
                direction: "column",
                gap: "sm",
                Tree {
                    // Remounts between drawer and column: Blitz kept the
                    // drawer's text layout, one letter per line (838).
                    key: "{drawer}",
                    aria_label: "Documentation pages",
                    size: "xs",
                    // Zero gap and indent at every depth: leaf borders form one continuous line,
                    // in line with the parent chevron. `args.depth` only pads the label.
                    sx: sx()
                        .gap("0")
                        .selector("& ul", sx().gap("0").padding_inline_start("0")),
                    data,
                    expanded: expanded(),
                    onexpandedchange: move |open: HashSet<String>| expanded.set(open),
                    // The tab stop starts on the current page, not "Guides".
                    current: current_path,
                    render_node: move |args: TreeNodeRenderArgs<NavEntry>| {
                        if args.expanded.is_some() {
                            return default_tree_render(args);
                        }
                        let padding_start = 7 + args.depth as u32 * 16;
                        // A page link (not a chevron) closes the panel and focuses the burger. `NavLink`
                        // has no `onclick`, so a `display: contents` wrapper catches the bubble.
                        rsx! {
                            div {
                                display: "contents",
                                onclick: move |_| {
                                    if open() {
                                        open.set(false);
                                        let _ = burger
                                            .query_selector("button")
                                            .and_then(|button| button.focus());
                                    }
                                },
                                NavLink {
                                    to: NavigationTarget::Internal(args.id),
                                    // `Tree`'s roving `<li>` is the only tab stop.
                                    tabindex: args.tabindex,
                                    scroll_into_view: true,
                                    // Only a nested leaf gets the connecting border.
                                    states: States::new().with("leaf", args.depth > 0),
                                    // The border follows `NavLink`'s own active state. Stretched and
                                    // unrounded, so it runs continuously between rows.
                                    sx: sx()
                                        .align_self("stretch")
                                        .padding_inline_start(0)
                                        .when("leaf", sx()
                                            .border_radius("0")
                                            .margin_inline_start("7px")
                                            .padding_inline_start(format!("{padding_start}px"))
                                            .border_inline_start(format!(
                                                "2px solid {}",
                                                ColorCss::MUTED.value(ColorShade::S3),
                                            ))
                                            // The rail is the one active indicator, so
                                            // `NavLink`'s start bar goes; in the bar's colour, which reads on the tint.
                                            .when(
                                                "active",
                                                sx().border_inline_start(format!(
                                                    "2px solid var({}, {})",
                                                    ColorCss::PRIMARY.role_name("on-tint-", ColorShade::S6),
                                                    ColorCss::PRIMARY.role_value("text-", ColorShade::S6),
                                                ))
                                                .with("background-image", "none"),
                                            )
                                        ),
                                    "{args.data.label}"
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use libero::components::spotlight_filter;

    use super::*;

    #[test]
    fn a_section_name_finds_its_overview() {
        // `onclick` makes a `Callback`, which needs a runtime.
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, page_actions);
        let hits = spotlight_filter("Accessibility", &actions);
        let hit = |label: &str| {
            hits.iter().any(|action| {
                action.label == label && action.group.as_deref() == Some("Accessibility")
            })
        };
        assert!(hit("Overview") && hit("FocusTrap"));
    }
}
