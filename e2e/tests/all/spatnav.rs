//! Todo 3156: under the Android WebView's spatial navigation (on after the first D-pad key),
//! the topmost element of a `cursor: pointer` subtree is a Tab stop. Every route here must
//! offer only elements that are focusable on their own.

#[cfg(feature = "android")]
const ROUTES: &[&str] = &[
    "/checkbox",
    "/switch",
    "/radio-group",
    "/chip",
    "/rating",
    "/slider",
    "/segmented-control",
    "/select",
    "/multi-select",
    "/cascader",
    "/combobox",
    "/phone-field",
    "/file-field",
    "/color-picker",
    "/calendar",
    "/calendar/range",
    "/calendar/month",
    "/calendar/year",
    "/calendar/clock",
    "/time-picker",
    "/time-picker/analog",
    "/time-picker/date-time",
    "/accordion",
    "/table",
    "/table/select",
    "/table/filter",
    "/table/column-filter",
    "/table/date-filter",
    "/table/filter-panel",
    "/table-detail",
    "/table-groups",
    "/table-toolbar",
    "/table-reorder",
    "/table-grid",
    "/table-tree",
    "/table-tree-cells",
    "/tabs",
    "/stepper",
    "/tree",
    "/nav-link",
    "/menubar",
    "/menu",
    "/pagination",
    "/bottom-navigation",
    "/scroller",
    "/carousel",
    "/sortable",
    "/lightbox",
    "/spotlight",
    "/paper",
    "/button",
    "/action-icon",
    "/native-select",
    "/form/summary",
    "/text-field",
];

/// Route, the element a tap opens a list from, and the list it opens.
#[cfg(feature = "android")]
const OPEN: &[(&str, &str, &str)] = &[
    ("/select", "[role=combobox]", "[role=listbox]"),
    ("/multi-select", "[role=combobox]", "[role=listbox]"),
    ("/combobox", "[role=combobox]", "[role=listbox]"),
    ("/cascader", "[role=combobox]", "[role=option]"),
    ("/menu", "[aria-haspopup=menu]", "[role=menu]"),
    (
        "/menubar",
        "[role=menubar] [data-menubar-index]",
        "[role=menu]",
    ),
    ("/popover", "#trigger", "#box"),
    ("/table/menu", "[data-column-menu]", "[role=menu]"),
];

mod no_pointer_wrapper_is_a_tab_stop {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        use e2e::driver::Android;

        e2e::android::block_on(async {
            let mut found = Vec::new();
            for route in super::ROUTES {
                let mut d = Android::open(route).await.unwrap();
                for stray in d.stray_tab_stops().await.unwrap() {
                    found.push(format!("{route}: {stray}"));
                }
                d.finish("a Tab walk").await.unwrap();
            }
            assert!(found.is_empty(), "stray Tab stops:\n{}", found.join("\n"));
        });
    }
}

mod no_open_list_has_a_stray_tab_stop {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        use e2e::driver::{Android, Driver, eventually};

        e2e::android::block_on(async {
            let mut found = Vec::new();
            for (route, trigger, list) in super::OPEN {
                let mut d = Android::open(route).await.unwrap();
                let opened = async {
                    d.click(trigger).await?;
                    eventually(&mut d, "the list", async |d| d.exists(list).await).await
                }
                .await;
                if let Err(e) = opened {
                    found.push(format!("{route}: {e}"));
                    continue;
                }
                for stray in d.pointer_wrappers().await.unwrap() {
                    found.push(format!("{route}: {stray}"));
                }
                for stray in d.stray_tab_stops_open().await.unwrap() {
                    found.push(format!("{route}: {stray}"));
                }
                d.finish("an open list's Tab walk").await.unwrap();
            }
            assert!(found.is_empty(), "stray Tab stops:\n{}", found.join("\n"));
        });
    }
}
