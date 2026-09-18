//! The a11y sweep over every docs page (todo 760). Not part of the suite:
//! `cargo run -p e2e -- sweep` serves the docs site and runs only this.

#[test]
fn every_docs_page() {
    e2e::sweep::run().unwrap();
}
