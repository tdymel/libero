use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::CssLayer;
use crate::css::Stylesheet;

#[derive(Clone)]
struct RegisteredStylesheet {
    css: Stylesheet,
    ref_count: usize,
}

#[derive(Clone, Default)]
pub struct StylesheetRegistry {
    inner: Rc<RefCell<BTreeMap<(CssLayer, u64), RegisteredStylesheet>>>,
}

impl StylesheetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self, stylesheet: impl Into<Stylesheet>, layer: CssLayer) {
        let stylesheet = stylesheet.into();
        let key = (layer, stylesheet.hash());
        let mut registry = self.inner.borrow_mut();

        let entry = registry.entry(key).or_insert_with(|| RegisteredStylesheet {
            css: Stylesheet::from(format!(
                "@layer {}{{{}}}",
                layer.css_name(),
                stylesheet.as_str()
            )),
            ref_count: 0,
        });

        entry.ref_count += 1;
    }

    pub fn release(&self, key: (CssLayer, u64)) {
        let mut registry = self.inner.borrow_mut();

        if let Some(entry) = registry.get_mut(&key) {
            entry.ref_count -= 1;
            if entry.ref_count == 0 {
                registry.remove(&key);
            }
        }
    }

    pub fn stylesheets(&self) -> Vec<(String, String)> {
        self.inner
            .borrow()
            .iter()
            .map(|((layer, hash), entry)| {
                // Deliberately not `format!("{}-{hash:x}", ...)` - that
                // shape silently truncates under `--wasm-split` (wasm-split
                // fork BUGS.md, Bug 3).
                let mut key = String::from(layer.css_name());
                key.push('-');
                push_hex(&mut key, *hash);
                (key, entry.css.as_str().to_string())
            })
            .collect()
    }
}

fn push_hex(out: &mut String, mut value: u64) {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    if value == 0 {
        out.push('0');
        return;
    }
    let mut buf = [0u8; 16];
    let mut i = 16;
    while value != 0 {
        i -= 1;
        buf[i] = DIGITS[(value & 0xf) as usize];
        value >>= 4;
    }
    out.push_str(std::str::from_utf8(&buf[i..]).unwrap());
}
