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
    "/time-picker",
    "/accordion",
    "/table",
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
