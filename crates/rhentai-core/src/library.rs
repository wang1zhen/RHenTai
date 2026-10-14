use crate::store::Store;

#[derive(Clone)]
pub struct Library {
    #[allow(dead_code)]
    store: Store,
}

impl Library {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}
