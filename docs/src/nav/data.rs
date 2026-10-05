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
/// docs search opens on until a real search index exists. `section` lands a
/// section action on its `DocSection` or `ExtraTab`.
pub fn page_actions(section: Signal<Option<String>>) -> Vec<SpotlightAction> {
    let sections = SECTIONS
        .iter()
        .map(move |&(label, route, id, group, keywords)| {
            let mut action = SpotlightAction::new(label).onclick(move |_| {
                let mut section = section;
                section.set(Some(id.to_string()));
                navigator().push(route());
            });
            action.keywords = keywords.iter().map(|k| k.to_string()).collect();
            action.group = Some(group.to_string());
            action
        });
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
                .chain(aliases(group, label).iter().copied())
                .map(str::to_string)
                .collect();
            action.group = group.map(str::to_string);
            action
        })
        .chain(sections)
        .collect()
}

/// Search actions for a page's section: label, page, section id, group, keywords.
type SectionAction = (
    &'static str,
    fn() -> Route,
    &'static str,
    &'static str,
    &'static [&'static str],
);

const SECTIONS: &[SectionAction] = &[(
    "Icon catalogue",
    || Route::PictogramPage {},
    "icons",
    "Data display",
    &["icons", "icon search", "lucide", "browse icons"],
)];

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

// Pages sharing a label ("Getting started", "Overview") get theirs by section.
fn aliases(group: Option<&str>, label: &str) -> &'static [&'static str] {
    match (group, label) {
        (Some("About"), "Getting started") => &[
            "install",
            "installation",
            "setup",
            "quick start",
            "introduction",
            "cargo add",
        ],
        (Some("Form"), "Getting started") => &["forms", "use_field", "validation", "form state"],
        (Some("Accessibility"), "Overview") => &[
            "a11y",
            "aria",
            "screen reader",
            "keyboard navigation",
            "wcag",
        ],
        (Some("Hooks"), "Overview") => &["hooks", "use_", "utilities"],
        _ => label_aliases(label),
    }
}

// Hidden search names other libraries use for a page, and related words, by its label.
fn label_aliases(label: &str) -> &'static [&'static str] {
    match label {
        "Philosophy" => &["principles", "design goals", "vision"],
        "Styling" => &["css", "sx", "style props", "class", "custom styles"],
        "Theming" => &[
            "theme",
            "colors",
            "colours",
            "palette",
            "design tokens",
            "css variables",
            "dark mode",
        ],
        "Localization" => &[
            "i18n",
            "l10n",
            "translation",
            "language",
            "locale",
            "internationalization",
        ],
        "Providers" => &["context", "provider", "setup", "icon provider"],
        "Platform" => &["web", "desktop", "mobile", "native", "blitz", "wasm"],
        "Credits" => &["license", "attribution", "acknowledgements", "thanks"],
        "Box" => &["div", "element", "polymorphic"],
        "Paper" => &["card", "surface", "panel", "shadow", "elevation"],
        "Flex" => &[
            "stack", "row", "column", "group", "flexbox", "hstack", "vstack",
        ],
        "Grid" => &["simple grid", "layout", "css grid", "columns"],
        "Center" => &["centre", "align", "middle"],
        "Container" => &["max width", "wrapper", "content width"],
        "AspectRatio" => &["aspect ratio", "ratio"],
        "Divider" => &["separator", "rule", "hr"],
        "Collapse" => &["expand", "disclosure", "details", "show hide"],
        "Float" => &["floating"],
        "Header" => &["app bar", "top bar", "navbar"],
        "Sidebar" => &["navbar", "aside", "app shell", "side nav", "sidenav"],
        "Splitter" => &["resizable", "split pane", "panel group"],
        "Transition" => &["animate", "fade", "slide", "enter", "exit", "motion"],
        "ScrollArea" => &["scrollbar", "overflow", "scroll", "custom scrollbar"],
        "Scroller" => &["horizontal scroll", "scroll buttons", "overflow", "strip"],
        "Button" => &["btn", "click", "submit"],
        "ActionIcon" => &["icon button", "iconbutton"],
        "ButtonGroup" => &["button group", "attached buttons", "split button"],
        "Copy" => &["clipboard", "copy"],
        "DirectionToggle" => &["rtl", "ltr", "direction"],
        "Repository" => &["github", "repository"],
        "ThemeSwitcher" => &["dark mode", "color scheme", "light dark"],
        "Tldr" => &["summarize", "summary", "ai", "chatgpt", "claude"],
        "Toolbar" => &["tool bar", "action bar", "button bar"],
        "Form" => &["form field", "validation", "submit"],
        "Fieldset" => &["group", "legend"],
        "TextField" => &["input", "text input", "textinput", "text box", "textbox"],
        "Textarea" => &["multiline", "text area"],
        "RichTextEditor" => &[
            "wysiwyg",
            "rich text",
            "text editor",
            "markdown editor",
            "contenteditable",
        ],
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
        "Switch" => &["toggle", "toggle switch", "on off"],
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
        "ImageCropper" => &["crop", "cropper", "image crop", "avatar editor"],
        "Anchor" => &["link", "toc", "table of contents"],
        "NavLink" => &["nav item", "menu item", "link"],
        "BottomNavigation" => &["tab bar", "navigation bar", "mobile nav"],
        "Burger" => &["hamburger", "menu button", "nav toggle"],
        "Tabs" => &["tab list", "tabbed"],
        "Menubar" => &["menu bar"],
        "Pagination" => &["pager", "paging", "page numbers"],
        "Stepper" => &["steps", "wizard", "progress steps"],
        "Tree" => &["treeview", "tree view", "hierarchy", "file tree"],
        "Overlay" => &["backdrop", "scrim"],
        "Modal" => &["dialog", "popup"],
        "Dialog" => &["modal", "alert dialog", "confirm"],
        "Drawer" => &["sheet", "side panel", "offcanvas"],
        "Popover" => &["popup", "popper", "dropdown"],
        "Tooltip" => &["hint", "hover text", "title"],
        "HoverCard" => &["hover card", "preview card"],
        "Menu" => &["dropdown menu", "context menu", "actions menu"],
        "Spotlight" => &["command palette", "command k", "cmdk", "search"],
        "ShortcutHelp" => &["keyboard shortcuts", "hotkeys", "cheat sheet"],
        "Lightbox" => &["image viewer", "gallery", "zoom"],
        "FloatingWindow" => &["window", "draggable", "floating panel"],
        "Alert" => &["banner", "callout", "message", "notice", "warning"],
        "Notifications" => &["toast", "snackbar", "notify", "notification"],
        "Loader" => &["spinner", "loading", "activity indicator"],
        "ProgressBar" => &["progress", "meter"],
        "Skeleton" => &["placeholder", "shimmer", "loading"],
        // "icons" on all three, so "Icon" and "Icons" list the providers too (1445).
        "Icon" => &["icons", "glyph", "icon box"],
        "Pictogram" => &["icons", "svg", "lucide", "inline svg", "glyph"],
        "IconProvider" => &[
            "icons",
            "icon provider",
            "icon set",
            "icon theme",
            "swap icons",
            "lucide",
            "material",
            "tabler",
            "bootstrap",
            "phosphor",
        ],
        "Badge" => &["label", "tag", "pill", "chip"],
        "Indicator" => &["dot", "status dot", "notification badge"],
        "Avatar" => &["profile picture", "user picture", "initials"],
        "Image" => &["img", "picture", "photo"],
        "ImageList" => &["gallery", "masonry", "image grid"],
        "Audio" => &["sound", "music", "audio player", "podcast"],
        "Video" => &["video player", "movie", "media player", "clip"],
        "Carousel" => &["slideshow", "slider", "swiper"],
        "List" => &["ul", "ol", "bullet list"],
        "DataList" => &["description list", "definition list", "key value"],
        "Sortable" => &["drag and drop", "dnd", "reorder", "sortable list"],
        "Kanban" => &["board", "task board", "trello", "drag and drop", "columns"],
        "Table" => &["data grid", "datagrid", "datatable", "spreadsheet", "rows"],
        "Timeline" => &["history", "activity feed"],
        "Accordion" => &["expansion panel", "collapsible", "disclosure"],
        "Marquee" => &["ticker", "scrolling text"],
        "QrCode" => &["qr", "qrcode", "barcode"],
        "Title" => &["heading", "h1", "headline"],
        "Text" => &["typography", "paragraph", "body text", "font"],
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
        "Swipe" => &["use_swipe", "use_edge_swipe", "gesture", "edge swipe"],
        "Media" => &["use_media", "audio", "video", "player"],
        "Fullscreen" => &["use_fullscreen", "full screen", "maximize"],
        "Back button" => &["use_back", "android", "hardware back"],
        "Save file" => &["save_file", "download", "export", "share"],
        "Geolocation" => &[
            "use_geolocation",
            "location",
            "gps",
            "position",
            "permission",
        ],
        "Local storage" => &[
            "use_local_storage",
            "use_session_storage",
            "session storage",
            "persist",
            "remember",
        ],
        "User media" => &[
            "use_user_media",
            "use_user_media_devices",
            "camera",
            "microphone",
            "webcam",
            "record",
            "photo",
        ],
        "System notifications" => &[
            "use_system_notification",
            "use_push_subscription",
            "push",
            "web push",
            "desktop notification",
            "service worker",
        ],
        "Timers" => &["use_timeout", "use_interval", "setTimeout", "setInterval"],
        "Debounce and throttle" => &[
            "use_debounced_value",
            "use_debounced_callback",
            "use_throttled_value",
            "use_throttled_callback",
        ],
        "History" => &["use_history", "undo", "redo"],
        "Hotkeys" => &["use_hotkeys", "keyboard shortcut", "keybinding"],
        "Media query" => &["use_media_query", "use_is_mobile", "breakpoint", "mobile"],
        "Theme set" => &["use_theme_set", "switch theme"],
        "Stylesheet" => &["use_stylesheet", "css"],
        "Accessibility settings" => &["use_accessibility", "reduced motion", "contrast"],
        _ => &[],
    }
}

/// The title of `route`'s `DocPage`: its sidebar label; an "Overview" its section's name; a
/// label an earlier page already has, its section's name before it ("Form getting started").
#[cfg(test)]
pub fn page_title(route: &Route) -> Option<String> {
    let path = route.to_string();
    let pages = pages();
    let at = pages.iter().position(|(id, ..)| *id == path)?;
    let (label, group) = (pages[at].1, pages[at].2);
    let repeated = pages[..at].iter().any(|(_, earlier, _)| *earlier == label);
    Some(match group {
        Some(group) if label == "Overview" => group.to_string(),
        Some(group) if repeated => format!("{group} {}", label.to_lowercase()),
        _ => label.to_string(),
    })
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
                page(Route::CopyPage {}, "Copy"),
                page(Route::DirectionTogglePage {}, "DirectionToggle"),
                page(Route::RepositoryPage {}, "Repository"),
                page(Route::ThemeSwitcherPage {}, "ThemeSwitcher"),
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
                page(Route::RichTextEditorPage {}, "RichTextEditor"),
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
                page(Route::ImageCropperPage {}, "ImageCropper"),
            ],
        ),
        // Links, then section switchers, then step and tree navigation.
        group(
            "navigation",
            "Navigation",
            vec![
                page(Route::AnchorPage {}, "Anchor"),
                page(Route::NavLinkPage {}, "NavLink"),
                page(Route::BottomNavigationPage {}, "BottomNavigation"),
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
                page(Route::ShortcutHelpPage {}, "ShortcutHelp"),
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
                page(Route::AudioPage {}, "Audio"),
                page(Route::VideoPage {}, "Video"),
                page(Route::CarouselPage {}, "Carousel"),
                page(Route::ListPage {}, "List"),
                page(Route::DataListPage {}, "DataList"),
                page(Route::SortablePage {}, "Sortable"),
                page(Route::KanbanPage {}, "Kanban"),
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
                page(Route::UseSwipePage {}, "Swipe"),
                page(Route::UseTimersPage {}, "Timers"),
                page(Route::UseDebouncePage {}, "Debounce and throttle"),
                page(Route::UseHistoryPage {}, "History"),
                page(Route::UseHotkeysPage {}, "Hotkeys"),
                page(Route::UseMediaQueryPage {}, "Media query"),
                page(Route::UseMediaPage {}, "Media"),
                page(Route::UseFullscreenPage {}, "Fullscreen"),
                page(Route::UseBackPage {}, "Back button"),
                page(Route::UseGeolocationPage {}, "Geolocation"),
                page(Route::UseLocalStoragePage {}, "Local storage"),
                page(Route::UseUserMediaPage {}, "User media"),
                page(Route::UseSystemNotificationPage {}, "System notifications"),
                page(Route::SaveFilePage {}, "Save file"),
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
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let hits = spotlight_filter("Accessibility", &actions);
        let hit = |label: &str| {
            hits.iter().any(|action| {
                action.label == label && action.group.as_deref() == Some("Accessibility")
            })
        };
        assert!(hit("Overview") && hit("FocusTrap") && hit("Focus return"));
    }

    #[test]
    fn the_icon_catalogue_lands_on_its_tab() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let hits = spotlight_filter("browse icons", &actions);
        assert_eq!(hits[0].label, "Icon catalogue");
    }

    #[test]
    fn icon_and_icons_list_the_icon_providers() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        for query in ["Icon", "Icons"] {
            let hits = spotlight_filter(query, &actions);
            for label in ["Icon", "Pictogram", "IconProvider"] {
                assert!(
                    hits.iter().any(|hit| hit.label == label),
                    "{query}: {label}"
                );
            }
        }
    }

    #[test]
    fn every_page_has_search_words() {
        for (path, label, group) in pages() {
            assert!(!aliases(group, label).is_empty(), "{path}");
        }
    }

    #[test]
    fn a_shared_label_takes_its_sections_words() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let hits = spotlight_filter("a11y", &actions);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].group.as_deref(), Some("Accessibility"));
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
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
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
