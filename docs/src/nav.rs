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
    theme::{ColorCss, ColorShade, HEADER_HEIGHT_VAR, SIDEBAR_SIZE, Size},
};

use crate::Route;

// One `Sidebar`, one responsive `sx` - no second drawer, no viewport
// detection. Below the `Sm` breakpoint it's a fixed, off-canvas panel
// toggled by `open` (slides via `transform`, `visibility` hidden when
// closed so its links drop out of tab order/the a11y tree instead of just
// being invisible); at `Sm` and up it's back to the persistent sticky
// sidebar from before, with `open` irrelevant - the breakpoint override
// hardcodes `transform`/`visibility` regardless of its value, so nothing
// odd happens if the viewport crosses `Sm` while it happens to be open.
// With `drawer` (the full-width home page) it stays off-canvas at every width,
// only narrower from `Sm` up.
fn nav_responsive_sx(open: bool, drawer: bool) -> Sx {
    // The banner's published height, whatever size it renders at.
    let header_height = HEADER_HEIGHT_VAR.value();
    // `visibility` shouldn't flip to hidden until the slide-out finishes,
    // or the panel would vanish mid-animation instead of sliding away;
    // opening has no such concern, so only closing gets the delay.
    let transition = if open {
        "transform 200ms ease, visibility 0s"
    } else {
        "transform 200ms ease, visibility 0s 200ms"
    };

    // "ColorSchemeButton", the longest label, needs 4px past `Sm` to stay on one row.
    let width = format!("calc({} + 8px)", SIDEBAR_SIZE.value(Size::Sm));
    let base = sx()
        .position("fixed")
        .top(header_height.clone())
        .height(format!("calc(100vh - {header_height})"))
        .width("100%")
        // `Sidebar`'s own base has no background - fine sitting adjacent to
        // content in normal flow (desktop), but this mode overlaps the main
        // content, which would otherwise show through underneath it.
        .background("surface")
        // `position: fixed` alone only creates a stacking context - without
        // an explicit z-index it's `auto`, which paints below anything else
        // on the page that happens to have a real (even low, even `0`)
        // z-index, letting that content's hit-testing win instead. Well
        // under `Modal`'s own range (starts at 1000) so an actual modal
        // still stacks above this.
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

/// The nav's component and hook pages, for the home page's counts. A group's
/// guide ("Getting started") and "Overview" are not counted.
pub fn page_counts() -> (usize, usize) {
    let (mut components, mut hooks) = (0, 0);
    for group in nav_tree() {
        let pages = group
            .children
            .iter()
            .filter(|page| !page.data.label.contains(' ') && page.data.label != "Overview")
            .count();
        match group.id.as_str() {
            "group:guides" => {}
            "group:hooks" => hooks += pages,
            _ => components += pages,
        }
    }
    (components, hooks)
}

fn nav_tree() -> Vec<TreeNode<NavEntry>> {
    vec![
        group(
            "guides",
            "Guides",
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
                page(Route::ColorSchemeButtonPage {}, "ColorSchemeButton"),
            ],
        ),
        // Fields built on `use_field`. A component moves here when it is
        // ported onto that pattern, not before. Ordered for reading, not
        // alphabetically: the guide, the containers, then the fields from the
        // most common on, with related fields next to each other.
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
                page(Route::DateFieldPage {}, "DateField"),
                page(Route::DatePickerPage {}, "DatePicker"),
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
        // The overview, the primitives, then each component's hooks in the groups' order.
        group(
            "hooks",
            "Hooks",
            vec![
                page(Route::HooksPage {}, "Overview"),
                page(Route::UseIdPage {}, "use_id"),
                page(Route::UseElementPage {}, "use_element"),
                page(Route::UseFocusReturnPage {}, "use_focus_return"),
                page(Route::UseDragPage {}, "use_drag"),
                page(Route::UseClipboardPage {}, "use_clipboard"),
                page(Route::UseThemePage {}, "use_theme"),
                page(Route::UseThemeSetPage {}, "use_theme_set"),
                page(Route::UseColorSchemePage {}, "use_color_scheme"),
                page(Route::UseLocalizationPage {}, "use_localization"),
                page(
                    Route::UseLocalizationHandlePage {},
                    "use_localization_handle",
                ),
                page(Route::UseFormatsPage {}, "use_formats"),
                page(Route::UseFormatsHandlePage {}, "use_formats_handle"),
                page(Route::UseStylesheetPage {}, "use_stylesheet"),
                page(Route::UseScrollAreaPage {}, "use_scroll_area"),
                page(Route::UseScrollerPage {}, "use_scroller"),
                page(Route::UseFormPage {}, "use_form"),
                page(Route::UseFormContextPage {}, "use_form_context"),
                page(Route::UseComboboxPage {}, "use_combobox"),
                page(Route::UseModalPage {}, "use_modal"),
                page(Route::UseModalClosePage {}, "use_modal_close"),
                page(Route::UseDrawerPage {}, "use_drawer"),
                page(Route::UsePopoverPage {}, "use_popover"),
                page(Route::UseMenuPage {}, "use_menu"),
                page(Route::UseSpotlightPage {}, "use_spotlight"),
                page(Route::UseLightboxPage {}, "use_lightbox"),
                page(Route::UseFloatingWindowPage {}, "use_floating_window"),
                page(Route::UseNotificationsPage {}, "use_notifications"),
                page(Route::UseNotificationsWithPage {}, "use_notifications_with"),
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

    // Mirrors `Tree`'s open sections. Deep-linking to a page opens its section;
    // after that, expand/collapse is `Tree`'s own business.
    let mut expanded = use_signal(|| group.iter().cloned().collect::<HashSet<_>>());
    // `Tree` has no controlled `expanded`: a page in a closed section (the
    // search reaches any) remounts it with that section open too.
    let mut reveals = use_signal(|| 0_u32);
    use_effect(use_reactive!(|group| {
        if let Some(group) = group
            && !expanded.peek().contains(&group)
        {
            expanded.write().insert(group);
            reveals += 1;
        }
    }));

    rsx! {
        Sidebar {
            // What the header's `Burger` names in its `aria-controls`.
            id: "docs-nav",
            side: "left",
            component: "nav",
            sx: nav_responsive_sx(open(), drawer),
            Flex {
                direction: "column",
                gap: "sm",
                Tree {
                    key: "{reveals}",
                    aria_label: "Documentation pages",
                    size: "xs",
                    // Both zeroed off-scale, so they go through `sx` rather
                    // than `size`, and the `& ul` half is what carries them
                    // into every nested group as well as the root.
                    //
                    // Zero gap between sibling rows - each `NavLink`'s own
                    // left border then reads as one continuous line down
                    // the section instead of a dashed one.
                    //
                    // `Tree` no longer reserves any leading chevron column of
                    // its own (that's purely `default_tree_render`'s thing
                    // now) - with indent also zeroed, every row, branch or
                    // leaf, at any depth, starts at the exact same x. That's
                    // what puts a leaf's border in line with its parent
                    // group's own chevron with no offsetting math needed;
                    // `args.depth` below only pads a leaf's *label* inward.
                    sx: sx()
                        .gap("0")
                        .selector("& ul", sx().gap("0").padding_left("0")),
                    data,
                    default_expanded: expanded.peek().clone(),
                    onexpandedchange: move |open: HashSet<String>| expanded.set(open),
                    // The tab stop starts on the current page, not "Guides".
                    current: current_path,
                    render_node: move |args: TreeNodeRenderArgs<NavEntry>| {
                        if args.expanded.is_some() {
                            return default_tree_render(args);
                        }
                        let padding_left = 7 + args.depth as u32 * 16;
                        // Closes the panel when a page link is clicked, and
                        // only then - a chevron must expand its section, not
                        // close the panel. `NavLink` takes no `onclick`
                        // (deliberately, same as `Button`'s link mode), so a
                        // `display: contents` wrapper catches the bubbling
                        // click, including the one `Tree` fires on Enter.
                        // Only matters below `Sm`; at desktop widths `open`
                        // never becomes true, since its toggle is hidden there.
                        //
                        // The link hides with the panel, so focus moves to the
                        // burger that reopens it rather than falling to
                        // `<body>`. A pointer click too: one handler, and the
                        // burger shows no ring after a pointer interaction.
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
                                    // Suppresses the anchor's own native tab
                                    // stop - `Tree`'s roving `<li>` is the only
                                    // one; see `TreeNodeRenderArgs::tabindex`.
                                    tabindex: args.tabindex,
                                    scroll_into_view: true,
                                    // Only a *nested* leaf (inside a group) gets
                                    // the connecting border - a top-level page
                                    // like "Getting started" has no parent
                                    // chevron to line up with.
                                    states: States::new().with("leaf", args.depth > 0),
                                    // `NavLink` already knows whether it's
                                    // active (it compares `to` against the
                                    // current route itself) and sets its own
                                    // `data-state="active"` - this just adds a
                                    // border reacting to that same state,
                                    // rather than `Tree` tracking "selected"
                                    // at all. `align-self: stretch` overrides
                                    // the row's own `align-items: center`, so
                                    // this spans the row's full height instead
                                    // of just its own text height - otherwise
                                    // the border would stop short top/bottom
                                    // and not read as continuous between rows.
                                    // `border-radius: 0` overrides `NavLink`'s
                                    // own rounded corners, which otherwise curve
                                    // the border away from the edge and break
                                    // that same continuity.
                                    sx: sx()
                                        .align_self("stretch")
                                        .padding_left(0)
                                        .when("leaf", sx()
                                            .border_radius("0")
                                            .margin_left("7px")
                                            .padding_left(format!("{padding_left}px"))
                                            .border_left(format!(
                                                "2px solid {}",
                                                ColorCss::MUTED.value(ColorShade::S3),
                                            ))
                                            // The rail is the one active indicator, so
                                            // `NavLink`'s start bar goes; in the bar's colour, which reads on the tint.
                                            .when(
                                                "active",
                                                sx().border_left(format!(
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
