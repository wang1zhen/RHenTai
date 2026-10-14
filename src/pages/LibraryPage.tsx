import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  BookOpen,
  FolderPlus,
  RefreshCw,
  Search,
  Trash2,
  Folder,
  Archive,
  X,
  FileText,
  ImageOff,
} from "lucide-react";
import { listLibrary, rescanLibrary, addLibraryDir, removeLocal } from "@/api/ipc";
import { onLibraryChanged } from "@/api/events";
import { localPageUrl } from "@/api/rhimg";
import { CategoryBadge } from "@/components/CategoryBadge";
import { EmptyState } from "@/components/EmptyState";
import { ErrorState } from "@/components/ErrorState";
import { useToastStore } from "@/components/Toasts";
import { useT, type TranslationKey } from "@/i18n";
import type { LocalGallery } from "@/bindings/LocalGallery";

interface LibraryCardItemProps {
  gallery: LocalGallery;
  onRemove: (g: LocalGallery) => void;
}

const LibraryCardItem: React.FC<LibraryCardItemProps> = ({ gallery, onRemove }) => {
  const { t } = useT();
  const navigate = useNavigate();
  const [hasError, setHasError] = useState(false);
  const [retryKey, setRetryKey] = useState(0);
  const coverUrl = localPageUrl(gallery.id, 0);

  return (
    <div
      data-qa="library-card"
      className="group relative flex flex-col rounded-2xl overflow-hidden border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-xs hover:border-sky-500 hover:shadow-lg transition-all"
    >
      {/* Cover */}
      <div
        onClick={() =>
          navigate(
            `/read/local/${gallery.id}?page=${gallery.lastReadPage || 1}`,
          )
        }
        className="relative aspect-7/10 w-full overflow-hidden bg-gray-100 dark:bg-gray-800 cursor-pointer"
      >
        {hasError ? (
          <div
            role="button"
            tabIndex={0}
            title={t("retry")}
            aria-label={t("retry")}
            onClick={(e) => {
              e.stopPropagation();
              setHasError(false);
              setRetryKey((k) => k + 1);
            }}
            className="absolute inset-0 flex flex-col items-center justify-center gap-1.5 p-3 text-center bg-gray-100 dark:bg-gray-800 text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 transition-colors z-10 cursor-pointer"
          >
            <ImageOff className="w-8 h-8 text-gray-400 dark:text-gray-500" />
            <span className="text-[10px] font-semibold flex items-center gap-1 text-gray-500 dark:text-gray-400">
              <RefreshCw className="w-3 h-3" />
              {t("retry")}
            </span>
          </div>
        ) : (
          <img
            key={retryKey}
            src={retryKey > 0 ? `${coverUrl}?v=${retryKey}` : coverUrl}
            alt={gallery.title}
            loading="lazy"
            decoding="async"
            onError={() => setHasError(true)}
            className="h-full w-full object-cover group-hover:scale-105 transition-transform duration-300"
          />
        )}

        {/* Top Badges */}
        <div className="absolute top-2 left-2 flex items-center gap-1 z-10">
          {gallery.category && (
            <CategoryBadge category={gallery.category} size="sm" />
          )}
          <span className="p-1 rounded-md bg-black/60 backdrop-blur-xs text-white text-[10px]">
            {gallery.kind === "folder" ? (
              <Folder className="w-3 h-3" />
            ) : (
              <Archive className="w-3 h-3" />
            )}
          </span>
        </div>

        {/* Page Count */}
        <div className="absolute bottom-2 right-2 px-1.5 py-0.5 rounded-md bg-black/70 backdrop-blur-xs text-white text-[10px] font-medium flex items-center gap-1">
          <FileText className="w-3 h-3" />
          <span>{gallery.pageCount} {t("pages")}</span>
        </div>
      </div>

      {/* Info */}
      <div className="p-3 flex-1 flex flex-col justify-between">
        <h3
          onClick={() =>
            navigate(
              `/read/local/${gallery.id}?page=${gallery.lastReadPage || 1}`,
            )
          }
          className="font-semibold text-xs text-gray-900 dark:text-gray-100 line-clamp-2 cursor-pointer hover:text-sky-600 dark:hover:text-sky-400 mb-2"
          title={gallery.title}
        >
          {gallery.title}
        </h3>

        {/* Action buttons */}
        <div className="flex items-center justify-between pt-2 border-t border-gray-100 dark:border-gray-800">
          <button
            type="button"
            onClick={() =>
              navigate(
                `/read/local/${gallery.id}?page=${gallery.lastReadPage || 1}`,
              )
            }
            className="inline-flex items-center gap-1 px-2.5 py-1 text-xs font-semibold rounded-lg bg-sky-50 dark:bg-sky-950/50 text-sky-600 dark:text-sky-400 hover:bg-sky-100 cursor-pointer transition-colors"
          >
            <BookOpen className="w-3 h-3" />
            <span>{t("read")}</span>
          </button>

          <button
            type="button"
            onClick={() => onRemove(gallery)}
            className="p-1 rounded-lg text-gray-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors"
            title={t("remove")}
            aria-label={t("remove")}
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};

export const LibraryPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { showToast } = useToastStore();

  const [filterText, setFilterText] = useState("");
  const [isRescanning, setIsRescanning] = useState(false);
  const [galleryToRemove, setGalleryToRemove] = useState<LocalGallery | null>(null);
  const [deleteFilesOnDisk, setDeleteFilesOnDisk] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);

  const {
    data: library = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery({
    queryKey: ["library"],
    queryFn: listLibrary,
  });

  // Listen to library changed events
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    onLibraryChanged(() => {
      queryClient.invalidateQueries({ queryKey: ["library"] });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [queryClient]);

  const handleRescan = async () => {
    try {
      setIsRescanning(true);
      await rescanLibrary();
      await refetch();
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    } finally {
      setIsRescanning(false);
    }
  };

  const handleAddFolder = async () => {
    try {
      let selectedPath: string | null = null;
      try {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const res = await open({ directory: true, multiple: false });
        if (typeof res === "string") {
          selectedPath = res;
        }
      } catch {
        // Fallback for browser / mock mode
        selectedPath = window.prompt("Enter local library folder path:", "/home/user/Manga");
      }

      if (selectedPath) {
        await addLibraryDir({ path: selectedPath });
        await handleRescan();
        showToast(t("saveSuccess"), "success");
      }
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    }
  };

  const handleConfirmRemove = async () => {
    if (!galleryToRemove) return;
    try {
      setIsRemoving(true);
      await removeLocal({ id: galleryToRemove.id, deleteFiles: deleteFilesOnDisk });
      setGalleryToRemove(null);
      await refetch();
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    } finally {
      setIsRemoving(false);
    }
  };

  // Filter local galleries by title or tag
  const filteredGalleries = library.filter((g) => {
    if (!filterText) return true;
    const lower = filterText.toLowerCase();
    const titleMatch = g.title.toLowerCase().includes(lower);
    const tagMatch = g.tags.some(
      (tag) =>
        tag.name.toLowerCase().includes(lower) ||
        tag.namespace.toLowerCase().includes(lower),
    );
    return titleMatch || tagMatch;
  });

  return (
    <div className="flex flex-col h-full w-full overflow-hidden">
      {/* Header & Controls */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <BookOpen className="w-5 h-5 text-sky-500" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("localGalleries")}
          </h1>
          <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-500">
            {library.length}
          </span>
        </div>

        {/* Search filter & Actions */}
        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="w-3.5 h-3.5 text-gray-400 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              placeholder={t("filterLibrary")}
              className="pl-8 pr-3 py-1.5 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100 w-44 sm:w-56"
            />
          </div>

          <button
            type="button"
            onClick={handleRescan}
            disabled={isRescanning}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors cursor-pointer"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isRescanning ? "animate-spin" : ""}`} />
            <span>{isRescanning ? t("rescanning") : t("rescan")}</span>
          </button>

          <button
            type="button"
            onClick={handleAddFolder}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-700 text-white shadow-xs transition-colors cursor-pointer"
          >
            <FolderPlus className="w-3.5 h-3.5" />
            <span>{t("addFolder")}</span>
          </button>
        </div>
      </div>

      {/* Grid Content */}
      <div className="flex-1 overflow-y-auto p-6 pb-24">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <RefreshCw className="w-8 h-8 animate-spin text-sky-500" />
          </div>
        ) : isError ? (
          <ErrorState error={error} onRetry={() => refetch()} />
        ) : filteredGalleries.length === 0 ? (
          <EmptyState
            icon={BookOpen}
            title={t("noLocalGalleries")}
            description={t("noLocalGalleriesDesc")}
            action={{
              label: t("addFolder"),
              onClick: handleAddFolder,
            }}
          />
        ) : (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
            {filteredGalleries.map((gallery) => (
              <LibraryCardItem
                key={gallery.id}
                gallery={gallery}
                onRemove={(g) => {
                  setGalleryToRemove(g);
                  setDeleteFilesOnDisk(false);
                }}
              />
            ))}
          </div>
        )}
      </div>

      {/* Remove Library Entry Dialog */}
      {galleryToRemove && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs"
          onClick={() => setGalleryToRemove(null)}
        >
          <div
            className="w-full max-w-sm rounded-2xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 p-6 space-y-4 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between">
              <h3 className="text-base font-bold text-gray-900 dark:text-gray-100">
                {t("confirmRemoveLibrary")}
              </h3>
              <button
                type="button"
                onClick={() => setGalleryToRemove(null)}
                className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                aria-label={t("close")}
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <p className="text-xs text-gray-600 dark:text-gray-300 line-clamp-2">
              {galleryToRemove.title}
            </p>

            <label className="flex items-start gap-2.5 p-3 rounded-xl bg-gray-50 dark:bg-gray-800/60 cursor-pointer">
              <input
                type="checkbox"
                checked={deleteFilesOnDisk}
                onChange={(e) => setDeleteFilesOnDisk(e.target.checked)}
                className="mt-0.5 w-4 h-4 rounded border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-rose-600 dark:text-rose-500 accent-rose-500 focus:ring-2 focus:ring-rose-500/20 cursor-pointer"
              />
              <div className="text-xs">
                <span className="font-semibold block text-gray-900 dark:text-gray-100">
                  {t("deleteFiles")}
                </span>
                <span className="text-gray-500 text-[11px] block mt-0.5">
                  {t("deleteFilesDesc")}
                </span>
              </div>
            </label>

            <div className="flex justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={() => setGalleryToRemove(null)}
                disabled={isRemoving}
                className="px-4 py-2 text-xs font-semibold rounded-xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"
              >
                {t("cancel")}
              </button>
              <button
                type="button"
                onClick={handleConfirmRemove}
                disabled={isRemoving}
                className="px-4 py-2 text-xs font-semibold rounded-xl bg-rose-600 hover:bg-rose-700 text-white shadow-xs"
              >
                {isRemoving ? t("loading") : t("confirm")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
