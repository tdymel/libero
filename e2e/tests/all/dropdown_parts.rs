//! `dropdown_parts` reaches the portaled dropdowns: the box, a list's rows and
//! search box, a cascader's columns, a date field's calendar, a color field's picker.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

/// A selector, a computed-style property and its expected value.
type Check = (&'static str, &'static str, &'static str);

#[test]
fn dropdown_parts_style_the_portaled_dropdowns() {
    block_on(async {
        let fixture = Fixture::open("/dropdown-parts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#color").await.unwrap();
        let read = |selector: &'static str, property: &'static str| async move {
            page.evaluate(format!(
                "(() => {{ const el = document.querySelector({selector:?}); \
                 return el ? getComputedStyle(el).{property} : 'missing'; }})()"
            ))
            .await
            .unwrap()
            .into_value::<String>()
            .unwrap()
        };

        let dropdowns: [(&str, &str, &[Check]); 4] = [
            (
                "#select",
                "[data-slot=dropdown] [data-slot=option]",
                &[
                    ("[data-slot=dropdown]", "paddingTop", "11px"),
                    (
                        "[data-slot=dropdown] [data-slot=search]",
                        "letterSpacing",
                        "2px",
                    ),
                    (
                        "[data-slot=dropdown] [data-slot=option]",
                        "letterSpacing",
                        "3px",
                    ),
                ],
            ),
            (
                "#cascader",
                "[data-slot=dropdown] [data-slot=column]",
                &[(
                    "[data-slot=dropdown] [data-slot=column]",
                    "paddingTop",
                    "9px",
                )],
            ),
            (
                "#date",
                "[data-slot=dropdown] [data-slot=day]",
                &[
                    ("[data-slot=dropdown]", "paddingTop", "11px"),
                    // The rebased `& > * [data-slot='header']` reaches the calendar (2312).
                    (
                        "[data-slot=dropdown] [data-slot=header]",
                        "paddingTop",
                        "7px",
                    ),
                    (
                        "[data-slot=dropdown] [data-slot=day]",
                        "letterSpacing",
                        "3px",
                    ),
                ],
            ),
            (
                "#color",
                "[data-slot=dropdown] [data-slot=swatch]",
                &[
                    (
                        "[data-slot=dropdown] [data-slot=swatch]",
                        "marginTop",
                        "5px",
                    ),
                    (
                        "[data-slot=dropdown] [data-slot=saturation] > [data-slot=thumb]",
                        "borderTopWidth",
                        "3px",
                    ),
                ],
            ),
        ];
        for (trigger, shown, cases) in dropdowns {
            pointer::click(page, trigger).await.unwrap();
            wait::for_visible(page, shown).await.unwrap();
            for &(selector, property, expected) in cases {
                let value = read(selector, property).await;
                assert_eq!(value, expected, "{trigger}: {selector} {property}");
            }
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[data-slot=dropdown]')",
                "the dropdown to close",
            )
            .await
            .unwrap();
        }
        // The trigger stays the field's: `dropdown_parts` never reaches it.
        assert_eq!(read("#select", "letterSpacing").await, "normal");

        fixture.console.assert_clean("dropdown parts").unwrap();
        fixture.close().await.unwrap();
    });
}
