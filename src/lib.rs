use web_sys::console;
use crate::msg_proc::registr_msg_proc;
use crate::node::NodeRef;
use crate::event::Event;
use crate::context::Context;

pub mod node;
pub mod event;
pub mod msg_proc;
pub mod state;
pub mod context;
mod test_helper;

pub fn init<T>(
    root_id: &str,
    ui_func: fn(ctx: &Context, model: &T) -> NodeRef,
    model: &T,
    update: fn(String, &T) -> T
) -> Option<bool>
where
    T: Copy,
{
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
    
    registr_msg_proc(ui_func, &body, &document, &root_elm, model, update);
    // 初回レンダリングをトリガー
    Event::trigger_render_event(&body);

    return Some(true);
}

pub fn log(msg: &str) {
    console::log_1(&msg.into());
}
