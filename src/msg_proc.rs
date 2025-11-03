use web_sys::console;
use wasm_bindgen::prelude::*;
use crate::event::Event;
use crate::node::{NodeRef, render_node};

pub fn registr_msg_proc(
    ui_func: fn() -> NodeRef,
    body: &web_sys::HtmlElement,
    document: &web_sys::Document,
    root_elm: &web_sys::Element,
) {
    // cloneしたものをclosureにキャプチャして貰う必要がある
    let document = document.clone();
    let body = body.clone();
    let root_elm = root_elm.clone();
    
    let msg_proc = move |e: web_sys::Event| {
        let event = Event::from_event(&e);
        log_event(&event);
        
        let node = ui_func();
        let content = render_node(&node, &document);
        root_elm.set_inner_html("");
        root_elm.append_child(&content).unwrap();
    };
    let closure = Closure::wrap(Box::new(msg_proc) as Box<dyn FnMut(_)>);
    
    // Kokeで定義したEventに対して、msg_procを登録する
    for event in Event::all_events() {
        let event_str = event.to_str();
        match body.add_event_listener_with_callback(event_str, closure.as_ref().unchecked_ref()) {
            Ok(_) => (),
            Err(_) => {
                console::error_1(&format!("Failed to add event listener for event: {}", event_str).into());
            }
        }
    }
    // js側でイベントが発生するまで保持しておく必要があるのでforgetする
    closure.forget();
}

fn log_event(event: &Option<Event>) {
    match event {
        Some(Event::Click) => console::log_1(&"Click event received!!!".into()),
        Some(Event::Change(value)) => {
            console::log_1(&format!("Change event received with value: {:?}", &value).into())
        },
        Some(Event::KeyDown) => console::log_1(&"KeyDown event received".into()),
        Some(Event::RustRender) => console::log_1(&"RustRender event received".into()),
        None => console::log_1(&"Unknown event received".into()),
    }
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
    fn execute_rendering_if_event_triggered() {
        let document = document();
        let body = document.body().unwrap();
        let root_elm = document.create_element("div").unwrap();
        root_elm.set_id("root");
        body.append_child(&root_elm).unwrap();

        let ui_func = || {
            Node::new("div")
                .text("Hello, Koke!")
                .into_ref()
        };
        registr_msg_proc(ui_func, &body, &document, &root_elm);
        
        // まず、root_elmの中身は空であることを確認
        assert_eq!(root_elm.inner_html(), "");
        
        // Eventをトリガーしてレンダリングを実行
        Event::trigger_render_event(&body);

        // root_elmの中身にレンダリング結果が反映されていることを確認 
        let expected_html = r#"
            <div data-uuid=".*?">Hello, Koke!</div>
        "#;
        let minified = minify_html(expected_html);
        let re = Regex::new(&minified).unwrap();
        assert!(re.is_match(&root_elm.inner_html()));
    }
}
