//! `PieChart`: its shares beside the slices, a donut's centre in the ring's middle.

use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("pie_chart", "/pie-chart").run();
}

#[test]
fn the_slices_carry_their_shares() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg path")
            .await
            .unwrap();
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert_eq!(labels, ["60%", "25%", "15%"]);
        let shares: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart tbody td:last-child')].map(t => t.textContent)",
        )
        .await;
        assert_eq!(shares, ["60%", "25%", "15%"]);
        fixture.close().await.unwrap();
    });
}

/// The centre sits over the ring's hole, which is the plot's middle.
#[test]
fn a_donut_s_center_sits_in_its_middle() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart/donut", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#center").await.unwrap();
        let [dx, dy]: [f64; 2] = e2e::js(
            &fixture.page,
            "(() => { const mid = r => [r.x + r.width / 2, r.y + r.height / 2]; \
             const [cx, cy] = mid(document.querySelector('#center').getBoundingClientRect()); \
             const [px, py] = mid(document.querySelector('#chart svg').getBoundingClientRect()); \
             return [cx - px, cy - py]; })()",
        )
        .await;
        assert!(dx.abs() < 2.0 && dy.abs() < 2.0, "{dx} {dy}");
        let texts: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart svg text').length",
        )
        .await;
        assert_eq!(texts, 0);
        fixture.close().await.unwrap();
    });
}
