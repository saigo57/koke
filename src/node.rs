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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use regex::Regex;
    
    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }
    
    fn minify_html(input: &str) -> String {
        let re = Regex::new(r"\s*\n\s*").unwrap(); // 改行の前後の空白を含めて除去
        re.replace_all(input, "").into_owned()
    }
    
    #[wasm_bindgen_test]
    fn render_node_sets_inner_html_for_text_nodes() {
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
}
