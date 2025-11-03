use web_sys;
use web_sys::wasm_bindgen::JsCast;

const RENDER_EVENT_CODE: &str = "koke-rust-render";


#[derive(Debug, Clone)]
pub enum Event {
    Click,
    Change(Option<String>),
    KeyDown,
    RustRender,
}

impl Event {
    pub fn from_event(event: &web_sys::Event) -> Option<Event> {
        match event.type_().as_str() {
            "click" => Some(Event::Click),
            "change" => {
                if let Some(target) = event.target() {
                    if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                        return Some(Event::Change(Some(input.value())));
                    }
                }
                return None;
            },
            "keydown" => Some(Event::KeyDown),
            RENDER_EVENT_CODE => Some(Event::RustRender),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Event::Click => "click",
            Event::Change(_) => "change",
            Event::KeyDown => "keydown",
            Event::RustRender => RENDER_EVENT_CODE,
        }
    }

    pub fn all_events() -> Vec<Event> {
        vec![
            Event::Click,
            Event::Change(None),
            Event::KeyDown,
            Event::RustRender
        ]
    }

    pub fn trigger_render_event(body: &web_sys::Element) {
        let event = web_sys::Event::new(RENDER_EVENT_CODE).unwrap();
        body.dispatch_event(&event).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_event_from_event() {
        let click_event = web_sys::Event::new("click").unwrap();
        let event = Event::from_event(&click_event).unwrap();
        match event {
            Event::Click => (),
            _ => panic!("Expected Click event"),
        }

        let document = web_sys::window().unwrap().document().unwrap();
        let change_event = web_sys::Event::new("change").unwrap();
        let input = document.create_element("input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        input.set_value("new value");
        document.body().unwrap().append_child(&input).unwrap();
        input.dispatch_event(&change_event).unwrap();
        let event = Event::from_event(&change_event).unwrap();
        match event {
            Event::Change(value) => {
                assert_eq!(value.unwrap(), "new value");
            },
            _ => panic!("Expected Change event"),
        }

        let keydown_event = web_sys::Event::new("keydown").unwrap();
        let event = Event::from_event(&keydown_event).unwrap();
        match event {
            Event::KeyDown => (),
            _ => panic!("Expected KeyDown event"),
        }
        
        let rust_render_event = web_sys::Event::new(RENDER_EVENT_CODE).unwrap();
        let event = Event::from_event(&rust_render_event).unwrap();
        match event {
            Event::RustRender => (),
            _ => panic!("Expected RustRender event"),
        }
    }
    
    #[wasm_bindgen_test]
    fn test_event_to_str() {
        let click_event = Event::Click;
        assert_eq!(click_event.to_str(), "click");
        let change_event = Event::Change(Some("new value".into()));
        assert_eq!(change_event.to_str(), "change");
        let keydown_event = Event::KeyDown;
        assert_eq!(keydown_event.to_str(), "keydown");
        let rust_render_event = Event::RustRender;
        assert_eq!(rust_render_event.to_str(), RENDER_EVENT_CODE);
    }
}
