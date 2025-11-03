use web_sys::{Element, console};
use crate::node::{NodeRef, render_node};

pub struct KokeObj {
    root_elm: Element
}

impl KokeObj {
    pub fn new(root_elm: &Element) -> Self {
        Self {
            root_elm: root_elm.clone()
        }
    }
    
    pub fn render_node(&self, node: &NodeRef) {
        let window = match web_sys::window() {
            Some(win) => win,
            None => {
                console::error_1(&"cannot get window".into());
                return;
            }
        };
        let document = match window.document() {
            Some(doc) => doc,
            None => {
                console::error_1(&"cannot get document".into());
                return;
            }
        };

        let content = render_node(node, &document);
        self.root_elm.set_inner_html("");
        self.root_elm.append_child(&content).unwrap();
    }
}
