use dioxus::prelude::*;

pub fn call_handler<E: 'static>(handler: &Option<EventHandler<E>>, event: E) {
    if let Some(handler) = handler {
        handler.call(event);
    }
}
