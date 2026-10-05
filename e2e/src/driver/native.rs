use std::{str::FromStr, time::Duration};

use anyhow::{Result, anyhow};

use super::{Driver, Platform, Rect};
use crate::passes::keyboard;

/// A fixture route mounted in a windowless Blitz document.
pub struct Native {
    pub page: crate::native::Page,
}

impl Native {
    pub fn open(route: &str) -> Self {
        let page = e2e_fixtures::route(route).unwrap_or_else(|| panic!("no fixture at {route}"));
        Self {
            page: crate::native::mount(page),
        }
    }
}

fn native_key(key: keyboard::Key) -> Result<crate::native::Key> {
    crate::native::Key::from_str(key.key).map_err(|_| anyhow!("no dioxus key named {:?}", key.key))
}

impl Driver for Native {
    fn platform(&self) -> Platform {
        Platform::Native
    }

    async fn click(&mut self, selector: &str) -> Result<()> {
        self.page.click(selector);
        Ok(())
    }

    async fn hover(&mut self, selector: &str) -> Result<()> {
        self.page.hover(selector);
        Ok(())
    }

    async fn double_click(&mut self, selector: &str) -> Result<()> {
        self.page.click(selector);
        self.page.click(selector);
        Ok(())
    }

    async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
        self.page.click_at(x as f32, y as f32);
        Ok(())
    }

    async fn viewport(&mut self) -> Result<(f64, f64)> {
        let (width, height) = self.page.window_size();
        Ok((f64::from(width), f64::from(height)))
    }

    async fn press(&mut self, key: keyboard::Key) -> Result<()> {
        let key = native_key(key)?;
        self.page.press(key);
        Ok(())
    }

    async fn press_shift(&mut self, key: keyboard::Key) -> Result<()> {
        let key = native_key(key)?;
        self.page.press_with(key, crate::native::Modifiers::SHIFT);
        Ok(())
    }

    async fn press_ctrl(&mut self, key: keyboard::Key) -> Result<()> {
        let key = native_key(key)?;
        self.page.press_with(key, crate::native::Modifiers::CONTROL);
        Ok(())
    }

    async fn press_alt(&mut self, key: keyboard::Key) -> Result<()> {
        let key = native_key(key)?;
        self.page.press_with(key, crate::native::Modifiers::ALT);
        Ok(())
    }

    async fn type_text(&mut self, text: &str) -> Result<()> {
        for ch in text.chars() {
            self.page
                .press(crate::native::Key::Character(ch.to_string()));
        }
        Ok(())
    }

    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
        self.page.drag(selector, dx as f32, dy as f32);
        Ok(())
    }

    async fn long_press(&mut self, selector: &str, ms: u64) -> Result<()> {
        let (x, y) = self.page.touch_down(selector);
        self.page.wait(Duration::from_millis(ms));
        self.page.touch_up(x, y);
        Ok(())
    }

    async fn swipe_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.page
            .swipe_from(x as f32, y as f32, dx as f32, dy as f32);
        Ok(())
    }

    async fn focus(&mut self, selector: &str) -> Result<()> {
        self.page.focus(selector);
        Ok(())
    }

    async fn text(&mut self, selector: &str) -> Result<String> {
        Ok(self.page.text(selector))
    }

    async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>> {
        Ok(self.page.attr(selector, name))
    }

    async fn exists(&mut self, selector: &str) -> Result<bool> {
        Ok(self.page.exists(selector))
    }

    async fn rect(&mut self, selector: &str) -> Result<Rect> {
        let (x, y, width, height) = self.page.rect(selector);
        Ok(Rect {
            x,
            y,
            width,
            height,
        })
    }

    async fn style(&mut self, selector: &str, property: &str) -> Result<String> {
        Ok(self.page.computed(selector, property))
    }

    async fn is_focused(&mut self, selector: &str) -> Result<bool> {
        Ok(self.page.is_focused(selector))
    }

    async fn focused_id(&mut self) -> Result<String> {
        let page = &self.page;
        Ok(page
            .focused()
            .and_then(|node| page.attr_of(node, "id"))
            .unwrap_or_default())
    }

    async fn focus_owner(&mut self) -> Result<String> {
        Ok(self.page.focus_owner())
    }

    /// No paint to await: the harness resolves on demand, so a frame is one 16 ms step of the
    /// animation clock, as Blitz's own frame timer heals and redraws (todo 1667).
    async fn frame(&mut self) -> Result<()> {
        self.page.advance(0.016);
        Ok(())
    }

    /// Real time for libero's timers, the same span on the animation clock.
    async fn idle(&mut self) {
        self.page.wait(Duration::from_millis(20));
        self.page.advance(0.02);
    }

    fn budget(&self) -> Duration {
        Duration::from_secs(3)
    }
}
