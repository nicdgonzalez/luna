use crate::store::FileStore;

#[derive(Debug)]
pub struct Context {
    pub store: FileStore,
}

impl Context {
    #[must_use]
    pub const fn new(store: FileStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &FileStore {
        &self.store
    }
}
