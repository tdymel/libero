use super::{
    ActionIconDefaults, AnchorDefaults, AnchorUnderline, AspectRatioDefaults, ButtonDefaults,
    ButtonSizeLevel, CenterDefaults,
    CodeDefaults, Color, ContainerDefaults, DataListDefaults, DialogDefaults, DividerDefaults,
    DrawerDefaults, FlexAxisDefaults, FlexDefaults, FloatDefaults, HeaderDefaults, HexColor,
    IconDefaults, ImageDefaults, ImageFit, KbdDefaults, ListDefaults, MarkDefaults,
    NavLinkDefaults, OverlayDefaults,
    Placement, QrCodeDefaults, QrRobustness, ScrollAreaDefaults, ScrollAxis, ScrollbarSize,
    ScrollbarVisibility, SelectDefaults, SelectSizeLevel, Size, Sizes, SplitterDefaults,
    TextDefaults, TextSize, TitleDefaults, TitleSize, TreeDefaults, ZIndexDefaults,
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
    pub flex: FlexDefaults,
    pub center: CenterDefaults,
    pub container: ContainerDefaults,
    pub aspect_ratio: AspectRatioDefaults,
    pub float: FloatDefaults,
    pub overlay: OverlayDefaults,
    pub z_index: ZIndexDefaults,
    pub dialog: DialogDefaults,
    pub drawer: DrawerDefaults,
    pub divider: DividerDefaults,
    pub splitter: SplitterDefaults,
    pub scroll_area: ScrollAreaDefaults,
    pub button: ButtonDefaults,
    pub select: SelectDefaults,
    pub list: ListDefaults,
    pub data_list: DataListDefaults,
    pub tree: TreeDefaults,
    pub titles: TitleDefaults,
    pub texts: TextDefaults,
    pub code: CodeDefaults,
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
    pub grey: HexColor,
    pub black: HexColor,
    pub white: HexColor,
    pub font_smoothing: bool,
}

impl Theme {
    pub const DEFAULT: Theme = Theme {
        spacing: Sizes::new(4, 8, 12, 16, 20, 24),
        radius: Sizes::new(2, 4, 8, 16, 32, 64),
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
        },
        dialog: DialogDefaults {
            size: Sizes::new(240, 300, 510, 600, 750, 900),
        },
        drawer: DrawerDefaults {
            size: Sizes::new(200, 240, 280, 320, 400, 480),
        },
        divider: DividerDefaults { spacing: None },
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
        select: SelectDefaults {
            size: Size::Md,
            radius: Size::Sm,
            sizes: Sizes::new(
                SelectSizeLevel {
                    font_size: "0.75rem",
                    height: "28px",
                    padding_x: "8px",
                },
                SelectSizeLevel {
                    font_size: "0.8125rem",
                    height: "32px",
                    padding_x: "10px",
                },
                SelectSizeLevel {
                    font_size: "0.875rem",
                    height: "36px",
                    padding_x: "12px",
                },
                SelectSizeLevel {
                    font_size: "0.9375rem",
                    height: "40px",
                    padding_x: "14px",
                },
                SelectSizeLevel {
                    font_size: "1rem",
                    height: "44px",
                    padding_x: "16px",
                },
                SelectSizeLevel {
                    font_size: "1.0625rem",
                    height: "48px",
                    padding_x: "18px",
                },
            ),
        },
        list: ListDefaults {
            size: Size::Md,
            gap: Sizes::new(4, 8, 12, 16, 20, 24),
            indent: Sizes::new(8, 12, 16, 20, 24, 28),
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
        code: CodeDefaults {
            font_family: MONO_FONT_FAMILY,
            background: "#f6f8fa",
            border: "#d0d7de",
            muted_text: "#57606a",
            line_number: "#8c959f",
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
        grey: HexColor::new(0x868E96),
        black: HexColor::new(0x000000),
        white: HexColor::new(0xFFFFFF),
        font_smoothing: true,
    };
}
