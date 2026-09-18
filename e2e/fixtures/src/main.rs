//! The fixture app on the web, served by the e2e runner's `dx run`.

fn main() {
    e2e_fixtures::perf::install();
    dioxus::launch(e2e_fixtures::App);
}
