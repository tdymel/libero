use dioxus::prelude::*;
use libero::components::{SpotlightAction, TreeLabel, TreeNode};

use crate::Route;

#[derive(Clone, PartialEq)]
pub(super) struct NavEntry {
    pub(super) label: &'static str,
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
            action.keywords = group
                .into_iter()
                .chain(aliases(label).iter().copied())
                .map(str::to_string)
                .collect();
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

// Hidden search names other libraries use for a page, by its label. Add a row to extend.
fn aliases(label: &str) -> &'static [&'static str] {
    match label {
        "Box" => &["div"],
        "Paper" => &["card", "surface"],
        "Flex" => &["stack", "row", "column", "group"],
        "Grid" => &["simple grid", "layout"],
        "Center" => &["centre"],
        "Container" => &["max width"],
        "AspectRatio" => &["aspect ratio", "ratio"],
        "Divider" => &["separator", "rule", "hr"],
        "Collapse" => &["expand", "disclosure", "details"],
        "Float" => &["floating"],
        "Header" => &["app bar", "top bar", "navbar"],
        "Sidebar" => &["navbar", "aside", "app shell"],
        "Splitter" => &["resizable", "split pane", "panel group"],
        "Transition" => &["animate", "fade", "slide", "enter", "exit", "motion"],
        "ScrollArea" => &["scrollbar", "overflow"],
        "Button" => &["btn"],
        "ActionIcon" => &["icon button", "iconbutton"],
        "CopyButton" => &["clipboard", "copy"],
        "DirectionToggle" => &["rtl", "ltr", "direction"],
        "RepoButton" => &["github", "repository"],
        "ThemeToggle" => &["dark mode", "color scheme", "light dark"],
        "Tldr" => &["summarize", "summary", "ai", "chatgpt", "claude"],
        "Form" => &["form field"],
        "Fieldset" => &["group", "legend"],
        "TextField" => &["input", "text input", "textinput"],
        "Textarea" => &["multiline", "text area"],
        "PasswordField" => &["password input", "secret"],
        "PhoneField" => &["phone input", "tel", "telephone"],
        "NumberField" => &["number input", "numeric", "spinner", "stepper"],
        "PinField" => &["pin input", "otp", "verification code"],
        "Autocomplete" => &["typeahead", "suggest", "search select"],
        "Select" => &["dropdown", "drop down", "listbox"],
        "MultiSelect" => &["multi select", "multiple select", "dropdown"],
        "TagsField" => &["tags input", "taginput", "chips input"],
        "Cascader" => &["cascading select", "nested select"],
        "NativeSelect" => &["native dropdown", "html select"],
        "Combobox" => &["combo box", "dropdown", "autocomplete"],
        "Checkbox" => &["check box", "tick"],
        "Chip" => &["tag", "pill", "toggle chip"],
        "Switch" => &["toggle"],
        "RadioGroup" => &["radio", "radio button", "option group"],
        "SegmentedControl" => &["segmented button", "button group", "toggle group"],
        "Slider" => &["range", "track"],
        "RangeSlider" => &["range", "two thumb", "min max"],
        "Rating" => &["stars", "star rating", "review score"],
        "ColorField" => &["color input", "colour"],
        "ColorPicker" => &["colour picker", "eyedropper", "swatch"],
        "ChronoField" => &[
            "DateField",
            "date picker",
            "datepicker",
            "date input",
            "time field",
            "time input",
            "datetime",
        ],
        "ChronoPicker" => &[
            "calendar",
            "date picker",
            "datepicker",
            "time picker",
            "datetime",
        ],
        "FileField" => &["file input", "upload", "dropzone", "file picker"],
        "Anchor" => &["link", "toc", "table of contents"],
        "NavLink" => &["nav item", "menu item", "link"],
        "Burger" => &["hamburger", "menu button", "nav toggle"],
        "Tabs" => &["tab list", "tabbed"],
        "Menubar" => &["menu bar"],
        "Pagination" => &["pager", "paging", "page numbers"],
        "Stepper" => &["steps", "wizard", "progress steps"],
        "Tree" => &["treeview", "tree view", "hierarchy"],
        "Overlay" => &["backdrop", "scrim"],
        "Modal" => &["dialog", "popup"],
        "Dialog" => &["modal", "alert dialog", "confirm"],
        "Drawer" => &["sheet", "side panel", "offcanvas"],
        "Popover" => &["popup", "popper", "dropdown"],
        "Tooltip" => &["hint", "hover text", "title"],
        "HoverCard" => &["hover card", "preview card"],
        "Menu" => &["dropdown menu", "context menu", "actions menu"],
        "Spotlight" => &["command palette", "command k", "cmdk", "search"],
        "Lightbox" => &["image viewer", "gallery", "zoom"],
        "FloatingWindow" => &["window", "draggable", "floating panel"],
        "Alert" => &["banner", "callout", "message"],
        "Notifications" => &["toast", "snackbar", "notify"],
        "Loader" => &["spinner", "loading", "activity indicator"],
        "ProgressBar" => &["progress", "meter"],
        "Skeleton" => &["placeholder", "shimmer", "loading"],
        "Icon" => &["glyph", "icon box"],
        "Pictogram" => &["svg", "lucide", "inline svg"],
        "IconProvider" => &["icon set", "icon theme", "swap icons", "lucide"],
        "Badge" => &["label", "tag", "pill", "chip"],
        "Indicator" => &["dot", "status dot", "notification badge"],
        "Avatar" => &["profile picture", "user picture", "initials"],
        "Image" => &["img", "picture", "photo"],
        "ImageList" => &["gallery", "masonry", "image grid"],
        "Carousel" => &["slideshow", "slider", "swiper"],
        "List" => &["ul", "ol", "bullet list"],
        "DataList" => &["description list", "definition list", "key value"],
        "Table" => &["data grid", "datagrid", "datatable"],
        "Timeline" => &["history", "activity feed"],
        "Accordion" => &["expansion panel", "collapsible", "disclosure"],
        "Marquee" => &["ticker", "scrolling text"],
        "QrCode" => &["qr", "qrcode", "barcode"],
        "Title" => &["heading", "h1", "headline"],
        "Text" => &["typography", "paragraph"],
        "Mark" => &["highlight"],
        "Code" => &["inline code", "monospace"],
        "Kbd" => &["keyboard", "key", "shortcut"],
        "CodeBlock" => &["pre", "code snippet", "syntax highlight"],
        "Blockquote" => &["quote", "citation"],
        "FocusTrap" => &["focus lock", "focus scope"],
        "VisuallyHidden" => &["sr only", "screen reader only", "hidden"],
        "Unique ID" => &["use_id", "id"],
        "Element handle" => &["use_element", "ref", "measure"],
        "Focus return" => &["use_focus_return", "restore focus"],
        "Drag" => &["use_drag", "pointer"],
        "Intersection" => &["use_intersection", "use_in_viewport", "viewport", "visible"],
        "Long press" => &["use_long_press", "hold"],
        "Timers" => &["use_timeout", "use_interval", "setTimeout", "setInterval"],
        "Debounce and throttle" => &[
            "use_debounced_value",
            "use_debounced_callback",
            "use_throttled_value",
            "use_throttled_callback",
        ],
        "Hotkeys" => &["use_hotkeys", "keyboard shortcut", "keybinding"],
        "Media query" => &["use_media_query", "use_is_mobile", "breakpoint", "mobile"],
        "Theme set" => &["use_theme_set", "switch theme"],
        "Stylesheet" => &["use_stylesheet", "css"],
        "Accessibility settings" => &["use_accessibility", "reduced motion", "contrast"],
        _ => &[],
    }
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

pub(super) fn nav_tree() -> Vec<TreeNode<NavEntry>> {
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
                page(Route::ProvidersPage {}, "Providers"),
                page(Route::PlatformPage {}, "Platform"),
                page(Route::CreditsPage {}, "Credits"),
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
                page(Route::TransitionPage {}, "Transition"),
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
                page(Route::ButtonGroupPage {}, "ButtonGroup"),
                page(Route::CopyButtonPage {}, "CopyButton"),
                page(Route::DirectionTogglePage {}, "DirectionToggle"),
                page(Route::RepoButtonPage {}, "RepoButton"),
                page(Route::ThemeTogglePage {}, "ThemeToggle"),
                page(Route::TldrPage {}, "Tldr"),
                page(Route::ToolbarPage {}, "Toolbar"),
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
                page(Route::RatingPage {}, "Rating"),
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
                page(Route::PictogramPage {}, "Pictogram"),
                page(Route::IconProviderPage {}, "IconProvider"),
                page(Route::BadgePage {}, "Badge"),
                page(Route::IndicatorPage {}, "Indicator"),
                page(Route::AvatarPage {}, "Avatar"),
                page(Route::ImagePage {}, "Image"),
                page(Route::ImageListPage {}, "ImageList"),
                page(Route::CarouselPage {}, "Carousel"),
                page(Route::ListPage {}, "List"),
                page(Route::DataListPage {}, "DataList"),
                page(Route::SortablePage {}, "Sortable"),
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
                page(Route::UseIdPage {}, "Unique ID"),
                page(Route::UseFocusReturnPage {}, "Focus return"),
                page(Route::UseAccessibilityPage {}, "Accessibility settings"),
            ],
        ),
        // The overview, the primitives, then the app-wide ones. A hook a
        // component or guide page already shows well has no page of its own.
        group(
            "hooks",
            "Hooks",
            vec![
                page(Route::HooksPage {}, "Overview"),
                page(Route::UseElementPage {}, "Element handle"),
                page(Route::UseDragPage {}, "Drag"),
                page(Route::UseIntersectionPage {}, "Intersection"),
                page(Route::UseLongPressPage {}, "Long press"),
                page(Route::UseTimersPage {}, "Timers"),
                page(Route::UseDebouncePage {}, "Debounce and throttle"),
                page(Route::UseHotkeysPage {}, "Hotkeys"),
                page(Route::UseMediaQueryPage {}, "Media query"),
                page(Route::UseThemeSetPage {}, "Theme set"),
                page(Route::UseStylesheetPage {}, "Stylesheet"),
            ],
        ),
    ]
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
        assert!(hit("Overview") && hit("FocusTrap") && hit("Focus return"));
    }

    /// The a11y hooks moved from Hooks to Accessibility: their old links still land.
    #[test]
    fn a_moved_hook_page_keeps_its_old_url() {
        for (old, route) in [
            ("/hooks/use-id", Route::UseIdPage {}),
            ("/hooks/use-focus-return", Route::UseFocusReturnPage {}),
            ("/hooks/use-accessibility", Route::UseAccessibilityPage {}),
        ] {
            assert_eq!(old.parse::<Route>().ok(), Some(route.clone()), "{old}");
            assert!(route.to_string().starts_with("/accessibility/"), "{route}");
        }
    }

    #[test]
    fn an_alias_finds_its_page_after_label_matches() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, page_actions);
        let labels = |query| {
            spotlight_filter(query, &actions)
                .into_iter()
                .map(|action| action.label)
                .collect::<Vec<_>>()
        };
        assert!(labels("datepicker").contains(&"ChronoField".to_string()));
        assert_eq!(labels("select")[0], "Select");
        let popup = labels("popup");
        assert!(popup.contains(&"Modal".to_string()) && popup.contains(&"Popover".to_string()));
        // "Menu" lists "Menu" and "Menubar" (labels) before "Burger" (alias "menu button").
        let menu = labels("menu");
        let at = |label: &str| menu.iter().position(|found| found == label).unwrap();
        assert!(at("Menubar") < at("Burger"));
    }
}
