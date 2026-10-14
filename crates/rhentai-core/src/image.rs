use std::path::Path;

use eh_client::EhClient;

#[derive(Clone)]
pub struct ImageService {
    #[allow(dead_code)]
    client: EhClient,
}

impl ImageService {
    pub fn new(_cache_dir: &Path, client: EhClient) -> Self {
        Self { client }
    }
}
