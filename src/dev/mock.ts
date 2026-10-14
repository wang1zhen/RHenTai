import { mockIPC } from "@tauri-apps/api/mocks";
import type { AuthStatus } from "@/bindings/AuthStatus";
import type { Category } from "@/bindings/Category";
import type { DownloadTask } from "@/bindings/DownloadTask";
import type { FavoriteCategory } from "@/bindings/FavoriteCategory";
import type { GalleryDetail } from "@/bindings/GalleryDetail";
import type { GallerySummary } from "@/bindings/GallerySummary";
import type { HistoryEntry } from "@/bindings/HistoryEntry";
import type { LocalGallery } from "@/bindings/LocalGallery";
import type { LocalPages } from "@/bindings/LocalPages";
import type { Settings } from "@/bindings/Settings";
import type { ThumbnailPage } from "@/bindings/ThumbnailPage";

// Load fixture data
import initialGalleries from "./fixtures/galleries.json";
import initialDetail from "./fixtures/detail.json";
import initialThumbnails from "./fixtures/thumbnails.json";
import initialSettings from "./fixtures/settings.json";
import initialAuth from "./fixtures/auth.json";
import initialFavorites from "./fixtures/favorites.json";
import initialDownloads from "./fixtures/downloads.json";
import initialLibrary from "./fixtures/library.json";
import initialHistory from "./fixtures/history.json";
import initialLocalPages from "./fixtures/local_pages.json";

export function setupMock() {
  console.log("%c[RHenTai Dev Mock Mode Active]", "color: #38bdf8; font-weight: bold; font-size: 14px;");

  // In-memory mutable state
  let galleries: GallerySummary[] = JSON.parse(JSON.stringify(initialGalleries));
  let detail: GalleryDetail = JSON.parse(JSON.stringify(initialDetail));
  let thumbnails: ThumbnailPage = JSON.parse(JSON.stringify(initialThumbnails));
  let settings: Settings = JSON.parse(JSON.stringify(initialSettings));
  let auth: AuthStatus = JSON.parse(JSON.stringify(initialAuth));
  let favorites: FavoriteCategory[] = JSON.parse(JSON.stringify(initialFavorites));
  let downloads: DownloadTask[] = JSON.parse(JSON.stringify(initialDownloads));
  let library: LocalGallery[] = JSON.parse(JSON.stringify(initialLibrary));
  let history: HistoryEntry[] = JSON.parse(JSON.stringify(initialHistory));
  let localPagesData: LocalPages = JSON.parse(JSON.stringify(initialLocalPages));

  // Helper to recompute category counts from actual galleries state
  function updateFavoriteCategoryCounts() {
    const counts: Record<number, number> = {};
    for (let i = 0; i < 10; i++) counts[i] = 0;
    for (const g of galleries) {
      if (
        g.favoriteSlot !== null &&
        g.favoriteSlot !== undefined &&
        counts[g.favoriteSlot] !== undefined
      ) {
        counts[g.favoriteSlot]++;
      }
    }
    favorites = favorites.map((cat) => ({
      ...cat,
      count: counts[cat.slot] ?? 0,
    }));
  }

  // Assign favorite slots/notes to a deterministic subset of fixture galleries
  galleries = galleries.map((g, index) => {
    if (index === 0) return { ...g, favoriteSlot: 0, favoriteNote: "Must read" };
    if (index === 1) return { ...g, favoriteSlot: 0, favoriteNote: "Favorite 0 note" };
    if (index === 2) return { ...g, favoriteSlot: 0, favoriteNote: null };
    if (index === 3) return { ...g, favoriteSlot: 0, favoriteNote: "Top tier" };
    if (index === 4) return { ...g, favoriteSlot: 1, favoriteNote: "Read later" };
    if (index === 5) return { ...g, favoriteSlot: 1, favoriteNote: null };
    if (index === 6) return { ...g, favoriteSlot: 2, favoriteNote: "Masterpiece" };
    if (index === 7) return { ...g, favoriteSlot: 2, favoriteNote: "Art style" };
    if (index === 8) return { ...g, favoriteSlot: 3, favoriteNote: null };
    if (index === 9) return { ...g, favoriteSlot: 4, favoriteNote: "CG set" };
    return { ...g, favoriteSlot: null, favoriteNote: null };
  });

  if (galleries[0] && detail.summary.gid === galleries[0].gid) {
    detail.summary.favoriteSlot = galleries[0].favoriteSlot;
    detail.summary.favoriteNote = galleries[0].favoriteNote;
  }

  updateFavoriteCategoryCounts();

  // Registered event listeners for mock event emission
  const eventListeners = new Map<string, Array<(payload: unknown) => void>>();

  function emitEvent(event: string, payload: unknown) {
    const list = eventListeners.get(event);
    if (list) {
      for (const cb of list) {
        cb(payload);
      }
    }
  }

  // Periodic download progress simulator
  setInterval(() => {
    let hasRunning = false;
    for (const task of downloads) {
      if (task.state === "running") {
        hasRunning = true;
        if (task.done < task.total) {
          task.done += 1;
          task.updatedAt = Math.floor(Date.now() / 1000);
          emitEvent("download-progress", {
            id: task.id,
            done: task.done,
            total: task.total,
            failed: task.failed,
            state: task.state,
            error: task.error,
            bytesPerSec: 1540000,
          });
        } else {
          task.state = "completed";
          emitEvent("download-progress", {
            id: task.id,
            done: task.done,
            total: task.total,
            failed: task.failed,
            state: "completed",
            error: null,
            bytesPerSec: 0,
          });
          emitEvent("downloads-changed", null);
        }
      }
    }
  }, 1000);

  // Install Tauri mock IPC
  mockIPC(async (cmd, args) => {
    // Event plugin handling
    if (cmd === "plugin:event|listen") {
      const { event, handler } = (args as { event: string; handler: (p: unknown) => void });
      if (!eventListeners.has(event)) {
        eventListeners.set(event, []);
      }
      eventListeners.get(event)!.push(handler);
      return handler;
    }

    if (cmd === "plugin:event|emit") {
      const { event, payload } = (args as { event: string; payload: unknown });
      emitEvent(event, payload);
      return null;
    }

    if (cmd === "plugin:dialog|open") {
      return "/home/user/Downloads/RHenTai";
    }

    if (cmd.startsWith("plugin:opener|")) {
      return null;
    }

    // RHenTai Core Commands
    const req = (args as { req?: any })?.req;

    switch (cmd) {
      case "auth_status":
        return auth;

      case "login_password": {
        if (req.password === "cloudflare") {
          throw {
            kind: "cloudflare",
            message: "Cloudflare challenge detected",
          };
        }
        auth = {
          loggedIn: true,
          memberId: "1234567",
          username: req.username || "MockUser",
          exAccess: true,
        };
        emitEvent("auth-changed", auth);
        return auth;
      }

      case "login_cookies": {
        auth = {
          loggedIn: true,
          memberId: req.memberId,
          username: "CookieUser",
          exAccess: Boolean(req.igneous),
        };
        emitEvent("auth-changed", auth);
        return auth;
      }

      case "login_web": {
        auth = {
          loggedIn: true,
          memberId: "9876543",
          username: "WebUser",
          exAccess: true,
        };
        setTimeout(() => {
          emitEvent("auth-changed", auth);
        }, 1500);
        return null;
      }

      case "logout": {
        auth = {
          loggedIn: false,
          memberId: null,
          username: null,
          exAccess: false,
        };
        emitEvent("auth-changed", auth);
        return auth;
      }

      case "get_settings":
        return settings;

      case "update_settings": {
        settings = { ...settings, ...req };
        return settings;
      }

      case "list_galleries": {
        const { source, filter, cursor } = req;
        let items = [...galleries];

        // Filter source
        if (source.kind === "watched") {
          if (!auth.loggedIn) {
            throw { kind: "loginRequired", message: "Login required to view watched tags" };
          }
        } else if (source.kind === "favorites") {
          if (!auth.loggedIn) {
            throw { kind: "loginRequired", message: "Login required to view favorites" };
          }
          if (source.slot !== null && source.slot !== undefined) {
            items = items.filter((g) => g.favoriteSlot === source.slot);
          } else {
            items = items.filter((g) => g.favoriteSlot !== null);
          }
        }

        // Apply SearchFilter
        if (filter) {
          if (filter.keyword) {
            const kw = filter.keyword.toLowerCase();
            items = items.filter(
              (g) =>
                g.title.toLowerCase().includes(kw) ||
                (g.uploader && g.uploader.toLowerCase().includes(kw)) ||
                g.tags.some((t) => t.name.toLowerCase().includes(kw) || t.namespace.toLowerCase().includes(kw)),
            );
          }
          if (filter.excludedCategories && filter.excludedCategories.length > 0) {
            items = items.filter((g) => !filter.excludedCategories.includes(g.category));
          }
          if (filter.minRating !== null && filter.minRating !== undefined) {
            items = items.filter((g) => g.rating >= filter.minRating);
          }
          if (filter.minPages !== null && filter.minPages !== undefined) {
            items = items.filter((g) => g.fileCount >= filter.minPages);
          }
          if (filter.maxPages !== null && filter.maxPages !== undefined) {
            items = items.filter((g) => g.fileCount <= filter.maxPages);
          }
        }

        // Cursor pagination simulation
        const pageSize = 12;
        let startIndex = 0;
        if (cursor && cursor.startsWith("page:")) {
          const pageNum = parseInt(cursor.slice(5), 10);
          startIndex = pageNum * pageSize;
        }

        const pageItems = items.slice(startIndex, startIndex + pageSize);
        const hasNext = startIndex + pageSize < items.length;
        const nextCursor = hasNext ? `page:${Math.floor(startIndex / pageSize) + 1}` : null;

        return {
          items: pageItems,
          next: source.kind === "popular" ? null : nextCursor,
          prev: startIndex > 0 ? `page:${Math.floor(startIndex / pageSize) - 1}` : null,
        };
      }

      case "gallery_detail": {
        const { gid } = req;
        const found = galleries.find((g) => g.gid === gid);
        if (found) {
          return {
            ...detail,
            summary: found,
          };
        }
        return detail;
      }

      case "gallery_thumbnails": {
        return thumbnails;
      }

      case "reload_page_image": {
        return null;
      }

      case "favorite_categories": {
        if (!auth.loggedIn) {
          throw { kind: "loginRequired", message: "Login required to access favorites" };
        }
        return favorites;
      }

      case "set_favorite": {
        const { gid, slot, note } = req;
        galleries = galleries.map((g) => {
          if (g.gid === gid) {
            return { ...g, favoriteSlot: slot, favoriteNote: note ?? null };
          }
          return g;
        });
        if (detail.summary.gid === gid) {
          detail.summary.favoriteSlot = slot;
          detail.summary.favoriteNote = note ?? null;
        }
        updateFavoriteCategoryCounts();
        return null;
      }

      case "batch_favorites": {
        const { gids, slot } = req;
        const gidSet = new Set(gids);
        galleries = galleries.map((g) => {
          if (gidSet.has(g.gid)) {
            return { ...g, favoriteSlot: slot };
          }
          return g;
        });
        if (gidSet.has(detail.summary.gid)) {
          detail.summary.favoriteSlot = slot;
        }
        updateFavoriteCategoryCounts();
        return null;
      }

      case "enqueue_downloads": {
        const { galleries: reqGalleries } = req;
        for (const item of reqGalleries) {
          const found = galleries.find((g) => g.gid === item.gid);
          const existing = downloads.find((d) => d.gid === item.gid);
          if (!existing && found) {
            downloads.push({
              id: Date.now() + Math.floor(Math.random() * 1000),
              gid: found.gid,
              token: found.token,
              site: "eh",
              title: found.title,
              thumb: found.thumb,
              category: found.category,
              total: found.fileCount,
              done: 0,
              failed: 0,
              state: "running",
              error: null,
              dir: `/home/user/Downloads/RHenTai/${found.gid} - ${found.title.slice(0, 30)}`,
              original: false,
              createdAt: Math.floor(Date.now() / 1000),
              updatedAt: Math.floor(Date.now() / 1000),
            });
          }
        }
        emitEvent("downloads-changed", null);
        return downloads;
      }

      case "list_downloads":
        return downloads;

      case "pause_download": {
        const { id } = req;
        downloads = downloads.map((d) => (d.id === id ? { ...d, state: "paused" } : d));
        emitEvent("downloads-changed", null);
        return null;
      }

      case "resume_download": {
        const { id } = req;
        downloads = downloads.map((d) => (d.id === id ? { ...d, state: "running" } : d));
        emitEvent("downloads-changed", null);
        return null;
      }

      case "remove_download": {
        const { id } = req;
        downloads = downloads.filter((d) => d.id !== id);
        emitEvent("downloads-changed", null);
        return null;
      }

      case "pause_all": {
        downloads = downloads.map((d) => (d.state === "running" ? { ...d, state: "paused" } : d));
        emitEvent("downloads-changed", null);
        return null;
      }

      case "resume_all": {
        downloads = downloads.map((d) => (d.state === "paused" ? { ...d, state: "running" } : d));
        emitEvent("downloads-changed", null);
        return null;
      }

      case "list_library":
        return library;

      case "rescan_library": {
        emitEvent("library-changed", null);
        return library;
      }

      case "local_pages":
        return localPagesData;

      case "add_library_dir": {
        const { path: newPath } = req;
        if (!settings.libraryDirs.includes(newPath)) {
          settings.libraryDirs.push(newPath);
        }
        return settings;
      }

      case "remove_local": {
        const { id } = req;
        library = library.filter((l) => l.id !== id);
        emitEvent("library-changed", null);
        return null;
      }

      case "list_history":
        return history;

      case "record_progress": {
        const { target, page, pageCount, title, thumb, category } = req;
        const existingIdx = history.findIndex((h) => {
          if (target.kind === "online" && h.target.kind === "online") {
            return target.gid === h.target.gid;
          }
          if (target.kind === "local" && h.target.kind === "local") {
            return target.id === h.target.id;
          }
          return false;
        });

        const entry: HistoryEntry = {
          target,
          title,
          thumb,
          category,
          pageCount,
          lastPage: page,
          readAt: Math.floor(Date.now() / 1000),
        };

        if (existingIdx !== -1) {
          history[existingIdx] = entry;
        } else {
          history.unshift(entry);
        }
        return null;
      }

      case "clear_history": {
        history = [];
        return null;
      }

      case "open_path": {
        console.log("[Mock IPC] open_path:", req.path);
        return null;
      }

      case "clear_image_cache": {
        return { bytes: 52428800 }; // 50 MB
      }

      default:
        console.warn(`[Mock IPC] Unhandled command: ${cmd}`, args);
        return null;
    }
  });

  // Mock convertFileSrc for rhimg:// protocol
  const win = window as any;
  win.__TAURI_INTERNALS__ = win.__TAURI_INTERNALS__ || {};
  win.__TAURI_INTERNALS__.convertFileSrc = (filePath: string, protocol = "asset") => {
    if (protocol === "rhimg") {
      // Decode base64 thumb route to original remote URL
      if (filePath.startsWith("thumb/")) {
        const b64 = filePath.slice(6);
        try {
          let base64 = b64.replace(/-/g, "+").replace(/_/g, "/");
          while (base64.length % 4) base64 += "=";
          const decoded = atob(base64);
          return decoded;
        } catch {
          // ignore error
        }
      }
      // For page and local routes, resolve to a realistic fixture image
      return "https://ehgt.org/w/02/714/57443-ao8ipua7.webp";
    }
    return filePath;
  };
}
