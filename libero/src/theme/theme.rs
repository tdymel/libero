use super::{
    ContainerDefaults, HexColor, Sizes, StackAxisDefaults, StackDefaults, TitleDefaults, TitleLevel,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub spacing: Sizes<u8>,
    pub stack: StackDefaults,
    pub container: ContainerDefaults,
    pub titles: TitleDefaults,
    pub primary: HexColor,
    pub secondary: HexColor,
    pub error: HexColor,
    pub warning: HexColor,
    pub info: HexColor,
    pub success: HexColor,
    pub grey: HexColor,
    pub black: HexColor,
    pub white: HexColor,
}

impl Theme {
    pub const DEFAULT: Theme = Theme::new(
        Sizes::new(4, 8, 12, 16, 20),
        StackDefaults::new(
            StackAxisDefaults::new("stretch", "flex-start", super::Size::Md, false),
            StackAxisDefaults::new("center", "flex-start", super::Size::Md, true),
        ),
        ContainerDefaults::new(super::Size::Lg, super::Size::Md),
        TitleDefaults::new(
            TitleLevel::new(
                "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif",
                "700",
                "2.125rem",
                "-0.01em",
                "1.3",
            ),
            TitleLevel::new(
                "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif",
                "700",
                "1.625rem",
                "-0.005em",
                "1.35",
            ),
            TitleLevel::new(
                "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif",
                "700",
                "1.375rem",
                "0em",
                "1.4",
            ),
            TitleLevel::new(
                "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif",
                "600",
                "1rem",
                "0em",
                "1.45",
            ),
            TitleLevel::new(
                "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif",
                "600",
                "0.875rem",
                "0em",
                "1.5",
            ),
            TitleLevel::new(
                "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif",
                "600",
                "0.75rem",
                "0em",
                "1.5",
            ),
        ),
        HexColor::new(0x228BE6),
        HexColor::new(0xE03131),
        HexColor::new(0xE03131),
        HexColor::new(0xF08C00),
        HexColor::new(0x228BE6),
        HexColor::new(0x2F9E44),
        HexColor::new(0x868E96),
        HexColor::new(0x000000),
        HexColor::new(0xFFFFFF),
    );

    pub const fn new(
        spacing: Sizes<u8>,
        stack: StackDefaults,
        container: ContainerDefaults,
        titles: TitleDefaults,
        primary: HexColor,
        secondary: HexColor,
        error: HexColor,
        warning: HexColor,
        info: HexColor,
        success: HexColor,
        grey: HexColor,
        black: HexColor,
        white: HexColor,
    ) -> Self {
        Self {
            spacing,
            stack,
            container,
            titles,
            primary,
            secondary,
            error,
            warning,
            info,
            success,
            grey,
            black,
            white,
        }
    }
}
