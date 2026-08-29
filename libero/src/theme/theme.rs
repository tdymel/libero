use super::{
    ActionIconDefaults, AnchorDefaults, AnchorUnderline, AspectRatioDefaults, AutocompleteDefaults,
    ButtonDefaults, ButtonSizeLevel, CenterDefaults, CheckboxDefaults, ChipDefaults, ChipSizeLevel,
    CodeBlockDefaults, CodeDefaults, Color, ColorFieldDefaults, ColorPickerDefaults,
    ColorPickerSizeLevel, ColorShade, ColorSwatchDefaults, ColorValue, ComboboxDefaults,
    ComboboxSizeLevel, ContainerDefaults, DataListDefaults, DialogDefaults, DividerDefaults,
    DrawerDefaults, FieldDefaults, FieldSizeLevel, FileFieldDefaults, FileFieldVariant,
    FlexAxisDefaults, FlexDefaults, FloatDefaults, GridDefaults, HeaderDefaults, HexColor,
    IconDefaults, ImageDefaults, ImageFit, KbdDefaults, ListDefaults, MarkDefaults,
    NativeSelectDefaults, NavLinkDefaults, NumberFieldDefaults, OverlayDefaults, PinFieldDefaults,
    PinKind, Placement, PopoverDefaults, QrCodeDefaults, QrRobustness, RadioDefaults,
    ScrollAreaDefaults, ScrollAxis, ScrollbarSize, ScrollbarVisibility, SelectDefaults,
    SidebarDefaults, Size, Sizes, SliderDefaults, SliderSizeLevel, SplitterDefaults,
    SwitchDefaults, SwitchSizeLevel, TableDefaults, TabsDefaults, TabsSizeLevel, TextDefaults,
    TextFieldDefaults, TextSize, TextareaDefaults, TitleDefaults, TitleSize, TooltipDefaults,
    TooltipPlacement, TreeDefaults, ZIndexDefaults,
};

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
    pub center: CenterDefaults,
    pub container: ContainerDefaults,
    pub aspect_ratio: AspectRatioDefaults,
    pub float: FloatDefaults,
    pub overlay: OverlayDefaults,
    pub z_index: ZIndexDefaults,
    pub popover: PopoverDefaults,
    pub dialog: DialogDefaults,
    pub drawer: DrawerDefaults,
    pub sidebar: SidebarDefaults,
    pub divider: DividerDefaults,
    pub splitter: SplitterDefaults,
    pub scroll_area: ScrollAreaDefaults,
    pub button: ButtonDefaults,
    pub chip: ChipDefaults,
    pub switch: SwitchDefaults,
    pub checkbox: CheckboxDefaults,
    pub radio: RadioDefaults,
    /// Shared by every field: the typography of the slots stacked around a
    /// control. The frame numbers stay per component until R5.
    pub field: FieldDefaults,
    pub native_select: NativeSelectDefaults,
    pub select: SelectDefaults,
    pub multi_select: SelectDefaults,
    pub file_field: FileFieldDefaults,
    pub pin_field: PinFieldDefaults,
    pub text_field: TextFieldDefaults,
    pub textarea: TextareaDefaults,
    pub number_field: NumberFieldDefaults,
    pub combobox: ComboboxDefaults,
    pub autocomplete: AutocompleteDefaults,
    pub color_picker: ColorPickerDefaults,
    pub color_field: ColorFieldDefaults,
    pub color_swatch: ColorSwatchDefaults,
    pub slider: SliderDefaults,
    pub list: ListDefaults,
    pub data_list: DataListDefaults,
    pub table: TableDefaults,
    pub tabs: TabsDefaults,
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
    pub mark: MarkDefaults,
    pub kbd: KbdDefaults,
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
        center: CenterDefaults { inline: false },
        container: ContainerDefaults {
            size: Size::Lg,
            gutters: Size::Md,
        },
        aspect_ratio: AspectRatioDefaults { ratio: 1.0 },
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
        pin_field: PinFieldDefaults {
            length: 4,
            size: Size::Md,
            radius: Size::Sm,
            kind: PinKind::Numeric,
            gap: "8px",
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
        table: TableDefaults {
            padding_x: 12,
            padding_y: 10,
            font_size: 14,
            border_color: ColorValue::Shade(Color::Grey, ColorShade::S3),
            hover_color: ColorValue::Shade(Color::Grey, ColorShade::S1),
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
