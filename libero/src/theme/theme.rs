use super::{
    AccordionDefaults, AccordionSizeLevel, ActionIconDefaults, AlertDefaults, AnchorDefaults,
    AnchorUnderline, AspectRatioDefaults, AutocompleteDefaults, AvatarDefaults,
    AvatarGroupDefaults, BadgeDefaults, BadgeSizeLevel, BarPosition, BlockquoteDefaults,
    BlockquoteSizeLevel, BurgerDefaults, BurgerLabels, ButtonDefaults, ButtonSizeLevel,
    CalendarVariant, CarouselAlign, CarouselDefaults, CascaderDefaults, CenterDefaults,
    CheckboxDefaults, ChipDefaults, ChipSizeLevel, CodeBlockDefaults, CodeDefaults,
    CollapseDefaults, Color, ColorFieldDefaults, ColorPickerDefaults, ColorPickerSizeLevel,
    ColorShade, ColorSwatchDefaults, ColorValue, ComboboxDefaults, ComboboxSizeLevel,
    ContainerDefaults, DataListDefaults, DateDefaults, DateFieldDefaults, DatePickerDefaults,
    DatePickerSizeLevel, DialogDefaults, DividerDefaults, DrawerDefaults, FieldDefaults,
    FieldSizeLevel, FieldsetDefaults, FileFieldDefaults, FileFieldVariant, FlexAxisDefaults,
    FlexDefaults, FloatDefaults, FormDefaults, GridDefaults, HeaderDefaults, HexColor,
    IconDefaults, ImageDefaults, ImageFit, ImageListDefaults, ImageListVariant, IndicatorDefaults,
    IndicatorSizeLevel, KbdDefaults, ListDefaults, LoaderDefaults, LoaderVariant, MarkDefaults,
    MarqueeDefaults, NativeSelectDefaults, NavLinkDefaults, NumberFieldDefaults, OverlayDefaults,
    PaginationDefaults, PaginationLabels, PaperDefaults, PhoneFieldDefaults, PinFieldDefaults,
    PinKind, Placement, PopoverDefaults, ProgressBarDefaults, QrCodeDefaults, QrRobustness,
    RadioDefaults, ScrollAreaDefaults, ScrollAxis, ScrollbarSize, ScrollbarVisibility,
    SelectDefaults, SidebarDefaults, Size, Sizes, SkeletonDefaults, SliderDefaults,
    SliderSizeLevel, SplitterDefaults, StepLabelPosition, StepperDefaults, StepperSizeLevel,
    SwitchDefaults, SwitchSizeLevel, TIMELINE_BULLET_BACKGROUND_DEFAULT, TableDefaults,
    TabsDefaults, TabsSizeLevel, TagsFieldDefaults, TextDefaults, TextFieldDefaults, TextSize,
    TextareaDefaults, TimePickerDefaults, TimePickerVariant, TimelineAlign, TimelineDefaults,
    TitleDefaults, TitleSize, TooltipDefaults, TooltipPlacement, TreeDefaults, ZIndexDefaults,
};
use crate::components::ButtonVariant;

const SANS_FONT_FAMILY: &str = "ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Noto Sans', Ubuntu, Cantarell, 'Helvetica Neue', sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol', 'Noto Color Emoji'";
const MONO_FONT_FAMILY: &str = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";

// Deliberately not `Copy`, despite being built from `Copy` fields: at ~2KB a
// stray by-value use is a silent memcpy. Nothing needs it - `Theme::DEFAULT`
// is a `const` (so `Theme { ..Theme::DEFAULT }` still works) and `use_theme()`
// hands out `&'static Theme`.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub spacing: Sizes<u8>,
    pub radius: Sizes<u8>,
    /// Drop shadows, `xs` (resting) to `xxl` (a modal). M3's elevation levels.
    pub elevation: Sizes<&'static str>,
    pub flex: FlexDefaults,
    pub grid: GridDefaults,
    pub carousel: CarouselDefaults,
    pub center: CenterDefaults,
    pub container: ContainerDefaults,
    pub aspect_ratio: AspectRatioDefaults,
    pub collapse: CollapseDefaults,
    pub accordion: AccordionDefaults,
    pub float: FloatDefaults,
    pub overlay: OverlayDefaults,
    pub z_index: ZIndexDefaults,
    pub popover: PopoverDefaults,
    pub progress_bar: ProgressBarDefaults,
    pub paper: PaperDefaults,
    pub dialog: DialogDefaults,
    pub drawer: DrawerDefaults,
    pub sidebar: SidebarDefaults,
    pub divider: DividerDefaults,
    pub splitter: SplitterDefaults,
    pub scroll_area: ScrollAreaDefaults,
    pub blockquote: BlockquoteDefaults,
    pub burger: BurgerDefaults,
    pub button: ButtonDefaults,
    pub chip: ChipDefaults,
    pub badge: BadgeDefaults,
    pub alert: AlertDefaults,
    pub switch: SwitchDefaults,
    pub checkbox: CheckboxDefaults,
    pub radio: RadioDefaults,
    /// Shared by every field: the typography of the slots stacked around a
    /// control. The frame numbers stay per component until R5.
    pub field: FieldDefaults,
    pub form: FormDefaults,
    pub fieldset: FieldsetDefaults,
    pub native_select: NativeSelectDefaults,
    pub select: SelectDefaults,
    pub multi_select: SelectDefaults,
    pub cascader: CascaderDefaults,
    pub file_field: FileFieldDefaults,
    pub pin_field: PinFieldDefaults,
    pub phone_field: PhoneFieldDefaults,
    pub tags_field: TagsFieldDefaults,
    pub text_field: TextFieldDefaults,
    pub textarea: TextareaDefaults,
    pub number_field: NumberFieldDefaults,
    /// Names, first weekday and display format for every date and time
    /// component. The one place to translate them.
    pub date: DateDefaults,
    pub date_picker: DatePickerDefaults,
    pub date_field: DateFieldDefaults,
    pub time_picker: TimePickerDefaults,
    pub combobox: ComboboxDefaults,
    pub autocomplete: AutocompleteDefaults,
    pub color_picker: ColorPickerDefaults,
    pub color_field: ColorFieldDefaults,
    pub color_swatch: ColorSwatchDefaults,
    pub slider: SliderDefaults,
    pub list: ListDefaults,
    pub loader: LoaderDefaults,
    pub indicator: IndicatorDefaults,
    pub skeleton: SkeletonDefaults,
    pub marquee: MarqueeDefaults,
    pub data_list: DataListDefaults,
    pub table: TableDefaults,
    pub timeline: TimelineDefaults,
    pub tabs: TabsDefaults,
    pub stepper: StepperDefaults,
    pub tree: TreeDefaults,
    pub titles: TitleDefaults,
    pub texts: TextDefaults,
    pub tooltip: TooltipDefaults,
    pub code: CodeDefaults,
    pub code_block: CodeBlockDefaults,
    pub header: HeaderDefaults,
    pub icon: IconDefaults,
    pub action_icon: ActionIconDefaults,
    pub qr_code: QrCodeDefaults,
    pub image: ImageDefaults,
    pub image_list: ImageListDefaults,
    pub avatar: AvatarDefaults,
    pub avatar_group: AvatarGroupDefaults,
    pub mark: MarkDefaults,
    pub kbd: KbdDefaults,
    pub pagination: PaginationDefaults,
    /// Every string `Pagination` shows a reader. Swapped whole for a locale.
    pub pagination_labels: PaginationLabels,
    pub nav_link: NavLinkDefaults,
    pub anchor: AnchorDefaults,
    pub primary: HexColor,
    pub secondary: HexColor,
    pub error: HexColor,
    pub warning: HexColor,
    pub info: HexColor,
    pub success: HexColor,
    /// Text-dark neutral: the label and outline color of a `neutral` control.
    pub neutral: HexColor,
    pub grey: HexColor,
    pub black: HexColor,
    pub white: HexColor,
    pub font_smoothing: bool,
}

impl Theme {
    pub const DEFAULT: Theme = Theme {
        spacing: Sizes::new(4, 8, 12, 16, 20, 24),
        radius: Sizes::new(2, 4, 8, 16, 32, 64),
        elevation: Sizes::new(
            "0 1px 2px rgba(0, 0, 0, 0.10), 0 1px 3px rgba(0, 0, 0, 0.06)",
            "0 1px 2px rgba(0, 0, 0, 0.10), 0 2px 6px rgba(0, 0, 0, 0.10)",
            "0 2px 4px rgba(0, 0, 0, 0.10), 0 4px 10px rgba(0, 0, 0, 0.12)",
            "0 4px 8px rgba(0, 0, 0, 0.10), 0 8px 20px rgba(0, 0, 0, 0.14)",
            "0 8px 16px rgba(0, 0, 0, 0.12), 0 12px 32px rgba(0, 0, 0, 0.18)",
            "0 12px 24px rgba(0, 0, 0, 0.14), 0 20px 48px rgba(0, 0, 0, 0.22)",
        ),
        flex: FlexDefaults {
            column: FlexAxisDefaults {
                align: "stretch",
                justify: "flex-start",
                spacing: Size::Md,
                wrap: false,
            },
            row: FlexAxisDefaults {
                align: "center",
                justify: "flex-start",
                spacing: Size::Md,
                wrap: false,
            },
        },
        grid: GridDefaults {
            gap: Size::Md,
            zone_gap: Size::Md,
            row_unit: 2,
        },
        carousel: CarouselDefaults {
            per_view: 1.0,
            gap: Size::Md,
            align: CarouselAlign::Start,
            radius: Size::Sm,
            controls: true,
            indicators: false,
            control_size: "28px",
            controls_offset: Size::Sm,
            indicator_length: "24px",
            indicator_current_length: "40px",
            indicator_thickness: "5px",
            indicators_gap: "8px",
            indicator_color: ColorValue::Shade(Color::Grey, ColorShade::S6),
            indicator_current_color: ColorValue::Shade(Color::Primary, ColorShade::S6),
            autoplay_delay: 4000,
            label: "Carousel",
            previous_label: "Previous slide",
            next_label: "Next slide",
            indicator_label: "Go to slide {n}",
            slide_label: "{n} of {m}",
            status_label: "Slide {n} of {m}",
            pause_label: "Pause slideshow",
            play_label: "Play slideshow",
        },
        center: CenterDefaults { inline: false },
        container: ContainerDefaults {
            size: Size::Lg,
            gutters: Size::Md,
        },
        aspect_ratio: AspectRatioDefaults { ratio: 1.0 },
        collapse: CollapseDefaults {
            duration: 200,
            easing: "ease",
            animate_opacity: true,
        },
        accordion: AccordionDefaults {
            size: Size::Md,
            sizes: Sizes::new(
                AccordionSizeLevel {
                    font_size: "12px",
                    padding_x: "8px",
                    padding_y: "6px",
                    chevron: "14px",
                },
                AccordionSizeLevel {
                    font_size: "14px",
                    padding_x: "12px",
                    padding_y: "8px",
                    chevron: "16px",
                },
                AccordionSizeLevel {
                    font_size: "16px",
                    padding_x: "16px",
                    padding_y: "12px",
                    chevron: "18px",
                },
                AccordionSizeLevel {
                    font_size: "18px",
                    padding_x: "20px",
                    padding_y: "14px",
                    chevron: "20px",
                },
                AccordionSizeLevel {
                    font_size: "20px",
                    padding_x: "24px",
                    padding_y: "16px",
                    chevron: "22px",
                },
                AccordionSizeLevel {
                    font_size: "24px",
                    padding_x: "28px",
                    padding_y: "20px",
                    chevron: "24px",
                },
            ),
            border_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            hover_color: ColorValue::Shade(Color::Grey, ColorShade::S1),
            chevron_duration: 150,
        },
        float: FloatDefaults {
            offset_x: "0px",
            offset_y: "0px",
            placement: Placement::CenterCenter,
        },
        overlay: OverlayDefaults {
            opacity: 0.6,
            blur: "none",
        },
        z_index: ZIndexDefaults {
            header: 100,
            float: 200,
            overlay: 300,
            modal: 1000,
            modal_step: 10,
            popover: 2000,
        },
        popover: PopoverDefaults {
            gap: 4.0,
            padding: 8.0,
        },
        paper: PaperDefaults {
            radius: Size::Md,
            shadow: Size::Sm,
            background: "#fff",
            contrast: ColorValue::Shade(Color::Black, ColorShade::S1),
            border_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
        },
        dialog: DialogDefaults {
            size: Sizes::new(240, 300, 510, 600, 750, 900),
        },
        drawer: DrawerDefaults {
            size: Sizes::new(200, 240, 280, 320, 400, 480),
        },
        sidebar: SidebarDefaults {
            size: Sizes::new(200, 240, 280, 320, 400, 480),
        },
        divider: DividerDefaults {
            spacing: None,
            thickness: Sizes::new(1, 2, 3, 4, 5, 6),
        },
        splitter: SplitterDefaults {
            size: Size::Sm,
            divider_size: Sizes::new(1, 1, 2, 3, 4, 6),
            hit_size: Sizes::new(10, 10, 12, 14, 16, 20),
            min_size: 10.0,
            step: 1.0,
            big_step: 10.0,
        },
        scroll_area: ScrollAreaDefaults {
            scrollbars: ScrollAxis::Vertical,
            visibility: ScrollbarVisibility::Always,
            size: ScrollbarSize::Thin,
            overscan: 4,
        },
        blockquote: BlockquoteDefaults {
            size: Size::Md,
            radius: Size::Sm,
            color: Color::Primary,
            cite_opacity: "0.65",
            sizes: Sizes::new(
                BlockquoteSizeLevel {
                    padding_y: "0.5rem",
                    padding_x: "0.75rem",
                    border_width: "2px",
                },
                BlockquoteSizeLevel {
                    padding_y: "0.75rem",
                    padding_x: "1rem",
                    border_width: "2px",
                },
                BlockquoteSizeLevel {
                    padding_y: "1rem",
                    padding_x: "1.5rem",
                    border_width: "3px",
                },
                BlockquoteSizeLevel {
                    padding_y: "1.25rem",
                    padding_x: "2rem",
                    border_width: "3px",
                },
                BlockquoteSizeLevel {
                    padding_y: "1.5rem",
                    padding_x: "2.5rem",
                    border_width: "4px",
                },
                BlockquoteSizeLevel {
                    padding_y: "2rem",
                    padding_x: "3rem",
                    border_width: "5px",
                },
            ),
        },
        burger: BurgerDefaults {
            size: Size::Md,
            // `xs`..`xl` are Mantine's own five, adopted exactly. `xxl`
            // continues the ramp past the widest step it offers.
            sizes: Sizes::new(12, 18, 24, 34, 42, 52),
            transition_duration: "300ms",
            transition_timing: "ease",
            labels: BurgerLabels::ENGLISH,
        },
        button: ButtonDefaults {
            size: Size::Md,
            radius: Size::Md,
            sizes: Sizes::new(
                ButtonSizeLevel {
                    font_size: "0.75rem",
                    height: "30px",
                    padding_x: "10px",
                },
                ButtonSizeLevel {
                    font_size: "0.875rem",
                    height: "36px",
                    padding_x: "14px",
                },
                ButtonSizeLevel {
                    font_size: "1rem",
                    height: "42px",
                    padding_x: "18px",
                },
                ButtonSizeLevel {
                    font_size: "1.125rem",
                    height: "50px",
                    padding_x: "22px",
                },
                ButtonSizeLevel {
                    font_size: "1.25rem",
                    height: "60px",
                    padding_x: "28px",
                },
                ButtonSizeLevel {
                    font_size: "1.375rem",
                    height: "72px",
                    padding_x: "34px",
                },
            ),
        },
        chip: ChipDefaults {
            size: Size::Md,
            radius: Size::Xl,
            sizes: Sizes::new(
                ChipSizeLevel {
                    font_size: "0.6875rem",
                    height: "20px",
                    padding_x: "8px",
                },
                ChipSizeLevel {
                    font_size: "0.75rem",
                    height: "24px",
                    padding_x: "10px",
                },
                ChipSizeLevel {
                    font_size: "0.8125rem",
                    height: "28px",
                    padding_x: "12px",
                },
                ChipSizeLevel {
                    font_size: "0.875rem",
                    height: "32px",
                    padding_x: "14px",
                },
                ChipSizeLevel {
                    font_size: "0.9375rem",
                    height: "36px",
                    padding_x: "16px",
                },
                ChipSizeLevel {
                    font_size: "1rem",
                    height: "40px",
                    padding_x: "18px",
                },
            ),
        },
        alert: AlertDefaults {
            variant: ButtonVariant::Tonal,
            // Not the primary colour: severity is the caller's to state, and
            // the brand colour on an alert reads as decoration.
            color: "info",
            // The same step as `paper.radius` - an alert is a surface.
            radius: Size::Md,
            padding: Size::Md,
            gap: Size::Md,
            body_gap: Size::Xs,
            icon_size: "20px",
            close_label: "Close",
        },
        badge: BadgeDefaults {
            size: Size::Md,
            // Off the 2..64px radius scale on purpose: a badge is a pill at
            // every height, which no fixed step gives.
            radius: "9999px",
            text_transform: "uppercase",
            letter_spacing: "0.25px",
            font_weight: "700",
            // `xs`..`xl` are Mantine's own numbers; `xxl` continues the ramp,
            // since our scale has a sixth step and theirs does not. Font sizes
            // are rem so they follow a reader's own text size, the way
            // `Button` and `Chip` already do; the boxes around them are px.
            sizes: Sizes::new(
                BadgeSizeLevel {
                    font_size: "0.5625rem",
                    height: "16px",
                    padding_x: "6px",
                },
                BadgeSizeLevel {
                    font_size: "0.625rem",
                    height: "18px",
                    padding_x: "8px",
                },
                BadgeSizeLevel {
                    font_size: "0.6875rem",
                    height: "20px",
                    padding_x: "10px",
                },
                BadgeSizeLevel {
                    font_size: "0.8125rem",
                    height: "26px",
                    padding_x: "12px",
                },
                BadgeSizeLevel {
                    font_size: "1rem",
                    height: "32px",
                    padding_x: "16px",
                },
                BadgeSizeLevel {
                    font_size: "1.125rem",
                    height: "38px",
                    padding_x: "20px",
                },
            ),
        },
        progress_bar: ProgressBarDefaults {
            size: Size::Md,
            radius: Size::Xl,
            track_shade: ColorShade::S2,
            transition: "100ms",
            // Track heights. A bar is a full pill once the radius reaches half
            // of these, which is why most of the radius scale is inert here -
            // see `ProgressBarDefaults::radius`.
            sizes: Sizes::new("3px", "5px", "8px", "12px", "16px", "20px"),
        },
        switch: SwitchDefaults {
            size: Size::Md,
            radius: Size::Xl,
            sizes: Sizes::new(
                SwitchSizeLevel {
                    track_width: "30px",
                    track_height: "16px",
                    thumb_size: "12px",
                },
                SwitchSizeLevel {
                    track_width: "34px",
                    track_height: "18px",
                    thumb_size: "14px",
                },
                SwitchSizeLevel {
                    track_width: "42px",
                    track_height: "22px",
                    thumb_size: "18px",
                },
                SwitchSizeLevel {
                    track_width: "50px",
                    track_height: "26px",
                    thumb_size: "22px",
                },
                SwitchSizeLevel {
                    track_width: "58px",
                    track_height: "30px",
                    thumb_size: "26px",
                },
                SwitchSizeLevel {
                    track_width: "66px",
                    track_height: "34px",
                    thumb_size: "30px",
                },
            ),
        },
        checkbox: CheckboxDefaults {
            size: Size::Md,
            radius: Size::Sm,
            sizes: Sizes::new("14px", "16px", "18px", "20px", "22px", "24px"),
        },
        radio: RadioDefaults {
            size: Size::Md,
            sizes: Sizes::new("14px", "16px", "18px", "20px", "22px", "24px"),
        },
        field: FieldDefaults {
            gap: "4px",
            frame_gap: "8px",
            sizes: Sizes::new(
                FieldSizeLevel {
                    label_font_size: "0.6875rem",
                    caption_font_size: "0.6875rem",
                    font_size: "0.75rem",
                    height: "28px",
                    padding_y: "4px",
                    padding_x: "8px",
                },
                FieldSizeLevel {
                    label_font_size: "0.75rem",
                    caption_font_size: "0.75rem",
                    font_size: "0.8125rem",
                    height: "32px",
                    padding_y: "5px",
                    padding_x: "10px",
                },
                FieldSizeLevel {
                    label_font_size: "0.8125rem",
                    caption_font_size: "0.75rem",
                    font_size: "0.875rem",
                    height: "36px",
                    padding_y: "6px",
                    padding_x: "12px",
                },
                FieldSizeLevel {
                    label_font_size: "0.875rem",
                    caption_font_size: "0.8125rem",
                    font_size: "0.9375rem",
                    height: "40px",
                    padding_y: "7px",
                    padding_x: "14px",
                },
                FieldSizeLevel {
                    label_font_size: "0.9375rem",
                    caption_font_size: "0.875rem",
                    font_size: "1rem",
                    height: "44px",
                    padding_y: "8px",
                    padding_x: "16px",
                },
                FieldSizeLevel {
                    label_font_size: "1rem",
                    caption_font_size: "0.9375rem",
                    font_size: "1.0625rem",
                    height: "48px",
                    padding_y: "9px",
                    padding_x: "18px",
                },
            ),
            card_padding: Sizes::new("8px", "10px", "12px", "14px", "16px", "18px"),
        },
        native_select: NativeSelectDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        select: SelectDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        multi_select: SelectDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        file_field: FileFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
            variant: FileFieldVariant::Input,
            clearable: true,
            // Three or four lines of prompt at each step, and always taller
            // than the one-line control it replaces.
            dropzone_heights: Sizes::new("72px", "88px", "104px", "124px", "148px", "176px"),
        },
        form: FormDefaults { gap: "16px" },
        fieldset: FieldsetDefaults { gap: "12px" },
        pin_field: PinFieldDefaults {
            length: 4,
            size: Size::Md,
            radius: Size::Sm,
            kind: PinKind::Numeric,
            gap: "8px",
        },
        phone_field: PhoneFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
            country: "US",
        },
        tags_field: TagsFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        cascader: CascaderDefaults {
            size: Size::Md,
            radius: Size::Sm,
            column_width: "220px",
        },
        text_field: TextFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        textarea: TextareaDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        number_field: NumberFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        date: DateDefaults::ENGLISH,
        date_picker: DatePickerDefaults {
            size: Size::Md,
            calendar: CalendarVariant::Full,
            days: 7,
            sizes: Sizes::new(
                DatePickerSizeLevel {
                    day_size: "28px",
                    font_size: "12px",
                },
                DatePickerSizeLevel {
                    day_size: "32px",
                    font_size: "13px",
                },
                DatePickerSizeLevel {
                    day_size: "36px",
                    font_size: "14px",
                },
                DatePickerSizeLevel {
                    day_size: "40px",
                    font_size: "16px",
                },
                DatePickerSizeLevel {
                    day_size: "44px",
                    font_size: "18px",
                },
                DatePickerSizeLevel {
                    day_size: "48px",
                    font_size: "20px",
                },
            ),
        },
        date_field: DateFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
            close_on_change: true,
        },
        time_picker: TimePickerDefaults {
            size: Size::Md,
            variant: TimePickerVariant::Analog,
            step: 5,
        },
        autocomplete: AutocompleteDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        // Mantine's scale, with a `xxl` step continued from it.
        color_picker: ColorPickerDefaults {
            size: Size::Md,
            swatches_per_row: None,
            radius: Size::Xxl,
            sizes: Sizes::new(
                ColorPickerSizeLevel {
                    width: "180px",
                    saturation_height: "100px",
                    thumb_size: "8px",
                    preview_size: "26px",
                    spacing: "4px",
                    swatch_size: "20px",
                },
                ColorPickerSizeLevel {
                    width: "200px",
                    saturation_height: "110px",
                    thumb_size: "12px",
                    preview_size: "34px",
                    spacing: "6px",
                    swatch_size: "22px",
                },
                ColorPickerSizeLevel {
                    width: "240px",
                    saturation_height: "120px",
                    thumb_size: "16px",
                    preview_size: "42px",
                    spacing: "8px",
                    swatch_size: "26px",
                },
                ColorPickerSizeLevel {
                    width: "280px",
                    saturation_height: "140px",
                    thumb_size: "20px",
                    preview_size: "50px",
                    spacing: "10px",
                    swatch_size: "30px",
                },
                ColorPickerSizeLevel {
                    width: "320px",
                    saturation_height: "160px",
                    thumb_size: "22px",
                    preview_size: "54px",
                    spacing: "12px",
                    swatch_size: "34px",
                },
                ColorPickerSizeLevel {
                    width: "360px",
                    saturation_height: "180px",
                    thumb_size: "26px",
                    preview_size: "62px",
                    spacing: "14px",
                    swatch_size: "38px",
                },
            ),
        },
        color_field: ColorFieldDefaults {
            size: Size::Md,
            radius: Size::Sm,
            with_preview: true,
            with_eye_dropper: true,
            fix_on_blur: true,
            close_on_swatch_click: false,
        },
        color_swatch: ColorSwatchDefaults {
            size: Size::Md,
            radius: Size::Xxl,
            sizes: Sizes::new("16px", "20px", "28px", "36px", "44px", "52px"),
        },
        combobox: ComboboxDefaults {
            size: Size::Md,
            radius: Size::Sm,
            max_dropdown_height: "260px",
            sizes: Sizes::new(
                ComboboxSizeLevel {
                    font_size: "0.75rem",
                    row_height: 28.0,
                    padding_x: "8px",
                },
                ComboboxSizeLevel {
                    font_size: "0.8125rem",
                    row_height: 32.0,
                    padding_x: "10px",
                },
                ComboboxSizeLevel {
                    font_size: "0.875rem",
                    row_height: 36.0,
                    padding_x: "12px",
                },
                ComboboxSizeLevel {
                    font_size: "0.9375rem",
                    row_height: 40.0,
                    padding_x: "14px",
                },
                ComboboxSizeLevel {
                    font_size: "1rem",
                    row_height: 44.0,
                    padding_x: "16px",
                },
                ComboboxSizeLevel {
                    font_size: "1.0625rem",
                    row_height: 48.0,
                    padding_x: "18px",
                },
            ),
            labels: crate::theme::ComboboxLabels::ENGLISH,
        },
        slider: SliderDefaults {
            size: Size::Md,
            sizes: Sizes::new(
                SliderSizeLevel {
                    track_size: "2px",
                    thumb_size: "12px",
                    font_size: "0.6875rem",
                },
                SliderSizeLevel {
                    track_size: "3px",
                    thumb_size: "14px",
                    font_size: "0.75rem",
                },
                SliderSizeLevel {
                    track_size: "4px",
                    thumb_size: "16px",
                    font_size: "0.8125rem",
                },
                SliderSizeLevel {
                    track_size: "6px",
                    thumb_size: "20px",
                    font_size: "0.875rem",
                },
                SliderSizeLevel {
                    track_size: "8px",
                    thumb_size: "24px",
                    font_size: "0.9375rem",
                },
                SliderSizeLevel {
                    track_size: "10px",
                    thumb_size: "28px",
                    font_size: "1rem",
                },
            ),
            step: 1.0,
            big_step: 10.0,
        },
        list: ListDefaults {
            size: Size::Md,
            gap: Sizes::new(4, 8, 12, 16, 20, 24),
            indent: Sizes::new(8, 12, 16, 20, 24, 28),
        },
        loader: LoaderDefaults {
            variant: LoaderVariant::Oval,
            size: Size::Md,
            color: Color::Primary,
            // `xs`..`xl` are Mantine's; `xxl` continues the ramp at the same
            // step, since our scale has a sixth level and theirs does not.
            sizes: Sizes::new("18px", "22px", "36px", "44px", "58px", "72px"),
        },
        indicator: IndicatorDefaults {
            size: Size::Md,
            color: Color::Error,
            radius: "9999px",
            max: 99,
            border_width: "2px",
            processing_duration: "1000ms",
            sizes: Sizes::new(
                IndicatorSizeLevel {
                    size: "6px",
                    font_size: "8px",
                },
                IndicatorSizeLevel {
                    size: "8px",
                    font_size: "9px",
                },
                IndicatorSizeLevel {
                    size: "10px",
                    font_size: "10px",
                },
                IndicatorSizeLevel {
                    size: "14px",
                    font_size: "11px",
                },
                IndicatorSizeLevel {
                    size: "18px",
                    font_size: "12px",
                },
                IndicatorSizeLevel {
                    size: "22px",
                    font_size: "14px",
                },
            ),
        },
        skeleton: SkeletonDefaults {
            radius: Size::Sm,
            color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            duration: "1500ms",
        },
        marquee: MarqueeDefaults {
            duration: 40_000,
            repeat: 4,
            gap: Size::Md,
            pause_on_hover: false,
            pause_control: true,
            fade_edges: false,
            fade_size: "5%",
            pause_label: "Pause",
        },
        table: TableDefaults {
            padding_x: 12,
            padding_y: 10,
            font_size: 14,
            border_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            hover_color: ColorValue::Shade(Color::Grey, ColorShade::S1),
        },
        timeline: TimelineDefaults {
            align: TimelineAlign::Left,
            color: ColorValue::Shade(Color::Primary, ColorShade::S6),
            line_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            bullet_background: TIMELINE_BULLET_BACKGROUND_DEFAULT,
            radius: Size::Xl,
            bullet_size: Size::Md,
            bullet_sizes: Sizes::new(12, 16, 20, 24, 28, 32),
            line_width: 2,
            gap: Size::Xl,
            gaps: Sizes::new(12, 16, 24, 32, 40, 48),
        },
        tabs: TabsDefaults {
            size: Size::Md,
            sizes: Sizes::new(
                TabsSizeLevel {
                    font_size: "12px",
                    padding_x: "10px",
                    padding_y: "6px",
                    indicator: "2px",
                    icon_gap: "6px",
                },
                TabsSizeLevel {
                    font_size: "13px",
                    padding_x: "12px",
                    padding_y: "8px",
                    indicator: "2px",
                    icon_gap: "6px",
                },
                TabsSizeLevel {
                    font_size: "14px",
                    padding_x: "16px",
                    padding_y: "10px",
                    indicator: "2px",
                    icon_gap: "8px",
                },
                TabsSizeLevel {
                    font_size: "16px",
                    padding_x: "20px",
                    padding_y: "12px",
                    indicator: "3px",
                    icon_gap: "10px",
                },
                TabsSizeLevel {
                    font_size: "18px",
                    padding_x: "24px",
                    padding_y: "14px",
                    indicator: "3px",
                    icon_gap: "12px",
                },
                TabsSizeLevel {
                    font_size: "20px",
                    padding_x: "28px",
                    padding_y: "16px",
                    indicator: "4px",
                    icon_gap: "14px",
                },
            ),
            border_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            hover_color: ColorValue::Shade(Color::Grey, ColorShade::S1),
        },
        stepper: StepperDefaults {
            size: Size::Md,
            sizes: Sizes::new(
                StepperSizeLevel {
                    marker: "24px",
                    font_size: "12px",
                    description_font_size: "11px",
                    gap: "6px",
                    spacing: "8px",
                },
                StepperSizeLevel {
                    marker: "28px",
                    font_size: "13px",
                    description_font_size: "12px",
                    gap: "8px",
                    spacing: "12px",
                },
                StepperSizeLevel {
                    marker: "32px",
                    font_size: "14px",
                    description_font_size: "12px",
                    gap: "10px",
                    spacing: "16px",
                },
                StepperSizeLevel {
                    marker: "36px",
                    font_size: "16px",
                    description_font_size: "14px",
                    gap: "12px",
                    spacing: "20px",
                },
                StepperSizeLevel {
                    marker: "40px",
                    font_size: "18px",
                    description_font_size: "16px",
                    gap: "14px",
                    spacing: "24px",
                },
                StepperSizeLevel {
                    marker: "48px",
                    font_size: "20px",
                    description_font_size: "18px",
                    gap: "16px",
                    spacing: "28px",
                },
            ),
            label_position: StepLabelPosition::Side,
            color: ColorValue::Shade(Color::Primary, ColorShade::S6),
            pending_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            error_color: ColorValue::Shade(Color::Error, ColorShade::S6),
            connector_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            description_color: ColorValue::Shade(Color::Grey, ColorShade::S7),
            line_width: 2,
            content_padding: Size::Md,
            completed_label: "Completed",
            error_label: "Error",
        },
        data_list: DataListDefaults {
            size: Size::Md,
            gap: Sizes::new(6, 8, 12, 16, 20, 24),
        },
        tree: TreeDefaults { size: Size::Md },
        titles: TitleDefaults {
            font_family: SANS_FONT_FAMILY,
            sizes: Sizes::new(
                TitleSize {
                    font_weight: "400",
                    font_size: "0.75rem",
                    letter_spacing: "0em",
                    line_height: "1.5",
                }, // xs (h6)
                TitleSize {
                    font_weight: "400",
                    font_size: "0.875rem",
                    letter_spacing: "0em",
                    line_height: "1.5",
                }, // sm (h5)
                TitleSize {
                    font_weight: "400",
                    font_size: "1rem",
                    letter_spacing: "0em",
                    line_height: "1.45",
                }, // md (h4)
                TitleSize {
                    font_weight: "400",
                    font_size: "1.375rem",
                    letter_spacing: "0em",
                    line_height: "1.4",
                }, // lg (h3)
                TitleSize {
                    font_weight: "400",
                    font_size: "1.625rem",
                    letter_spacing: "-0.005em",
                    line_height: "1.35",
                }, // xl (h2)
                TitleSize {
                    font_weight: "400",
                    font_size: "2.125rem",
                    letter_spacing: "-0.01em",
                    line_height: "1.3",
                }, // xxl (h1, default)
            ),
        },
        texts: TextDefaults {
            font_family: SANS_FONT_FAMILY,
            sizes: Sizes::new(
                TextSize {
                    font_weight: "400",
                    font_size: "0.75rem",
                    letter_spacing: "0em",
                    line_height: "1.4",
                }, // xs
                TextSize {
                    font_weight: "400",
                    font_size: "0.875rem",
                    letter_spacing: "0em",
                    line_height: "1.45",
                }, // sm
                TextSize {
                    font_weight: "400",
                    font_size: "1rem",
                    letter_spacing: "0em",
                    line_height: "1.5",
                }, // md (default)
                TextSize {
                    font_weight: "400",
                    font_size: "1.125rem",
                    letter_spacing: "0em",
                    line_height: "1.55",
                }, // lg
                TextSize {
                    font_weight: "400",
                    font_size: "1.25rem",
                    letter_spacing: "0em",
                    line_height: "1.6",
                }, // xl
                TextSize {
                    font_weight: "400",
                    font_size: "1.375rem",
                    letter_spacing: "0em",
                    line_height: "1.65",
                }, // xxl
            ),
        },
        tooltip: TooltipDefaults {
            placement: TooltipPlacement::Top,
            gap: Size::Xs,
            size: Size::Sm,
            open_delay: 0,
            close_delay: 0,
            duration: 150,
            font_size: Sizes::new(10, 12, 13, 14, 16, 18),
            background: "#1f2328",
            color: "#ffffff",
        },
        code: CodeDefaults {
            font_family: MONO_FONT_FAMILY,
            tok_keyword: "#cf222e",
            tok_string: "#0a3069",
            tok_comment: "#6e7781",
            tok_number: "#0550ae",
            tok_constant: "#0550ae",
            tok_function: "#8250df",
            tok_type: "#953800",
            tok_tag: "#116329",
            tok_attribute: "#0969da",
            tok_heading: "#cf222e",
        },
        code_block: CodeBlockDefaults {
            background: "#f6f8fa",
            border: "#d0d7de",
            muted_text: "#57606a",
            line_number: "#8c959f",
            copy_hover_background: "rgba(31, 35, 40, 0.08)",
            copy_hover_text: "#1f2328",
        },
        header: HeaderDefaults {
            height: Sizes::new(48, 56, 64, 72, 80, 88),
        },
        icon: IconDefaults {
            size: Sizes::new(16, 20, 24, 32, 40, 48),
        },
        action_icon: ActionIconDefaults {
            size: Size::Md,
            radius: Size::Sm,
        },
        qr_code: QrCodeDefaults {
            background: "#FFFFFF",
            foreground: "#000000",
            robustness: QrRobustness::Medium,
        },
        image: ImageDefaults {
            fit: ImageFit::Cover,
            radius: "0",
        },
        image_list: ImageListDefaults {
            cols: 2,
            variant: ImageListVariant::Standard,
            gap: Size::Xs,
            radius: Size::Sm,
            bar_position: BarPosition::Bottom,
            bar_background: "linear-gradient(to top, rgba(0,0,0,0.72), rgba(0,0,0,0.36) 70%, transparent)",
            bar_background_top: "linear-gradient(to bottom, rgba(0,0,0,0.72), rgba(0,0,0,0.36) 70%, transparent)",
            bar_color: "#fff",
            bar_padding: Size::Sm,
        },
        avatar: AvatarDefaults {
            size: Size::Md,
            // A circle. Not `Size::Xl` (64px), which is a rounded square on
            // the two largest avatars and a circle on the two smallest.
            radius: "9999px",
            // Mantine's scale, plus an `xxl` continuing its steps.
            size_scale: Sizes::new(20, 28, 38, 56, 84, 120),
            font_size: Sizes::new(8, 11, 15, 22, 34, 48),
        },
        avatar_group: AvatarGroupDefaults {
            spacing: Size::Sm,
            ring: "2px",
        },
        mark: MarkDefaults {
            color: Color::Warning,
        },
        kbd: KbdDefaults {
            font_size: Sizes::new(10, 12, 14, 16, 20, 24),
            font_family: MONO_FONT_FAMILY,
            background: "#f6f8fa",
            border: "#d0d7de",
            color: "#57606a",
        },
        pagination: PaginationDefaults {
            size: Size::Md,
            radius: Size::Sm,
            color: Color::Primary,
            siblings: 1,
            boundaries: 1,
            gap: Size::Xs,
            // Mantine's control scale, plus an `xxl` continuing its steps.
            control_size: Sizes::new(22, 26, 32, 38, 44, 52),
            font_size: Sizes::new(11, 12, 14, 16, 18, 20),
            border: ColorValue::Shade(Color::Grey, ColorShade::S4),
        },
        pagination_labels: PaginationLabels::ENGLISH,
        nav_link: NavLinkDefaults {
            color: Color::Primary,
        },
        anchor: AnchorDefaults {
            size: Size::Md,
            underline: AnchorUnderline::Hover,
            color: Color::Primary,
        },
        primary: HexColor::new(0x228BE6),
        secondary: HexColor::new(0x7950F2),
        error: HexColor::new(0xFA5252),
        warning: HexColor::new(0xFAB005),
        info: HexColor::new(0x15AABF),
        success: HexColor::new(0x40C057),
        neutral: HexColor::new(0x373A3C),
        grey: HexColor::new(0x868E96),
        black: HexColor::new(0x000000),
        white: HexColor::new(0xFFFFFF),
        font_smoothing: true,
    };
}
