import { invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "@/bindings/AuthStatus";
import type { BatchFavorite } from "@/bindings/BatchFavorite";
import type { CacheCleared } from "@/bindings/CacheCleared";
import type { CookieLogin } from "@/bindings/CookieLogin";
import type { DownloadTask } from "@/bindings/DownloadTask";
import type { EnqueueRequest } from "@/bindings/EnqueueRequest";
import type { FavoriteCategory } from "@/bindings/FavoriteCategory";
import type { FavoriteUpdate } from "@/bindings/FavoriteUpdate";
import type { GalleryDetail } from "@/bindings/GalleryDetail";
import type { GalleryKey } from "@/bindings/GalleryKey";
import type { GalleryListPage } from "@/bindings/GalleryListPage";
import type { HistoryEntry } from "@/bindings/HistoryEntry";
import type { IpcError } from "@/bindings/IpcError";
import type { ListRequest } from "@/bindings/ListRequest";
import type { LocalGallery } from "@/bindings/LocalGallery";
import type { LocalId } from "@/bindings/LocalId";
import type { LocalPages } from "@/bindings/LocalPages";
import type { PageImageKey } from "@/bindings/PageImageKey";
import type { PasswordLogin } from "@/bindings/PasswordLogin";
import type { PathRequest } from "@/bindings/PathRequest";
import type { ReadProgress } from "@/bindings/ReadProgress";
import type { RemoveDownload } from "@/bindings/RemoveDownload";
import type { RemoveLocal } from "@/bindings/RemoveLocal";
import type { Settings } from "@/bindings/Settings";
import type { TaskId } from "@/bindings/TaskId";
import type { ThumbnailPage } from "@/bindings/ThumbnailPage";
import type { ThumbnailRequest } from "@/bindings/ThumbnailRequest";

async function call<R>(cmd: string, args?: Record<string, unknown>): Promise<R> {
  try {
    return await invoke<R>(cmd, args);
  } catch (err) {
    throw err as IpcError;
  }
}

export async function authStatus(): Promise<AuthStatus> {
  return call<AuthStatus>("auth_status");
}

export async function loginPassword(req: PasswordLogin): Promise<AuthStatus> {
  return call<AuthStatus>("login_password", { req });
}

export async function loginCookies(req: CookieLogin): Promise<AuthStatus> {
  return call<AuthStatus>("login_cookies", { req });
}

export async function loginWeb(): Promise<void> {
  return call<void>("login_web");
}

export async function logout(): Promise<AuthStatus> {
  return call<AuthStatus>("logout");
}

export async function getSettings(): Promise<Settings> {
  return call<Settings>("get_settings");
}

export async function updateSettings(req: Settings): Promise<Settings> {
  return call<Settings>("update_settings", { req });
}

export async function listGalleries(req: ListRequest): Promise<GalleryListPage> {
  return call<GalleryListPage>("list_galleries", { req });
}

export async function galleryDetail(req: GalleryKey): Promise<GalleryDetail> {
  return call<GalleryDetail>("gallery_detail", { req });
}

export async function galleryThumbnails(
  req: ThumbnailRequest,
): Promise<ThumbnailPage> {
  return call<ThumbnailPage>("gallery_thumbnails", { req });
}

export async function reloadPageImage(req: PageImageKey): Promise<void> {
  return call<void>("reload_page_image", { req });
}

export async function favoriteCategories(): Promise<FavoriteCategory[]> {
  return call<FavoriteCategory[]>("favorite_categories");
}

export async function setFavorite(req: FavoriteUpdate): Promise<void> {
  return call<void>("set_favorite", { req });
}

export async function batchFavorites(req: BatchFavorite): Promise<void> {
  return call<void>("batch_favorites", { req });
}

export async function enqueueDownloads(
  req: EnqueueRequest,
): Promise<DownloadTask[]> {
  return call<DownloadTask[]>("enqueue_downloads", { req });
}

export async function listDownloads(): Promise<DownloadTask[]> {
  return call<DownloadTask[]>("list_downloads");
}

export async function pauseDownload(req: TaskId): Promise<void> {
  return call<void>("pause_download", { req });
}

export async function resumeDownload(req: TaskId): Promise<void> {
  return call<void>("resume_download", { req });
}

export async function removeDownload(req: RemoveDownload): Promise<void> {
  return call<void>("remove_download", { req });
}

export async function pauseAll(): Promise<void> {
  return call<void>("pause_all");
}

export async function resumeAll(): Promise<void> {
  return call<void>("resume_all");
}

export async function listLibrary(): Promise<LocalGallery[]> {
  return call<LocalGallery[]>("list_library");
}

export async function rescanLibrary(): Promise<LocalGallery[]> {
  return call<LocalGallery[]>("rescan_library");
}

export async function localPages(req: LocalId): Promise<LocalPages> {
  return call<LocalPages>("local_pages", { req });
}

export async function addLibraryDir(req: PathRequest): Promise<Settings> {
  return call<Settings>("add_library_dir", { req });
}

export async function removeLocal(req: RemoveLocal): Promise<void> {
  return call<void>("remove_local", { req });
}

export async function listHistory(): Promise<HistoryEntry[]> {
  return call<HistoryEntry[]>("list_history");
}

export async function recordProgress(req: ReadProgress): Promise<void> {
  return call<void>("record_progress", { req });
}

export async function clearHistory(): Promise<void> {
  return call<void>("clear_history");
}

export async function openPath(req: PathRequest): Promise<void> {
  return call<void>("open_path", { req });
}

export async function clearImageCache(): Promise<CacheCleared> {
  return call<CacheCleared>("clear_image_cache");
}
