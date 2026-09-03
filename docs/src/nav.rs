use std::collections::HashSet;

use dioxus::prelude::*;
use libero::{
    components::{
        Flex, NavLink, Sidebar, States, Tree, TreeLabel, TreeNode, TreeNodeRenderArgs,
        default_tree_render,
    },
    sx::{Sx, sx},
    theme::{ColorCss, ColorShade, HEADER_HEIGHT, SIDEBAR_SIZE, Size},
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
fn nav_responsive_sx(open: bool) -> Sx {
    let header_height = HEADER_HEIGHT.value(Size::Md);
    // `visibility` shouldn't flip to hidden until the slide-out finishes,
    // or the panel would vanish mid-animation instead of sliding away;
    // opening has no such concern, so only closing gets the delay.
    let transition = if open {
        "transform 200ms ease, visibility 0s"
    } else {
        "transform 200ms ease, visibility 0s 200ms"
    };

    sx().position("fixed")
        .top(header_height.clone())
        .height(format!("calc(100vh - {header_height})"))
        .width("100%")
        // `Sidebar`'s own base has no background - fine sitting adjacent to
        // content in normal flow (desktop), but this mode overlaps the main
        // content, which would otherwise show through underneath it.
        .background("white")
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
        .breakpoint(
            Size::Sm,
            sx().position("sticky")
                .top("0")
                .height("100%")
                .width(SIDEBAR_SIZE.value(Size::Sm))
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

fn nav_tree() -> Vec<TreeNode<NavEntry>> {
    vec![
        group(
            "about",
            "About",
            vec![
                page(Route::GettingStarted {}, "Getting Started"),
                page(Route::StylingPage {}, "Styling"),
                page(Route::ThemingPage {}, "Theming"),
                page(Route::PerformancePage {}, "Performance"),
            ],
        ),
        group(
            "a11y",
            "A11y",
            vec![
                page(Route::FocusTrapPage {}, "Focus Trap"),
                page(Route::VisuallyHiddenPage {}, "Visually Hidden"),
            ],
        ),
        group(
            "data-display",
            "Data Display",
            vec![
                page(Route::AvatarPage {}, "Avatar"),
                page(Route::BadgePage {}, "Badge"),
                page(Route::DataListPage {}, "DataList"),
                page(Route::IconPage {}, "Icon"),
                page(Route::ImagePage {}, "Image"),
                page(Route::IndicatorPage {}, "Indicator"),
                page(Route::ListPage {}, "List"),
                page(Route::QrCodePage {}, "QrCode"),
                page(Route::TablePage {}, "Table"),
                page(Route::TimelinePage {}, "Timeline"),
            ],
        ),
        group(
            "feedback",
            "Feedback",
            vec![
                page(Route::AlertPage {}, "Alert"),
                page(Route::LoaderPage {}, "Loader"),
                page(Route::ProgressBarPage {}, "ProgressBar"),
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
                page(Route::FormGettingStartedPage {}, "Getting Started"),
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
        group(
            "inputs",
            "Inputs",
            vec![
                page(Route::ActionIconPage {}, "ActionIcon"),
                page(Route::ButtonPage {}, "Button"),
                page(Route::ChipPage {}, "Chip"),
            ],
        ),
        group(
            "layout",
            "Layout",
            vec![
                page(Route::AspectRatioPage {}, "AspectRatio"),
                page(Route::BoxPage {}, "Box"),
                page(Route::CenterPage {}, "Center"),
                page(Route::CollapsePage {}, "Collapse"),
                page(Route::ContainerPage {}, "Container"),
                page(Route::DividerPage {}, "Divider"),
                page(Route::FlexPage {}, "Flex"),
                page(Route::FloatPage {}, "Float"),
                page(Route::GridPage {}, "Grid"),
                page(Route::HeaderPage {}, "Header"),
                page(Route::ImageListPage {}, "ImageList"),
                page(Route::ScrollAreaPage {}, "ScrollArea"),
                page(Route::SidebarPage {}, "Sidebar"),
                page(Route::SplitterPage {}, "Splitter"),
            ],
        ),
        group(
            "navigation",
            "Navigation",
            vec![
                page(Route::BurgerPage {}, "Burger"),
                page(Route::AnchorPage {}, "Anchor"),
                page(Route::CarouselPage {}, "Carousel"),
                page(Route::NavLinkPage {}, "NavLink"),
                page(Route::PaginationPage {}, "Pagination"),
                page(Route::TabsPage {}, "Tabs"),
                page(Route::TreePage {}, "Tree"),
            ],
        ),
        group(
            "overlay",
            "Overlay",
            vec![
                page(Route::DrawerPage {}, "Drawer"),
                page(Route::ModalPage {}, "Modal"),
                page(Route::OverlayPage {}, "Overlay"),
                page(Route::PopoverPage {}, "Popover"),
                page(Route::TooltipPage {}, "Tooltip"),
            ],
        ),
        group(
            "surface",
            "Surface",
            vec![
                page(Route::PaperPage {}, "Paper"),
                page(Route::DialogPage {}, "Dialog"),
            ],
        ),
        group(
            "typography",
            "Typography",
            vec![
                page(Route::BlockquotePage {}, "Blockquote"),
                page(Route::CodePage {}, "Code"),
                page(Route::CodeBlockPage {}, "CodeBlock"),
                page(Route::KbdPage {}, "Kbd"),
                page(Route::MarkPage {}, "Mark"),
                page(Route::TextPage {}, "Text"),
                page(Route::TitlePage {}, "Title"),
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
pub fn DocsNav(open: Signal<bool>) -> Element {
    let data = nav_tree();

    let current_path = try_router()
        .map(|router| router.full_route_string())
        .unwrap_or_default();

    // Only seeds which section starts open (deep-linking to a page opens
    // its section) - after that, expand/collapse is `Tree`'s own business,
    // including collapsing the section the active page is in.
    let mut default_expanded = HashSet::new();
    if let Some(group_id) = ancestor_group(&data, &current_path) {
        default_expanded.insert(group_id);
    }

    rsx! {
        Sidebar {
            // What the header's `Burger` names in its `aria-controls`.
            id: "docs-nav",
            side: "left",
            role: "navigation",
            sx: nav_responsive_sx(open()),
            Flex {
                direction: "column",
                gap: "sm",
                // Closes on any click inside - good enough for "tap a link,
                // the panel closes" without threading a callback through
                // `NavLink` (which has none, deliberately, same as
                // `Button`'s link mode). Only matters below `Sm`; at desktop
                // widths `open` never becomes true in the first place, since
                // the toggle that sets it is hidden there.
                onclick: move |_| open.set(false),
                Tree {
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
                    default_expanded,
                    render_node: move |args: TreeNodeRenderArgs<NavEntry>| {
                        if args.expanded.is_some() {
                            return default_tree_render(args);
                        }
                        let padding_left = 7 + args.depth as u32 * 16;
                        rsx! {
                            NavLink {
                                to: NavigationTarget::Internal(args.id),
                                // Suppresses the anchor's own native tab
                                // stop - `Tree`'s roving `<li>` is the only
                                // one; see `TreeNodeRenderArgs::tabindex`.
                                tabindex: args.tabindex,
                                scroll_into_view: true,
                                // Only a *nested* leaf (inside a group) gets
                                // the connecting border - a top-level page
                                // like "Getting Started" has no parent
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
                                            ColorCss::GREY.value(ColorShade::S3),
                                        ))
                                        .when(
                                            "active",
                                            sx().border_left(format!(
                                                "2px solid {}",
                                                ColorCss::PRIMARY.value(ColorShade::S6),
                                            )),
                                        )
                                    ),
                                "{args.data.label}"
                            }
                        }
                    },
                }
            }
        }
    }
}
