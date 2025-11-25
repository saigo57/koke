use uuid::Uuid;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{Document, Element};
use crate::event::Event;

pub const EVENT_CUSTOM_DATA_KEY: &str = "data-uuid";

pub type NodeRef = Rc<RefCell<Node>>;
pub struct Node {
    event_key: Uuid,
    tag: String,
    inner_html: Option<String>,
    on_click: Option<String>,
    bind: Option<Box<dyn FnMut(Option<String>)>>,
    children: Vec<NodeRef>,
}

impl Node {
    pub fn new(tag: &str) -> Self {
        Self {
            event_key: Uuid::new_v4(),
            tag: tag.to_string(),
            inner_html: None,
            on_click: None,
            bind: None,
            children: vec![],
        }
    }
    
    pub fn text(mut self, text: &str) -> Self {
        self.inner_html = Some(text.to_string());
        self
    }
    
    pub fn child(mut self, child: NodeRef) -> Self {
        self.children.push(child);
        self
    }
    
    pub fn on_click(mut self, message_id: &str) -> Self
    {
        self.on_click = Some(message_id.to_string());
        self
    }

    pub fn bind(mut self, ref_str: Rc<RefCell<String>>) -> Self {
        let bind_closure = Box::new(move |new_value: Option<String>| {
            if let Some(new_value) = new_value {
                *ref_str.borrow_mut() = new_value;
            }
        });
        self.bind = Some(bind_closure);
        self
    }

    pub fn into_ref(self) -> NodeRef {
        Rc::new(RefCell::new(self))
    }
}

pub fn render_node(node: &NodeRef, document: &Document) -> Element {
    let node = node.borrow();
    let elem = document.create_element(&node.tag).unwrap();
    elem.set_attribute(EVENT_CUSTOM_DATA_KEY, &node.event_key.to_string()).unwrap();
    match &node.inner_html {
        Some(html) => elem.set_inner_html(html),
        None => {
            for child in &node.children {
                let child_elem = render_node(child, document);
                elem.append_child(&child_elem).unwrap();
            }
        }
    }
    elem
}

pub fn dispatch_event(node: &NodeRef, uuid_str: &str, event: &Event) -> Option<String> {
    let mut node = node.borrow_mut();
    if node.event_key.to_string() == uuid_str {
        match event {
            Event::Click => {
                if let Some(on_click) = &mut node.on_click {
                    return on_click.clone().into();
                }
            },
            Event::Change(value) => {
                if let Some(bind) = &mut node.bind {
                    bind(value.clone());
                    // bindは再レンダリングをトリガーしないため、明示的にNoneを返す
                    return None;
                }
            },
            _ => {}
        }
    } else {
        for child in &node.children {
            if let Some(result) = dispatch_event(child, uuid_str, event) {
                return Some(result);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use regex::Regex;
    use crate::node::Node;
    use crate::test_helper::tests::{document, minify_html};
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn render_node_tree_to_element() {
        let doc = document();
        let node_ref = 
            Node::new("div")
                .child(
                    Node::new("ul")
                        .child(
                            Node::new("li")
                                .text("item1")
                                .into_ref()
                        )
                        .child(
                            Node::new("li")
                                .text("item2")
                                .into_ref()
                        )
                        .into_ref()
                )
                .into_ref();

        let rendered = render_node(&node_ref, &doc);

        let expected_html = r#"
            <div data-uuid=".*?">
                <ul data-uuid=".*?">
                    <li data-uuid=".*?">item1</li>
                    <li data-uuid=".*?">item2</li>
                </ul>
            </div>
        "#;
        let minified = minify_html(expected_html);
        let re = Regex::new(&minified).unwrap();
        assert!(re.is_match(&rendered.outer_html()));
    }
    
    #[wasm_bindgen_test]
    fn dispatch_event_to_node() {
        let node_ref = 
            Node::new("div")
                .child(
                    Node::new("button")
                        .on_click("button_click")
                        .into_ref()
                )
                .child(
                    Node::new("button")
                        .into_ref()
                )
                .into_ref();
        
        let button_node1 = node_ref.borrow().children[0].clone();
        let button_node2 = node_ref.borrow().children[1].clone();
        let button_uuid1 = button_node1.borrow().event_key.to_string();
        let button_uuid2 = button_node2.borrow().event_key.to_string();
        
        let dispatched = dispatch_event(&node_ref, &button_uuid1, &Event::Click);
        assert_eq!(dispatched, Some("button_click".to_string()), "Event should be dispatched to button node");
        
        let dispatched = dispatch_event(&node_ref, &button_uuid2, &Event::Click);
        assert!(dispatched.is_none(), "Event should not be dispatched to button node without handler");
        
        let fake_uuid = Uuid::new_v4().to_string();
        let dispatched = dispatch_event(&node_ref, &fake_uuid, &Event::Click);
        assert!(dispatched.is_none(), "Event should not be dispatched to any node");
    }
    
    #[wasm_bindgen_test]
    fn dispatch_change_event_updates_bound_value() {
        let bound_value = Rc::new(RefCell::new(String::new()));
        let node_ref = 
            Node::new("input")
                .bind(bound_value.clone())
                .into_ref();

        let input_uuid = node_ref.borrow().event_key.to_string();
        let new_value = "New input value".to_string();
        let dispatched = dispatch_event(&node_ref, &input_uuid, &Event::Change(Some(new_value.clone())));
        assert!(dispatched.is_none(), "Change event should not trigger re-rendering");
        assert_eq!(*bound_value.borrow(), new_value, "Bound value should be updated");
    }
}
