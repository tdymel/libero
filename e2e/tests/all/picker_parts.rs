//! The `parts` prop reaches the date and time pickers' inner parts, the
//! calendar inside a date-time's tab panel too.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

#[test]
fn parts_style_the_picker_parts() {
    block_on(async {
        let fixture = Fixture::open("/picker-parts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#flow [data-slot=day]")
            .await
            .unwrap();
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

        let cases = [
            ("#date [data-slot=day]", "letterSpacing", "3px"),
            ("#date [data-slot=title]", "fontStyle", "italic"),
            ("#time [data-slot=value]", "letterSpacing", "4px"),
            ("#duration [data-slot=value]", "letterSpacing", "4px"),
            ("#duration [data-slot=unit]", "fontStyle", "italic"),
            ("#flow [data-slot=day]", "letterSpacing", "3px"),
            ("#plain [data-slot=day]", "letterSpacing", "normal"),
            ("#plain [data-slot=title]", "fontStyle", "normal"),
        ];
        for (selector, property, expected) in cases {
            assert_eq!(
                read(selector, property).await,
                expected,
                "{selector} {property}"
            );
        }

        fixture.console.assert_clean("picker parts").unwrap();
        fixture.close().await.unwrap();
    });
}
