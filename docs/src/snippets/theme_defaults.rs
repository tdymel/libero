//! The size, radius, variant and color defaults a page prints against `Theme::DEFAULT`.

use super::*;

use libero::theme::Theme;

/// The look props `component` takes from the theme, with the value `Theme::DEFAULT` gives them.
macro_rules! looks {
    ($component:expr; $($($name:literal),+ => $field:ident: $($prop:ident)+;)+) => {
        match $component {
            $($($name)|+ => vec![$((stringify!($prop), Theme::DEFAULT.$field.$prop.as_str())),+],)+
            _ => Vec::new(),
        }
    };
}

fn theme_looks(component: &str) -> Vec<(&'static str, &'static str)> {
    looks! { component;
        "Accordion" => accordion: size;
        "ActionIcon" => action_icon: size radius;
        "Alert" => alert: variant radius;
        "Anchor" => anchor: size;
        "Autocomplete" => autocomplete: size radius;
        "Avatar", "AvatarGroup" => avatar: variant color size radius;
        "Badge" => badge: variant color size radius;
        "Blockquote" => blockquote: size radius color;
        "Burger" => burger: size;
        "Button" => button: variant size radius;
        "Cascader" => cascader: size radius;
        "Checkbox" => checkbox: variant size radius color;
        "Chip" => chip: variant size radius color;
        "ChronoField" => chrono_field: size radius;
        "ChronoPicker" => chrono_picker: size;
        "ColorField" => color_field: size radius;
        "ColorPicker" => color_picker: size radius;
        "ColorSwatch" => color_swatch: size radius;
        "Combobox" => combobox: size radius;
        "Container" => container: size;
        "DataList" => data_list: size;
        "Dialog" => dialog: size;
        "DirectionToggle" => direction_toggle: variant;
        "Divider" => divider: size;
        "Drawer", "DrawerOptions" => drawer: size;
        "FileField" => file_field: variant size radius;
        "Header" => header: size;
        "HoverCard" => hover_card: radius;
        "Icon" => icon: variant size radius;
        "ImageList" => image_list: variant radius;
        "Indicator" => indicator: size color radius;
        "Kbd" => kbd: size;
        "List" => list: size;
        "Loader" => loader: variant size color;
        "Menu" => menu: size radius;
        "Menubar" => menubar: size radius;
        "MultiSelect" => multi_select: size radius;
        "NativeSelect" => native_select: size radius;
        "NumberField" => number_field: size radius;
        "Pagination" => pagination: size radius color;
        "Paper" => paper: radius;
        "PasswordField", "TextField" => text_field: size radius;
        "PhoneField" => phone_field: size radius;
        "PinField" => pin_field: size radius;
        "ProgressBar" => progress_bar: size radius color;
        "Radio", "RadioGroup" => radio: variant size color;
        "Rating" => rating: size color;
        "Repository" => repository: variant;
        "RichTextEditor", "Textarea" => textarea: size radius;
        "SegmentedControl" => segmented_control: variant color;
        "Select" => select: size radius;
        "Sidebar" => sidebar: size;
        "Skeleton" => skeleton: radius;
        "Slider", "RangeSlider" => slider: size color;
        "Splitter" => splitter: size;
        "Stepper" => stepper: size;
        "Switch" => switch: variant size radius color;
        "Table" => table: size;
        "Tabs" => tabs: size color;
        "TagsField" => tags_field: size radius;
        "Text" => text: size;
        "ThemeSwitcher" => theme_switcher: variant;
        "Timeline" => timeline: radius;
        "Title" => title: size;
        "Tldr" => tldr: variant size;
        "Tooltip" => tooltip: size;
        "Tree" => tree: size;
    }
}

/// Todo 2697: a page's size, radius, variant and color rows and demo controls start at what
/// `Theme::DEFAULT` gives the component, so a changed default moves the docs with it.
#[test]
fn pages_print_the_theme_defaults() {
    let mut problems = Vec::new();
    for Rendered {
        route,
        demos,
        pages,
    } in rendered()
    {
        for group in pages.iter().flat_map(|page| &page.properties) {
            let component = group.component().split('<').next().unwrap_or_default();
            let looks = theme_looks(component);
            for (name, default) in group.defaults() {
                if let Some((_, expected)) = looks.iter().find(|(prop, _)| *prop == name)
                    && default != *expected
                {
                    problems.push(format!(
                        "{route}: `{component}` `{name}` row reads {default:?}, the theme {expected:?}"
                    ));
                }
            }
        }
        for demo in demos {
            let looks = theme_looks(&demo.component);
            for control in &demo.controls {
                if let Some((_, expected)) = looks.iter().find(|(prop, _)| *prop == control.name)
                    && control.default != *expected
                {
                    problems.push(format!(
                        "{route}: `{}` control `{}` starts at {:?}, the theme {expected:?}",
                        demo.component, control.name, control.default
                    ));
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The end of the call or chain starting at `from`: its first comma or closing bracket at depth 0.
fn chain_end(text: &str, from: usize) -> usize {
    let mut depth = 0i32;
    for (i, byte) in text.bytes().enumerate().skip(from) {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' if depth == 0 => return i,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => return i,
            _ => {}
        }
    }
    text.len()
}

/// A row or control the theme owns reads `theme.<component>.<prop>`, not a literal that keeps
/// its value until the theme moves.
#[test]
fn pages_do_not_print_theme_defaults_as_literals() {
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
        &mut files,
    );
    let mut problems = Vec::new();
    for file in files {
        let source: String = std::fs::read_to_string(&file)
            .unwrap()
            .split_whitespace()
            .collect();
        // The owner of a row is the `props("X"` above it, of a control the `component:"X"`.
        for (opener, owner_key) in [
            (r#"prop(""#, r#"props(""#),
            (r#"Control::"#, r#"component:""#),
        ] {
            for (at, _) in source.match_indices(opener) {
                let after = &source[at + opener.len()..];
                let after = if opener == "Control::" {
                    after.split_once(r#"(""#).map_or("", |(_, rest)| rest)
                } else {
                    after
                };
                let name = after.split('"').next().unwrap_or_default();
                let owner = source[..at].rfind(owner_key).map_or("", |key| {
                    let rest = &source[key + owner_key.len()..];
                    rest.split('"').next().unwrap_or_default()
                });
                if !theme_looks(owner).iter().any(|(prop, _)| *prop == name) {
                    continue;
                }
                if source[at..chain_end(&source, at)].contains(r#".default(""#) {
                    problems.push(format!(
                        "{}: `{owner}` `{name}` prints a literal default",
                        file.display()
                    ));
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The mirrors print the same defaults as `Theme::DEFAULT`, by hand.
#[test]
fn mirrors_print_the_theme_defaults() {
    let public = Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let mut problems = Vec::new();
    for Rendered { pages, .. } in rendered() {
        for page in pages {
            let Some(markdown) = &page.markdown else {
                continue;
            };
            let Ok(md) = std::fs::read_to_string(public.join(markdown.trim_start_matches('/')))
            else {
                continue;
            };
            for (heading, rows) in md_props(&md) {
                // A mirror with no heading over its table is its page's only group.
                let heading = match (heading.as_str(), &page.properties[..]) {
                    ("", [only]) => only.component(),
                    _ => heading.as_str(),
                };
                let component = heading.split('<').next().unwrap_or_default();
                for (name, expected) in theme_looks(component) {
                    if let Some(cell) = rows.get(name)
                        && cell.trim_matches('`') != expected
                    {
                        problems.push(format!(
                            "{markdown}: `{component}` `{name}` reads {cell:?}, the theme {expected:?}"
                        ));
                    }
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
