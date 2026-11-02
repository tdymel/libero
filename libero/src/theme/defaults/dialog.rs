use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const DIALOG_SIZE: SizeCss = SizeCss::new("--lsx-dialog-size-");
/// The width cap a dialog takes with no `size`; the prop writes `DIALOG_SIZE`'s override over it.
pub(crate) const DIALOG_DEFAULT_SIZE: CssVar = CssVar::new("--lsx-dialog-size");

str_enum! {
    /// When a `Dialog` in a modal fills the screen instead of centring.
    ///
    /// ```rust
    /// use libero::theme::DialogFullScreen;
    /// assert_eq!(DialogFullScreen::Phone.as_str(), "phone");
    /// ```
    pub enum DialogFullScreen {
        /// Centred at every width.
        #[default]
        Never = "never",
        /// Below the `sm` breakpoint, where a centred dialog leaves slivers of page.
        Phone = "phone",
        Always = "always",
    }
}

/// Theme defaults for `Dialog`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogDefaults {
    /// The width cap step when `size` is omitted.
    pub size: Size,
    /// `None`: `Paper`'s radius.
    pub radius: Option<Size>,
    pub sizes: Sizes<u16>,
    /// Inside a modal, when a dialog with no `full_screen` fills the screen.
    pub full_screen: DialogFullScreen,
}

impl DialogDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: None,
        sizes: Sizes::new(240, 300, 510, 600, 750, 900),
        full_screen: DialogFullScreen::Never,
    };
}

impl ToCssDeclarations for DialogDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.sizes.to_css_declarations(DIALOG_SIZE, "px");
        declarations.push(DIALOG_DEFAULT_SIZE.declare(DIALOG_SIZE.value(self.size)));
        declarations
    }
}
