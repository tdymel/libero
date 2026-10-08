use libero::components::{TreeLabel, TreeNode};

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

/// `route`'s sidebar section and the path of that section's first page.
#[cfg(test)]
pub fn page_group(route: &Route) -> Option<(&'static str, String)> {
    let path = route.to_string();
    let pages = pages();
    let group = pages.iter().find(|(id, ..)| *id == path)?.2?;
    let first = pages.iter().find(|(.., g)| *g == Some(group))?.0.clone();
    Some((group, first))
}

// Every page as (path, label, group label), in sidebar order.
pub(super) fn pages() -> Vec<(String, &'static str, Option<&'static str>)> {
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
                page(Route::TourPage {}, "Tour"),
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
                page(Route::UseIndexedDbPage {}, "IndexedDB"),
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
    use std::fmt::Write as _;

    use dioxus::prelude::Routable;

    use super::*;

    /// `public/sitemap.xml` lists the home page and every sidebar page; rewritten when stale.
    #[test]
    fn the_sitemap_is_current() {
        let mut expected = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        let paths = std::iter::once("/".to_string()).chain(pages().into_iter().map(|p| p.0));
        for path in paths {
            let _ = writeln!(
                expected,
                "  <url><loc>{}{path}</loc></url>",
                crate::site::SITE
            );
        }
        expected.push_str("</urlset>\n");

        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public/sitemap.xml");
        if std::fs::read_to_string(&file).ok().as_deref() != Some(&expected) {
            std::fs::write(&file, &expected).unwrap();
            panic!("docs/public/sitemap.xml was stale: rewritten, commit it");
        }
    }

    /// A page that is a route but not in the sidebar would be missing from the sitemap.
    #[test]
    fn every_static_route_is_a_sidebar_page_or_home() {
        let listed: Vec<String> = pages().into_iter().map(|p| p.0).collect();
        for route in Route::static_routes() {
            let path = route.to_string();
            assert!(
                path == "/" || listed.contains(&path),
                "{path} is not in the sitemap"
            );
        }
    }

    /// Pages may 301 a page that is also a directory to `/x/`; `route_pages.py` writes
    /// `x/index.html` for it, and the router must land on the same page.
    #[test]
    fn a_page_that_is_also_a_directory_takes_a_trailing_slash() {
        for (path, route) in [
            ("/accessibility/", Route::AccessibilityPage {}),
            ("/hooks/", Route::HooksPage {}),
        ] {
            assert_eq!(path.parse::<Route>().ok(), Some(route), "{path}");
        }
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
}
