use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

pub struct StateStore {
    states: RefCell<HashMap<Uuid, Rc<dyn Any>>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            states: RefCell::new(HashMap::new()),
        }
    }
    
    pub fn use_state<T: 'static>(&self, uuid: Uuid, default: T) -> Rc<RefCell<T>> {
        let mut map = self.states.borrow_mut();

        if let Some(state) = map.get(&uuid) {
            if let Some(existing) = state.clone().downcast::<RefCell<T>>().ok() {
                return existing;
            }
        }
        
        let new_state = Rc::new(RefCell::new(default));
        map.insert(uuid, new_state.clone() as Rc<dyn Any>);
        new_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_use_state() {
        let state_store = StateStore::new();
        let uuid1 = Uuid::new_v4();
        let state1 = state_store.use_state(uuid1, 10);
        assert_eq!(*state1.borrow(), 10);

        let uuid2 = Uuid::new_v4();
        let state2 = state_store.use_state(uuid2, 100);
        assert_eq!(*state2.borrow(), 100);
        
        *state1.borrow_mut() = 20;
        *state2.borrow_mut() = 200;

        // 状態が保持されていることを確認
        let next_state1 = state_store.use_state(uuid1, 10);
        assert_eq!(*next_state1.borrow(), 20);
        let next_state2 = state_store.use_state(uuid2, 100);
        assert_eq!(*next_state2.borrow(), 200);

        // UUIDが異なれば新しい状態が作成されることを確認
        let uuid3 = Uuid::new_v4();
        let state3 = state_store.use_state(uuid3, 100);
        assert_eq!(*state3.borrow(), 100);
    }
}
