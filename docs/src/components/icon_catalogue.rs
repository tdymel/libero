use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, Flex, Pagination, Pictogram, Select, Text, TextField},
    hooks::use_debounced_value,
    sx::{StaticSx, sx},
};
use pictogram_core::{Icon, Svg};

/// Cells per page: enough to browse, few enough inline svgs for Blitz.
const PER_PAGE: usize = 120;

/// The set shown first, the one the demos draw from.
const FIRST_SET: &str = "lucide";

/// `build.rs` writes one `<set>-<variant>.json` per set and variant in here.
static ICON_FILES: Asset = asset!("/assets/icons", AssetOptions::folder());

static GRID_SX: StaticSx = StaticSx::new(|| {
    sx().display("grid")
        .grid_template_columns("repeat(auto-fill, minmax(112px, 1fr))")
        .gap("xs")
});

/// One icon set as the catalogue lists it; its icons are fetched per variant.
#[derive(Clone, Copy, PartialEq)]
pub struct CatalogueSet {
    /// The crate's set name, e.g. `lucide`.
    pub name: &'static str,
    pub title: &'static str,
    pub license: &'static str,
    pub repository: &'static str,
    pub upstream_version: &'static str,
    /// Lobe's colour variants are left out by `build.rs`.
    pub variants: &'static [&'static str],
}

// `ICON_SETS`: every set of pictogram's index, from `build.rs`.
include!(concat!(env!("OUT_DIR"), "/icon_sets.rs"));

impl CatalogueSet {
    fn named(name: &str) -> &'static Self {
        ICON_SETS
            .iter()
            .find(|set| set.name == name)
            .unwrap_or(&ICON_SETS[0])
    }

    /// `variant` as the set's own `'static` name, else its first variant.
    fn variant(&self, variant: &str) -> &'static str {
        self.variants
            .iter()
            .copied()
            .find(|v| *v == variant)
            .unwrap_or(self.variants[0])
    }

    fn file(&self, variant: &str) -> String {
        format!("{}-{variant}.json", self.name)
    }

    /// The path of the icon's const, as an app writes it.
    fn rust_path(&self, icon: &Icon) -> String {
        format!(
            "pictogram_icons_{}::{}::{}",
            self.name.replace('-', "_"),
            icon.module,
            icon.variant
        )
    }
}

/// A file's `[name, module, view_box, attrs, body]` rows as icons. Leaked: `Pictogram`
/// takes `'static` parts, and `load` keeps each file once.
fn parse(json: &str, variant: &'static str) -> Result<Vec<Icon>, serde_json::Error> {
    let rows: Vec<(String, String, String, String, String)> = serde_json::from_str(json)?;
    Ok(rows
        .into_iter()
        .map(|(name, module, view_box, attrs, body)| Icon {
            name: name.leak(),
            module: module.leak(),
            variant,
            svg: Svg {
                view_box: view_box.leak(),
                attrs: attrs.leak(),
                body: body.leak(),
            },
        })
        .collect())
}

/// The icons of one set and variant, fetched once (web) or read from the bundle (native).
async fn load(
    set: &'static CatalogueSet,
    variant: &'static str,
) -> Result<&'static [Icon], String> {
    static LOADED: LazyLock<Mutex<HashMap<String, &'static [Icon]>>> =
        LazyLock::new(Default::default);
    let file = set.file(variant);
    if let Some(icons) = LOADED.lock().unwrap().get(&file) {
        return Ok(icons);
    }
    let bytes = dioxus::asset_resolver::read_asset_bytes(format!("{ICON_FILES}/{file}"))
        .await
        .map_err(|err| format!("{file}: {err}"))?;
    let json = String::from_utf8_lossy(&bytes);
    let icons: &'static [Icon] = parse(&json, variant)
        .map_err(|err| format!("{file}: {err}"))?
        .leak();
    LOADED.lock().unwrap().insert(file, icons);
    Ok(icons)
}

/// The icons that match `query`, in index order.
fn filtered(icons: &[Icon], query: &str) -> Vec<Icon> {
    icons
        .iter()
        .filter(|icon| icon.matches(query))
        .copied()
        .collect()
}

/// Page count for `len` icons; at least one, so an empty search keeps its page.
fn page_count(len: usize) -> u32 {
    len.div_ceil(PER_PAGE).max(1) as u32
}

/// The icons of 1-based `page`, clamped into range.
fn page_slice(icons: &[Icon], page: u32) -> &[Icon] {
    let page = page.clamp(1, page_count(icons.len())) as usize;
    let start = ((page - 1) * PER_PAGE).min(icons.len());
    &icons[start..(start + PER_PAGE).min(icons.len())]
}

/// Every icon of pictogram's sets, by set and variant, with search.
#[component]
pub fn IconCatalogue() -> Element {
    let mut set_name = use_signal(|| FIRST_SET.to_string());
    let mut variant = use_signal(|| CatalogueSet::named(FIRST_SET).variants[0].to_string());
    let mut query = use_signal(String::new);
    let settled = use_debounced_value(query.into(), 200);
    let mut page = use_signal(|| 1u32);
    use_effect(move || {
        settled.read();
        page.set(1);
    });
    let loaded = use_resource(move || async move {
        let set = CatalogueSet::named(&set_name());
        load(set, set.variant(&variant())).await
    });

    let set = CatalogueSet::named(&set_name());
    let (icons, status) = match &*loaded.read() {
        Some(Ok(icons)) => {
            let icons = filtered(icons, &settled());
            let status = format!("{} icons", icons.len());
            (icons, status)
        }
        Some(Err(err)) => (Vec::new(), format!("Could not load the icons: {err}")),
        None => (Vec::new(), "Loading icons".to_string()),
    };
    let count = icons.len();
    let shown = page_slice(&icons, page());
    let pattern = format!(
        "pictogram_icons_{}::<module>::{}",
        set.name.replace('-', "_"),
        set.variant(&variant())
    );

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text {
                "{set.title} {set.upstream_version}, "
                Code { source: set.license }
                ", "
                Anchor { to: set.repository, target: "_blank", "{set.repository}" }
            }
            Flex { direction: "row", gap: "sm", wrap: "wrap", align: "flex-end",
                Select {
                    label: "Set",
                    options: ICON_SETS.iter().map(|set| set.name.to_string()).collect::<Vec<_>>(),
                    value: Some(set_name()),
                    onchange: move |next: Option<String>| {
                        if let Some(next) = next {
                            variant.set(CatalogueSet::named(&next).variants[0].to_string());
                            set_name.set(next);
                            page.set(1);
                        }
                    },
                }
                Select {
                    label: "Variant",
                    options: set.variants.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                    value: Some(variant()),
                    onchange: move |next: Option<String>| {
                        if let Some(next) = next {
                            variant.set(next);
                            page.set(1);
                        }
                    },
                }
                TextField {
                    label: "Search",
                    placeholder: "arrow left",
                    value: query(),
                    oninput: move |text| query.set(text),
                }
            }
            // The path as text too: a cell's `title` reaches neither a keyboard nor a touch.
            Text { size: "sm",
                "Each icon is a const, "
                Code { source: pattern }
                if let Some(icon) = shown.first() {
                    ", such as "
                    Code { source: set.rust_path(icon) }
                    " for {icon.name}"
                }
                "."
            }
            Text { size: "sm", role: "status", "{status}" }
            Box { framework_sx: &GRID_SX,
                for icon in shown.iter() {
                    Flex {
                        key: "{icon.module}-{icon.variant}",
                        direction: "column",
                        align: "center",
                        gap: "xs",
                        sx: sx().padding("xs").min_width("0"),
                        // The const's path on hover; no copy button per cell (todo 1384).
                        title: set.rust_path(icon),
                        Pictogram { icon: icon.svg, width: "24px", height: "24px" }
                        Text {
                            size: "xs",
                            sx: sx().max_width("100%").overflow("hidden").text_overflow("ellipsis").white_space("nowrap"),
                            title: icon.name,
                            "{icon.name}"
                        }
                    }
                }
            }
            Pagination {
                total: page_count(count),
                page: page(),
                onchange: move |next| page.set(next),
                aria_label: "Icon pages",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libero::context::LiberoProvider;

    /// A file as `build.rs` wrote it, read from disk: no fetch on the host.
    fn read(set: &str, variant: &'static str) -> Vec<Icon> {
        let path = format!(
            "{}/assets/icons/{set}-{variant}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        parse(&std::fs::read_to_string(&path).unwrap(), variant).unwrap()
    }

    #[test]
    fn every_set_is_listed() {
        assert_eq!(ICON_SETS.len(), 14);
        assert_eq!(CatalogueSet::named("lucide").variants, ["outlined"]);
        assert_eq!(
            CatalogueSet::named("lobe").variants,
            ["brand", "mono", "text", "text_cn"]
        );
        assert_eq!(CatalogueSet::named("material").license, "Apache-2.0");
    }

    #[test]
    fn every_variant_has_a_file() {
        for set in ICON_SETS {
            for variant in set.variants {
                assert!(!read(set.name, variant).is_empty(), "{}", set.file(variant));
            }
        }
        assert!(read("lucide", "outlined").len() > 1500);
    }

    #[test]
    fn search_narrows_by_words() {
        let found = filtered(&read("lucide", "outlined"), "arrow left");
        assert!(found.iter().any(|icon| icon.name == "arrow-left"));
        assert!(
            found
                .iter()
                .all(|icon| icon.name.contains("arrow") && icon.name.contains("left"))
        );
    }

    #[test]
    fn pages_hold_at_most_120_and_clamp() {
        let icons = read("lucide", "outlined");
        let last = page_count(icons.len());
        assert_eq!(page_slice(&icons, 1).len(), PER_PAGE);
        assert_eq!(page_slice(&icons, 1)[0], icons[0]);
        assert_eq!(page_slice(&icons, last + 5), page_slice(&icons, last));
        assert_eq!(page_count(0), 1);
        assert!(page_slice(&[], 1).is_empty());
    }

    #[test]
    fn path_is_the_const() {
        let set = CatalogueSet::named("font-awesome");
        let icons = read(set.name, "solid");
        let icon = icons.iter().find(|icon| icon.name == "arrow-up").unwrap();
        assert_eq!(
            set.rust_path(icon),
            "pictogram_icons_font_awesome::arrow_up::solid"
        );
    }

    #[test]
    fn unknown_names_fall_back() {
        let set = CatalogueSet::named("nope");
        assert_eq!(set.name, ICON_SETS[0].name);
        assert_eq!(set.variant("nope"), set.variants[0]);
    }

    #[test]
    fn renders_the_set_before_its_icons() {
        fn app() -> Element {
            rsx! {
                LiberoProvider { IconCatalogue {} }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("ISC"), "{html}");
        assert!(html.contains("Loading icons"), "{html}");
        assert!(html.contains("pictogram_icons_lucide::"), "{html}");
    }
}
