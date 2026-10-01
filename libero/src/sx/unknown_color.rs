//! Todo 385: a typo in a theme colour name is valid CSS, so `color: "primry"`
//! paints nothing and says nothing. Debug builds say something.

use super::Property;
use crate::utils::warn;

/// The CSS named colours, system colours and CSS-wide keywords, lowercase.
const CSS_COLOR_KEYWORDS: &[&str] = &[
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "grey",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
    // System colours.
    "accentcolor",
    "accentcolortext",
    "activetext",
    "buttonborder",
    "buttonface",
    "buttontext",
    "canvas",
    "canvastext",
    "field",
    "fieldtext",
    "graytext",
    "highlight",
    "highlighttext",
    "linktext",
    "mark",
    "marktext",
    "selecteditem",
    "selecteditemtext",
    "visitedtext",
    "currentcolor",
    "transparent",
    // CSS-wide keywords.
    "inherit",
    "initial",
    "unset",
    "revert",
    "revert-layer",
];

/// Keywords a colour property takes besides a colour: `fill: none`, `caret-color: auto`.
const OTHER_COLOR_KEYWORDS: &[&str] = &["none", "auto", "context-fill", "context-stroke", "invert"];

/// The single keywords `background`'s shorthand takes besides a colour.
const BACKGROUND_KEYWORDS: &[&str] = &[
    "repeat",
    "repeat-x",
    "repeat-y",
    "no-repeat",
    "space",
    "round",
    "fixed",
    "scroll",
    "local",
    "center",
    "top",
    "bottom",
    "left",
    "right",
    "border-box",
    "padding-box",
    "content-box",
    "text",
    "cover",
    "contain",
];

/// `[a-z][a-z0-9-]*`, ignoring case: CSS keywords are case-insensitive.
fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// Warns once per property and string about a raw value on a colour property
/// that looks like a name but is neither a theme colour nor a CSS keyword it takes.
pub(crate) fn warn_unknown_color_name(property: Property, value: &str) {
    thread_local! {
        static WARNED: std::cell::RefCell<Vec<(Property, String)>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    if !property.takes_color() || !is_identifier(value) {
        return;
    }
    let lower = value.to_ascii_lowercase();
    let known = CSS_COLOR_KEYWORDS.contains(&lower.as_str())
        || OTHER_COLOR_KEYWORDS.contains(&lower.as_str())
        || (property == Property::Background && BACKGROUND_KEYWORDS.contains(&lower.as_str()));
    if known {
        return;
    }
    let first = WARNED.with_borrow_mut(|warned| {
        let first = !warned
            .iter()
            .any(|(seen, seen_value)| *seen == property && seen_value == value);
        if first {
            warned.push((property, value.to_string()));
        }
        first
    });
    if first {
        warn(&format!(
            "`{value}` on `{}` is neither a theme colour nor a CSS colour keyword; \
             it is emitted as-is and paints nothing.",
            property.as_str()
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::{css::Stylesheet, sx::sx, utils::take_warnings};

    fn warnings_of(sx: crate::sx::Sx) -> Vec<String> {
        take_warnings();
        let _ = Stylesheet::from(&sx);
        take_warnings()
    }

    #[test]
    fn a_misspelled_theme_colour_warns_once() {
        let warnings = warnings_of(sx().color("primry"));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`primry`"));
        assert!(warnings_of(sx().color("primry")).is_empty());
        assert_eq!(
            warnings_of(sx().background("primry")).len(),
            1,
            "once per property"
        );
    }

    #[test]
    fn every_colour_property_is_checked() {
        let warnings = warnings_of(
            sx().fill("primry")
                .stroke("primry")
                .outline_color("primry")
                .caret_color("primry")
                .accent_color("primry")
                .scrollbar_color("primry"),
        );
        assert_eq!(warnings.len(), 6, "{warnings:?}");
    }

    #[test]
    fn background_shorthand_keywords_stay_silent() {
        for value in ["no-repeat", "fixed", "center", "cover", "none"] {
            let warnings = warnings_of(sx().background(value));
            assert!(warnings.is_empty(), "{value}: {warnings:?}");
        }
        assert!(warnings_of(sx().fill("none").caret_color("auto")).is_empty());
    }

    #[test]
    fn css_colour_values_stay_silent() {
        for value in [
            "white",
            "currentColor",
            "rgba(0,0,0,.1)",
            "var(--x)",
            "transparent",
            "revert-layer",
            "primary",
        ] {
            let warnings = warnings_of(sx().color(value).border_color(value));
            assert!(warnings.is_empty(), "{value}: {warnings:?}");
        }
        assert!(warnings_of(sx().background("none")).is_empty());
    }

    #[test]
    fn a_non_colour_property_is_not_checked() {
        assert!(warnings_of(sx().display("flx")).is_empty());
    }
}
