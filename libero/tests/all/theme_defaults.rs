//! Todo 392: a `*Defaults` holding a literal CSS colour looks right on the
//! light page only, unless `Theme::DARK` restates it. A source scan, so a
//! component no test renders is covered too.

use std::{fs, path::Path};

/// Literal colours that are right on either page, because they are not drawn
/// on it: the image list's bar lies on a photo, and a QR code must stay dark on
/// light for a scanner.
const SCHEME_INDEPENDENT: &[&str] = &["ImageListDefaults", "QrCodeDefaults"];

/// The string literals on `line` that name a CSS colour: `#rgb`..`#rrggbbaa`,
/// or an `rgb()`/`rgba()`/`hsl()`/`hsla()`.
fn colour_literals(line: &str) -> Vec<&str> {
    line.split('"')
        .skip(1)
        .step_by(2)
        .filter(|literal| {
            let hex = literal
                .split(|c: char| !(c == '#' || c.is_ascii_hexdigit()))
                .any(|word| {
                    word.strip_prefix('#').is_some_and(|digits| {
                        matches!(digits.len(), 3 | 4 | 6 | 8)
                            && digits.chars().all(|c| c.is_ascii_hexdigit())
                    })
                });
            hex || ["rgb(", "rgba(", "hsl(", "hsla("]
                .iter()
                .any(|function| literal.contains(function))
        })
        .collect()
}

/// `Theme::DARK`'s body, where a `*Defaults::DARK` has to be named.
fn theme_dark(theme_rs: &str) -> &str {
    let start = theme_rs
        .find("pub const DARK: Theme = Theme {")
        .expect("Theme::DARK in theme.rs");
    let body = &theme_rs[start..];
    &body[..body.find("};").expect("Theme::DARK ends")]
}

/// Every colour literal in `source` that its `*Defaults`' `DARK` does not
/// restate by field, or whose `DARK` `theme_dark` does not use. A `DARK` that
/// is `Self::DEFAULT` restates nothing. Tests below `#[cfg(test)]` are skipped.
fn literals_without_dark(file: &str, source: &str, theme_dark: &str) -> Vec<String> {
    let source = source.split("#[cfg(test)]").next().unwrap_or_default();
    let first_struct = source
        .lines()
        .find_map(|line| line.trim().strip_prefix("pub struct "))
        .and_then(|rest| rest.split([' ', '{', '<', '(']).next());
    let mut owner = first_struct;
    // The `DARK` block's lines, once the scan is inside one.
    let mut dark_fields: Option<Vec<&str>> = None;
    let mut in_dark = false;
    let mut hits = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let code = line.trim();
        if let Some(rest) = code.strip_prefix("impl ")
            && let Some(name) = rest.strip_suffix(" {").filter(|name| !name.contains(' '))
        {
            owner = Some(name);
            dark_fields = dark_block(source, index);
        }
        let dark_header = code.starts_with("pub const DARK: Self");
        if dark_header {
            in_dark = !code.ends_with(';');
        } else if in_dark && code == "};" {
            in_dark = false;
        }
        if dark_header || in_dark || code.starts_with("//") {
            continue;
        }
        let literals = colour_literals(code);
        if literals.is_empty() {
            continue;
        }
        let Some(owner) = owner else {
            hits.push(format!("{file}:{}: {code} (no struct)", index + 1));
            continue;
        };
        if SCHEME_INDEPENDENT.contains(&owner) {
            continue;
        }
        let field = code.split(':').next().unwrap_or_default();
        let restated = dark_fields
            .as_ref()
            .is_some_and(|fields| fields.contains(&field));
        let has_dark = restated && theme_dark.contains(&format!("{owner}::DARK"));
        if !has_dark {
            hits.push(format!(
                "{file}:{}: {owner} {}",
                index + 1,
                literals.join(" ")
            ));
        }
    }
    hits
}

/// The field names the first `DARK` after line `from` sets, or `None` when
/// there is no `DARK` at all.
fn dark_block(source: &str, from: usize) -> Option<Vec<&str>> {
    let mut lines = source.lines().skip(from).map(str::trim);
    let header = lines.find(|code| code.starts_with("pub const DARK: Self"))?;
    fn field(code: &str) -> Option<&str> {
        code.split_once(':')
            .map(|(field, _)| field.trim_start_matches(['{', ' ']))
    }
    // One line, `Self::DEFAULT` or `Self { a: .., b: .. }`.
    if header.ends_with(';') {
        let body = header.split_once('{').map_or("", |(_, body)| body);
        return Some(body.split(',').filter_map(field).collect());
    }
    Some(
        lines
            .take_while(|code| *code != "};")
            .filter_map(field)
            .collect(),
    )
}

#[test]
fn every_literal_colour_in_a_defaults_has_a_dark_counterpart() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/theme");
    let theme_rs = fs::read_to_string(root.join("theme.rs")).unwrap();
    let dark = theme_dark(&theme_rs);
    let mut hits = Vec::new();

    for entry in fs::read_dir(root.join("defaults")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = fs::read_to_string(&path).unwrap();
        hits.extend(literals_without_dark(&file, &source, dark));
    }
    hits.sort();

    assert!(
        hits.is_empty(),
        "a literal colour is only right on the light page: derive it from a theme \
         colour (`var(--lsx-muted-N)`), or add `DARK` and name it in `Theme::DARK`:\n{}",
        hits.join("\n")
    );
}

/// The scan itself, so the test above cannot pass by finding nothing.
#[test]
fn a_new_literal_without_dark_is_caught() {
    let dark = "pub const DARK: Theme = Theme { paper: PaperDefaults::DARK, };";
    let source = r##"
pub struct WidgetDefaults { pub background: &'static str }
impl WidgetDefaults {
    pub const DEFAULT: Self = Self {
        background: "#f6f8fa",
    };
}
"##;
    let caught = ["widget.rs:5: WidgetDefaults #f6f8fa"];
    assert_eq!(literals_without_dark("widget.rs", source, dark), caught);

    let with_dark = |dark_const: &str| source.replace("\n}\n", &format!("\n{dark_const}\n}}\n"));
    let dark = "pub const DARK: Theme = Theme { widget: WidgetDefaults::DARK, };";

    // A `DARK` that only repeats `DEFAULT` keeps the light literal.
    let inherited = with_dark("    pub const DARK: Self = Self::DEFAULT;");
    assert_eq!(literals_without_dark("widget.rs", &inherited, dark), caught);

    let restated =
        with_dark("    pub const DARK: Self = Self {\n        background: \"#161b22\",\n    };");
    assert!(literals_without_dark("widget.rs", &restated, dark).is_empty());
}

#[test]
fn a_var_is_not_a_colour_but_a_hex_in_a_shorthand_is() {
    assert!(colour_literals(r##"background: "var(--lsx-muted-1)","##).is_empty());
    assert!(colour_literals(r##"id: "#main","##).is_empty());
    assert_eq!(
        colour_literals(r##"border: "1px solid #d0d7de","##),
        ["1px solid #d0d7de"]
    );
    assert_eq!(
        colour_literals(r##"hover: "rgba(31, 35, 40, 0.08)","##),
        ["rgba(31, 35, 40, 0.08)"]
    );
}

/// The rendered side: a `*Defaults` variant reaches every component that
/// leaves its own unset.
mod variant {
    use crate::common::{body, render};

    use dioxus::prelude::*;
    use libero::{LiberoProvider, theme::Color};

    static OUTLINED: libero::theme::Theme = libero::theme::Theme {
        button: libero::theme::ButtonDefaults {
            variant: libero::theme::Variant::Outlined,
            ..libero::theme::Theme::DEFAULT.button
        },
        action_icon: libero::theme::ActionIconDefaults {
            variant: libero::theme::Variant::Outlined,
            ..libero::theme::Theme::DEFAULT.action_icon
        },
        chip: libero::theme::ChipDefaults {
            variant: libero::theme::Variant::Outlined,
            ..libero::theme::Theme::DEFAULT.chip
        },
        badge: libero::theme::BadgeDefaults {
            variant: libero::theme::Variant::Outlined,
            ..libero::theme::Theme::DEFAULT.badge
        },
        icon: libero::theme::IconDefaults {
            variant: libero::theme::Variant::Outlined,
            ..libero::theme::Theme::DEFAULT.icon
        },
        ..libero::theme::Theme::DEFAULT
    };

    /// A project sets the variant once in the theme, and every chrome component
    /// with no `variant` of its own takes it. An `ActionIcon` draws chrome only
    /// once it has a `color` or a `variant`, so it gets a `color`.
    #[test]
    fn an_unset_variant_follows_the_theme() {
        use libero::components::{ActionIcon, Badge, Button, Chip, Icon};

        fn app() -> Element {
            rsx! {
                LiberoProvider { themes: &OUTLINED,
                    Button { "Save" }
                    ActionIcon { aria_label: "Close", color: Color::Primary, "x" }
                    Chip { "Tag" }
                    Badge { "New" }
                    Icon { "i" }
                }
            }
        }

        let html = body(&render(app));
        let outlined = html
            .split("data-state=\"")
            .skip(1)
            .filter(|rest| {
                rest.split('"')
                    .next()
                    .is_some_and(|state| state.split(' ').any(|token| token == "outlined"))
            })
            .count();
        assert_eq!(outlined, 5, "{html}");
    }
}

/// The rendered side: a `*Defaults` colour reaches every component that leaves
/// its own unset, exactly as if the call site had named it.
mod color {
    use crate::common::{body, render_with};

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{
            Avatar, AvatarGroup, AvatarSpec, Badge, Input, ProgressBar, RangeSlider, Slider,
        },
        sx::ThemeAwareValue,
        theme::{AvatarDefaults, BadgeDefaults, Color, ProgressBarDefaults, SliderDefaults, Theme},
    };

    static ERROR: Theme = Theme {
        badge: BadgeDefaults {
            color: Color::Error,
            ..Theme::DEFAULT.badge
        },
        avatar: AvatarDefaults {
            color: Color::Error,
            ..Theme::DEFAULT.avatar
        },
        progress_bar: ProgressBarDefaults {
            color: Color::Error,
            ..Theme::DEFAULT.progress_bar
        },
        slider: SliderDefaults {
            color: Color::Error,
            ..Theme::DEFAULT.slider
        },
        ..Theme::DEFAULT
    };

    #[component]
    fn Part(part: &'static str, themed: bool, explicit: bool) -> Element {
        let color: Input<ThemeAwareValue> = match explicit {
            true => ThemeAwareValue::from(Color::Error).into(),
            false => Input::None,
        };
        let theme: &'static Theme = match themed {
            true => &ERROR,
            false => &Theme::DEFAULT,
        };
        let people = vec![
            AvatarSpec::from("Ada Lovelace"),
            AvatarSpec::from("Grace Hopper"),
            AvatarSpec::from("Radia Perlman"),
        ];
        rsx! {
            LiberoProvider { themes: theme,
                match part {
                    "badge" => rsx! { Badge { color, "New" } },
                    "avatar" => rsx! { Avatar { name: "Ada Lovelace", color } },
                    "avatar_group" => rsx! { AvatarGroup { max: 2, people, color } },
                    "progress_bar" => rsx! { ProgressBar { aria_label: "Upload", value: 30.0, color } },
                    "slider" => rsx! { Slider { aria_label: "Volume", value: 25.0, color, oninput: move |_| {} } },
                    _ => rsx! { RangeSlider { aria_label: "Price", value: (20.0, 80.0), color, oninput: move |_| {} } },
                }
            }
        }
    }

    fn html(part: &'static str, themed: bool, explicit: bool) -> String {
        body(&render_with(
            Part,
            PartProps {
                part,
                themed,
                explicit,
            },
        ))
    }

    /// A project sets the colour once in the theme; with no `color` of its own a
    /// component draws what an explicit `color` would, and not the old primary.
    #[test]
    fn an_unset_color_follows_the_theme() {
        for part in [
            "badge",
            "avatar",
            "avatar_group",
            "progress_bar",
            "slider",
            "range_slider",
        ] {
            let followed = html(part, true, false);

            assert_eq!(followed, html(part, true, true), "{part}");
            assert_ne!(followed, html(part, false, false), "{part}");
        }
    }
}

/// A `*Defaults` size and radius reach the component that leaves its own unset.
mod size {
    use crate::common::render;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Dialog, Header, Icon, Tldr},
        theme::{
            BUTTON_HEIGHT, DIALOG_SIZE, DialogDefaults, HEADER_HEIGHT, HeaderDefaults, ICON_SIZE,
            IconDefaults, Size, SizeCss, Theme, TldrDefaults,
        },
    };

    static LARGE: Theme = Theme {
        tldr: TldrDefaults {
            size: Size::Lg,
            radius: Some(Size::Xs),
            ..Theme::DEFAULT.tldr
        },
        icon: IconDefaults {
            size: Size::Lg,
            radius: Size::Xs,
            ..Theme::DEFAULT.icon
        },
        header: HeaderDefaults {
            size: Size::Lg,
            ..Theme::DEFAULT.header
        },
        dialog: DialogDefaults {
            size: Size::Lg,
            radius: Some(Size::Xs),
            ..Theme::DEFAULT.dialog
        },
        ..Theme::DEFAULT
    };

    #[component]
    fn Large(children: Element) -> Element {
        rsx! { LiberoProvider { themes: &LARGE, {children} } }
    }

    /// Spaces dropped, so a declaration matches either way.
    fn html(app: fn() -> Element) -> String {
        render(app).replace(' ', "")
    }

    #[test]
    fn an_unset_tldr_size_and_radius_follow_the_theme() {
        fn app() -> Element {
            rsx! { Large { Tldr { url: "https://example.com", icon_only: true } } }
        }
        let html = html(app);
        assert!(html.contains(&BUTTON_HEIGHT.value(Size::Lg)), "{html}");
        assert!(html.contains(&SizeCss::RADIUS.value(Size::Xs)), "{html}");
    }

    #[test]
    fn an_unset_icon_size_and_radius_follow_the_theme() {
        fn app() -> Element {
            rsx! { Large { Icon { "i" } } }
        }
        let html = html(app);
        let size = format!("--lsx-icon-size:{}", ICON_SIZE.value(Size::Lg));
        let radius = format!(
            "--lsx-icon-radius-default:{}",
            SizeCss::RADIUS.value(Size::Xs)
        );
        assert!(html.contains(&size), "{html}");
        assert!(html.contains(&radius), "{html}");
        assert!(
            html.contains("var(--lsx-icon-size-override,var(--lsx-icon-size))"),
            "{html}"
        );
    }

    #[test]
    fn an_unset_header_size_follows_the_theme() {
        fn app() -> Element {
            rsx! { Large { Header { "Libero" } } }
        }
        let html = html(app);
        let size = format!(
            "--lsx-header-height-default:{}",
            HEADER_HEIGHT.value(Size::Lg)
        );
        assert!(html.contains(&size), "{html}");
        assert!(
            html.contains("var(--lsx-header-height-override,var(--lsx-header-height-default))"),
            "{html}"
        );
    }

    #[test]
    fn an_unset_dialog_size_and_radius_follow_the_theme() {
        fn app() -> Element {
            rsx! { Large { Dialog { aria_label: "Tip", "Drag to reorder." } } }
        }
        let html = html(app);
        let size = format!("--lsx-dialog-size:{}", DIALOG_SIZE.value(Size::Lg));
        let radius = format!("--lsx-dialog-radius:{}", SizeCss::RADIUS.value(Size::Xs));
        assert!(html.contains(&size), "{html}");
        assert!(html.contains(&radius), "{html}");
    }
}
