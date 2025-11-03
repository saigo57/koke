use uuid::Uuid;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{Document, Element};

pub type NodeRef = Rc<RefCell<Node>>;
pub struct Node {
    event_key: Uuid,
    tag: String,
    inner_html: Option<String>,
    value: Option<String>,
    on_click: Option<Box<dyn FnMut()>>,
    bind: Option<Box<dyn FnMut(String)>>,
    children: Vec<NodeRef>,
}

impl Node {
    pub fn new(tag: &str) -> Self {
        Self {
            event_key: Uuid::new_v4(),
            tag: tag.to_string(),
            inner_html: None,
            value: None,
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
    
    pub fn into_ref(self) -> NodeRef {
        Rc::new(RefCell::new(self))
    }
}

pub fn render_node(node: &NodeRef, document: &Document) -> Element {
    let node = node.borrow();
    let elem = document.create_element(&node.tag).unwrap();
    elem.set_attribute("data-uuid", &node.event_key.to_string()).unwrap();
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
