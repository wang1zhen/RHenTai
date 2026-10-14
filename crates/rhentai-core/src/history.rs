use crate::store::Store;

#[derive(Clone)]
pub struct History {
    #[allow(dead_code)]
    store: Store,
}

impl History {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}
