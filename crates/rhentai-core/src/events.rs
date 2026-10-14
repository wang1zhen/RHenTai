use eh_client::AuthStatus;

use crate::model::DownloadProgress;

pub trait EventSink: Send + Sync + 'static {
    fn emit(&self, event: CoreEvent);
}

#[derive(Debug, Clone)]
pub enum CoreEvent {
    DownloadProgress(DownloadProgress),
    DownloadsChanged,
    LibraryChanged,
    AuthChanged(AuthStatus),
}

#[derive(Default)]
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    fn emit(&self, _event: CoreEvent) {}
}
