use std::sync::Arc;

use eh_client::EhClient;

use crate::events::EventSink;
use crate::store::Store;

#[derive(Clone)]
pub struct DownloadManager {
    #[allow(dead_code)]
    store: Store,
    #[allow(dead_code)]
    client: EhClient,
    #[allow(dead_code)]
    sink: Arc<dyn EventSink>,
}

impl DownloadManager {
    pub fn new(store: Store, client: EhClient, sink: Arc<dyn EventSink>) -> Self {
        Self {
            store,
            client,
            sink,
        }
    }
}
