import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AuthStatus } from "@/bindings/AuthStatus";
import type { DownloadProgress } from "@/bindings/DownloadProgress";

export function onDownloadProgress(
  handler: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) =>
    handler(event.payload),
  );
}

export function onDownloadsChanged(handler: () => void): Promise<UnlistenFn> {
  return listen<null>("downloads-changed", () => handler());
}

export function onLibraryChanged(handler: () => void): Promise<UnlistenFn> {
  return listen<null>("library-changed", () => handler());
}

export function onAuthChanged(
  handler: (status: AuthStatus) => void,
): Promise<UnlistenFn> {
  return listen<AuthStatus>("auth-changed", (event) => handler(event.payload));
}
