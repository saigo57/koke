use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::Element;
use uuid::Uuid;
use crate::state::StateStore;

pub struct Context {
    uuid_map: RefCell<HashMap<String, Uuid>>,
    current_cycle_keys: RefCell<Vec<String>>,
    root_elm: Element,

    pub state: StateStore,
}

impl Context {
    pub fn new(root_elm: Element) -> Self {
        Self {
            uuid_map: RefCell::new(HashMap::new()),
            current_cycle_keys: RefCell::new(vec![]),
            state: StateStore::new(),
            root_elm: root_elm,
        }
    }
    
    pub fn key_uuid(&self, key: &str) -> Uuid {
        let (uuid, _is_new) = self.key_uuid_with_flag(key);
        uuid
    }
    
    // keyに対応するUUIDを返す。新規作成された場合はtrueを返す
    pub fn key_uuid_with_flag(&self, key: &str) -> (Uuid, bool) {
        self.current_cycle_keys.borrow_mut().push(key.to_string());

        let mut map = self.uuid_map.borrow_mut();
        if let Some(uuid) = map.get(key) {
            (*uuid, false)
        } else {
            let new_uuid = Uuid::new_v4();
            map.insert(key.to_string(), new_uuid);
            (new_uuid, true)
        }   
    }
    
    pub fn remove_unused_keys(&self) {
        let current_keys = self.current_cycle_keys.borrow();
        let mut map = self.uuid_map.borrow_mut();
        map.retain(|k, _| current_keys.contains(k));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use crate::test_helper::tests::document;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_context_key_uuid() {
        let doc = document();
        let root_elm = doc.create_element("div").unwrap();
        let ctx = Context::new(root_elm);
        
        let key1 = "component1";
        let key2 = "component2";
        
        let uuid1_first = ctx.key_uuid(key1);
        let uuid2_first = ctx.key_uuid(key2);
        let uuid1_second = ctx.key_uuid(key1);
        
        assert_eq!(uuid1_first, uuid1_second, "UUIDs for the same key should be equal");
        assert_ne!(uuid1_first, uuid2_first, "UUIDs for different keys should not be equal");
    }
}
