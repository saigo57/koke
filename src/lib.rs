use web_sys::{Document, console};
use crate::koke_obj::KokeObj;

pub mod node;
pub mod koke_obj;

pub fn init(root_id: &str) -> Option<KokeObj> {
    let window = match web_sys::window() {
        Some(win) => win,
        None => {
            console::error_1(&"cannot get window".into());
            return None;
        }
    };
    let document = match window.document() {
        Some(doc) => doc,
        None => {
            console::error_1(&"cannot get document".into());
            return None;
        }
    };
    let body = match document.body() {
        Some(body) => body,
        None => {
            console::error_1(&"cannot get body".into());
            return None;
        }
    };

    let root_elm = match document.get_element_by_id(root_id) {
        Some(elm) => elm,
        None => {
            console::error_1(&format!("cannot get root element: {}", root_id).into());
            return None;
        }
    };
    
    match body.append_child(&root_elm) {
        Ok(_) => (),
        Err(_) => {
            console::error_1(&"Failed to append root element to body".into());
            return None;
        }
    };
    
    return Some(KokeObj::new(&root_elm));
}
