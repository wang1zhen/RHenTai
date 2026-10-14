use rhentai_core::{CoreEvent, EventSink};
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
pub struct TauriSink(pub AppHandle);

impl EventSink for TauriSink {
    fn emit(&self, event: CoreEvent) {
        match event {
            CoreEvent::DownloadProgress(p) => {
                let _ = self.0.emit("download-progress", p);
            }
            CoreEvent::DownloadsChanged => {
                let _ = self.0.emit("downloads-changed", ());
            }
            CoreEvent::LibraryChanged => {
                let _ = self.0.emit("library-changed", ());
            }
            CoreEvent::AuthChanged(a) => {
                let _ = self.0.emit("auth-changed", a);
            }
        }
    }
}
