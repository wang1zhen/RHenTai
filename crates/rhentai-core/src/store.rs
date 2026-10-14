use std::path::Path;

use crate::error::Result;

#[derive(Clone, Default)]
pub struct Store;

impl Store {
    pub fn open(_data_dir: &Path) -> Result<Self> {
        Ok(Self)
    }
}
