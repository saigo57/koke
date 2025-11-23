use web_sys::console;
use wasm_bindgen::prelude::*;
use crate::event::Event;
use crate::node::{
    Node, 
    NodeRef,
    render_node,
    dispatch_event,
    EVENT_CUSTOM_DATA_KEY,
};
use crate::context::Context;
use crate::UiFunction;

pub fn registr_msg_proc(
    ui_func: UiFunction,
    body: &web_sys::HtmlElement,
    document: &web_sys::Document,
    root_elm: &web_sys::Element,
) {
    // cloneしたものをclosureにキャプチャして貰う必要がある
    let document = document.clone();
    let body = body.clone();
    let root_elm = root_elm.clone();
    let root_node: NodeRef = Node::new("div").into_ref(); // 一旦適当な初期値を入れておく
    
    let ctx = Context::new(root_elm.clone());

    let msg_proc = move |e: web_sys::Event| {
        let event = Event::from_event(&e);
        log_event(&event);

        let uuid_str = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.get_attribute(EVENT_CUSTOM_DATA_KEY));
        if let (Some(uuid_str), Some(event)) = (uuid_str, event) {
            // イベントをdispatchして、レンダリングが必要なかったら抜ける
            let is_render = dispatch_event(&root_node, &uuid_str, &event);
            if !is_render {
                return;
            }
        }
        
        let new_tree = ui_func(&ctx);
        // 次のmsg_procで参照できるように、root_nodeとnew_treeを差し替える
        std::mem::swap(&mut *root_node.borrow_mut(), &mut *new_tree.borrow_mut());

        let content = render_node(&root_node, &document);
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
    use crate::test_helper::tests::{document, minify_html, wait_js_event};
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn execute_rendering_if_event_triggered() {
        let document = document();
        let body = document.body().unwrap();
        let root_elm = document.create_element("div").unwrap();
        root_elm.set_id("root");
        body.append_child(&root_elm).unwrap();

        let ui_func = |_: &Context| {
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
    
    #[wasm_bindgen_test]
    async fn execute_rendering_if_click_event_triggered() {
        let document = document();
        let body = document.body().unwrap();
        let root_elm = document.create_element("div").unwrap();
        root_elm.set_id("root");
        body.append_child(&root_elm).unwrap();

        let ui_func = |_: &Context| {
            Node::new("div")
                .child(
                    Node::new("button")
                        .text("Click me1")
                        // イベントハンドラなし
                        .into_ref()
                )
                .child(
                    Node::new("button")
                        .text("Click me2")
                        .on_click(|| {
                            // イベントハンドラあり
                        })
                        .into_ref()
                )
                .into_ref()
        };
        registr_msg_proc(ui_func, &body, &document, &root_elm);
        
        // Eventをトリガーしてレンダリングを実行
        Event::trigger_render_event(&body);
        wait_js_event().await;

        // root_elmの中身にレンダリング結果が反映されていることを確認 
        let expected_html = r#"
            <div data-uuid=".*?">
                <button data-uuid=".*?">Click me1</button>
                <button data-uuid=".*?">Click me2</button>
            </div>
        "#;
        let minified = minify_html(expected_html);
        let re = Regex::new(&minified).unwrap();
        let prev_inner_html = root_elm.inner_html();
        assert!(re.is_match(&prev_inner_html));
        
        // イベントハンドラが無いときは再レンダリングされない
        let prev_inner_html = root_elm.inner_html();
        let button1 = root_elm.first_element_child().unwrap().first_element_child().unwrap();
        assert_eq!(button1.inner_html(), "Click me1");
        button1.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
        wait_js_event().await;
        assert_eq!(&prev_inner_html, &root_elm.inner_html(), "root_elm should not re-render");
        
        // イベントハンドラがあるときは再レンダリングされる
        let prev_inner_html = root_elm.inner_html();
        let button2 = root_elm.first_element_child().unwrap().last_element_child().unwrap();
        assert_eq!(button2.inner_html(), "Click me2");
        button2.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
        wait_js_event().await;
        assert_ne!(&prev_inner_html, &root_elm.inner_html(), "root_elm should re-render");
    }
}
