// Matches libero: a `demo/demo.rs` beside its siblings reads better than a
// flattened `mod.rs`.
#![allow(clippy::module_inception)]

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Notifications, localization::Formats};

mod components;
mod exports;
mod heading_focus;
#[cfg(test)]
mod index_html;
mod nav;
mod pages;
mod shell;
mod site;
#[cfg(test)]
mod snippets;

use shell::AppShell;
use site::LOGO;
// A glob, so a new page never edits this file's import list.
use pages::*;

fn main() {
    // dioxus's release default, INFO, marks every span in the performance timeline: 2-4 ms per mount (todo 2208).
    if !cfg!(debug_assertions) {
        let _ = dioxus::logger::init(dioxus::logger::tracing::Level::WARN);
    }
    #[cfg(feature = "native-cpu")]
    dioxus_native::launch(App);
    #[cfg(not(feature = "native-cpu"))]
    dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq, Debug)]
pub(crate) enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},

    #[route("/about/getting-started")]
    GettingStarted {},
    #[route("/about/philosophy")]
    PhilosophyPage {},
    #[route("/about/styling")]
    StylingPage {},
    #[route("/about/theming")]
    ThemingPage {},
    #[route("/about/localization")]
    LocalizationPage {},
    #[route("/about/providers")]
    ProvidersPage {},
    #[route("/about/platform")]
    PlatformPage {},
    #[route("/about/credits")]
    CreditsPage {},

    #[route("/accessibility")]
    AccessibilityPage {},
    #[route("/accessibility/focus-trap")]
    FocusTrapPage {},
    #[route("/accessibility/visually-hidden")]
    VisuallyHiddenPage {},
    // The a11y hooks' URLs from before they moved here from Hooks (2026-10-02).
    #[redirect("/hooks/use-id", || Route::UseIdPage {})]
    #[redirect("/hooks/use-focus-return", || Route::UseFocusReturnPage {})]
    #[redirect("/hooks/use-accessibility", || Route::UseAccessibilityPage {})]
    #[route("/accessibility/use-id")]
    UseIdPage {},
    #[route("/accessibility/use-focus-return")]
    UseFocusReturnPage {},
    #[route("/accessibility/use-accessibility")]
    UseAccessibilityPage {},

    #[route("/hooks")]
    HooksPage {},
    #[route("/hooks/use-element")]
    UseElementPage {},
    #[route("/hooks/use-drag")]
    UseDragPage {},
    #[route("/hooks/use-intersection")]
    UseIntersectionPage {},
    #[route("/hooks/use-long-press")]
    UseLongPressPage {},
    #[route("/hooks/use-swipe")]
    UseSwipePage {},
    #[route("/hooks/use-timers")]
    UseTimersPage {},
    #[route("/hooks/use-debounce")]
    UseDebouncePage {},
    #[route("/hooks/use-history")]
    UseHistoryPage {},
    #[route("/hooks/use-hotkeys")]
    UseHotkeysPage {},
    #[route("/hooks/use-media-query")]
    UseMediaQueryPage {},
    #[route("/hooks/use-media")]
    UseMediaPage {},
    #[route("/hooks/use-back")]
    UseBackPage {},
    #[route("/hooks/use-fullscreen")]
    UseFullscreenPage {},
    #[route("/hooks/use-geolocation")]
    UseGeolocationPage {},
    #[route("/hooks/use-user-media")]
    UseUserMediaPage {},
    #[route("/hooks/use-system-notification")]
    UseSystemNotificationPage {},
    #[route("/hooks/save-file")]
    SaveFilePage {},
    #[route("/hooks/use-theme-set")]
    UseThemeSetPage {},
    #[route("/hooks/use-stylesheet")]
    UseStylesheetPage {},

    #[route("/buttons/action-icon")]
    ActionIconPage {},
    #[route("/buttons/button")]
    ButtonPage {},
    #[route("/buttons/button-group")]
    ButtonGroupPage {},
    #[route("/buttons/copy")]
    CopyPage {},
    #[route("/buttons/direction-toggle")]
    DirectionTogglePage {},
    #[route("/buttons/repository")]
    RepositoryPage {},
    #[route("/buttons/theme-switcher")]
    ThemeSwitcherPage {},
    #[route("/buttons/tldr")]
    TldrPage {},
    #[route("/buttons/toolbar")]
    ToolbarPage {},

    #[route("/data-display/accordion")]
    AccordionPage {},
    #[route("/data-display/carousel")]
    CarouselPage {},
    #[route("/data-display/image-list")]
    ImageListPage {},
    #[route("/data-display/avatar")]
    AvatarPage {},
    #[route("/data-display/badge")]
    BadgePage {},
    #[route("/data-display/data-list")]
    DataListPage {},
    #[route("/data-display/icon")]
    IconPage {},
    #[route("/data-display/pictogram")]
    PictogramPage {},
    #[route("/data-display/icon-provider")]
    IconProviderPage {},
    #[route("/data-display/image")]
    ImagePage {},
    #[route("/data-display/audio")]
    AudioPage {},
    #[route("/data-display/video")]
    VideoPage {},
    #[route("/data-display/indicator")]
    IndicatorPage {},
    #[route("/data-display/table")]
    TablePage {},
    #[route("/data-display/timeline")]
    TimelinePage {},
    #[route("/data-display/list")]
    ListPage {},
    #[route("/data-display/marquee")]
    MarqueePage {},
    #[route("/data-display/qr-code")]
    QrCodePage {},
    #[route("/data-display/sortable")]
    SortablePage {},
    #[route("/data-display/kanban")]
    KanbanPage {},

    #[route("/feedback/alert")]
    AlertPage {},
    #[route("/feedback/loader")]
    LoaderPage {},
    #[route("/feedback/notifications")]
    NotificationsPage {},
    #[route("/feedback/progress-bar")]
    ProgressBarPage {},
    #[route("/feedback/skeleton")]
    SkeletonPage {},

    #[route("/form/getting-started")]
    FormGettingStartedPage {},
    #[route("/form/autocomplete")]
    AutocompletePage {},
    #[route("/form/checkbox")]
    CheckboxPage {},
    #[route("/form/chip")]
    ChipPage {},
    #[route("/form/color-field")]
    ColorFieldPage {},
    #[route("/form/color-picker")]
    ColorPickerPage {},
    #[route("/form/combobox")]
    ComboboxPage {},
    #[route("/form/chrono-field")]
    ChronoFieldPage {},
    #[route("/form/chrono-picker")]
    ChronoPickerPage {},
    #[route("/form/fieldset")]
    FieldsetPage {},
    #[route("/form/file-field")]
    FileFieldPage {},
    #[route("/form/form")]
    FormPage {},
    #[route("/form/image-cropper")]
    ImageCropperPage {},
    #[route("/form/multi-select")]
    MultiSelectPage {},
    #[route("/form/cascader")]
    CascaderPage {},
    #[route("/form/native-select")]
    NativeSelectPage {},
    #[route("/form/number-field")]
    NumberFieldPage {},
    #[route("/form/password-field")]
    PasswordFieldPage {},
    #[route("/form/phone-field")]
    PhoneFieldPage {},
    #[route("/form/pin-field")]
    PinFieldPage {},
    #[route("/form/radio-group")]
    RadioGroupPage {},
    #[route("/form/range-slider")]
    RangeSliderPage {},
    #[route("/form/rating")]
    RatingPage {},
    #[route("/form/rich-text-editor")]
    RichTextEditorPage {},
    #[route("/form/segmented-control")]
    SegmentedControlPage {},
    #[route("/form/select")]
    SelectPage {},
    #[route("/form/slider")]
    SliderPage {},
    #[route("/form/switch")]
    SwitchPage {},
    #[route("/form/tags-field")]
    TagsFieldPage {},
    #[route("/form/text-field")]
    TextFieldPage {},
    #[route("/form/textarea")]
    TextareaPage {},

    #[route("/layout/aspect-ratio")]
    AspectRatioPage {},
    #[route("/layout/box")]
    BoxPage {},
    #[route("/layout/center")]
    CenterPage {},
    #[route("/layout/collapse")]
    CollapsePage {},
    #[route("/layout/container")]
    ContainerPage {},
    #[route("/layout/divider")]
    DividerPage {},
    #[route("/layout/flex")]
    FlexPage {},
    #[route("/layout/float")]
    FloatPage {},
    #[route("/layout/grid")]
    GridPage {},
    #[route("/layout/header")]
    HeaderPage {},
    #[route("/layout/paper")]
    PaperPage {},
    #[route("/layout/scroll-area")]
    ScrollAreaPage {},
    #[route("/layout/scroller")]
    ScrollerPage {},
    #[route("/layout/sidebar")]
    SidebarPage {},
    #[route("/layout/splitter")]
    SplitterPage {},
    #[route("/layout/transition")]
    TransitionPage {},

    #[route("/navigation/bottom-navigation")]
    BottomNavigationPage {},
    #[route("/navigation/burger")]
    BurgerPage {},
    #[route("/navigation/anchor")]
    AnchorPage {},
    #[route("/navigation/nav-link")]
    NavLinkPage {},
    #[route("/navigation/pagination")]
    PaginationPage {},
    #[route("/navigation/stepper")]
    StepperPage {},
    #[route("/navigation/tabs")]
    TabsPage {},
    #[route("/navigation/menubar")]
    MenubarPage {},
    #[route("/navigation/tree")]
    TreePage {},

    #[route("/overlay/dialog")]
    DialogPage {},
    #[route("/overlay/drawer")]
    DrawerPage {},
    #[route("/overlay/floating-window")]
    FloatingWindowPage {},
    #[route("/overlay/hover-card")]
    HoverCardPage {},
    #[route("/overlay/lightbox")]
    LightboxPage {},
    #[route("/overlay/menu")]
    MenuPage {},
    #[route("/overlay/modal")]
    ModalPage {},
    #[route("/overlay/overlay")]
    OverlayPage {},
    #[route("/overlay/popover")]
    PopoverPage {},
    #[route("/overlay/shortcut-help")]
    ShortcutHelpPage {},
    #[route("/overlay/spotlight")]
    SpotlightPage {},
    #[route("/overlay/tooltip")]
    TooltipPage {},

    #[route("/typography/blockquote")]
    BlockquotePage {},
    #[route("/typography/code")]
    CodePage {},
    #[route("/typography/code-block")]
    CodeBlockPage {},
    #[route("/typography/kbd")]
    KbdPage {},
    #[route("/typography/mark")]
    MarkPage {},
    #[route("/typography/text")]
    TextPage {},
    #[route("/typography/title")]
    TitlePage {},

    // Without it an unknown URL is a router parse error with no shell around it.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
fn App() -> Element {
    heading_focus::use_load_fragment();
    rsx! {
        document::Title { "Libero" }
        document::Link { rel: "icon", href: LOGO }
        LiberoProvider { formats: &Formats::GERMAN,
            Router::<Route> {}
            Notifications {}
        }
    }
}
