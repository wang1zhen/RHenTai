import React, { useRef, useState, useEffect, useMemo, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  CheckSquare,
  Square,
  Download,
  FolderHeart,
  HeartCrack,
  X,
  Loader2,
} from "lucide-react";
import type { GallerySummary } from "@/bindings/GallerySummary";
import { GalleryCard } from "@/components/GalleryCard";
import { FavoriteDialog } from "@/components/FavoriteDialog";
import { enqueueDownloads, batchFavorites } from "@/api/ipc";
import { useToastStore } from "@/components/Toasts";
import { useT } from "@/i18n";

interface GalleryGridProps {
  items: GallerySummary[];
  hasNextPage?: boolean;
  isFetchingNextPage?: boolean;
  fetchNextPage?: () => void;
  isLoading?: boolean;
  onRefresh?: () => void;
}

export const GalleryGrid: React.FC<GalleryGridProps> = ({
  items,
  hasNextPage = false,
  isFetchingNextPage = false,
  fetchNextPage,
  isLoading = false,
  onRefresh,
}) => {
  const { t } = useT();
  const { showToast } = useToastStore();
  const parentRef = useRef<HTMLDivElement>(null);

  // Responsive column count
  const [columns, setColumns] = useState<number>(4);

  useEffect(() => {
    const updateColumns = () => {
      if (!parentRef.current) return;
      const width = parentRef.current.clientWidth;
      if (width < 480) {
        setColumns(2);
      } else if (width < 768) {
        setColumns(3);
      } else if (width < 1100) {
        setColumns(4);
      } else if (width < 1440) {
        setColumns(5);
      } else {
        setColumns(6);
      }
    };

    updateColumns();
    const observer = new ResizeObserver(updateColumns);
    if (parentRef.current) observer.observe(parentRef.current);
    return () => observer.disconnect();
  }, []);

  // Multi-select state
  const [isSelectMode, setIsSelectMode] = useState<boolean>(false);
  const [selectedGids, setSelectedGids] = useState<Set<number>>(new Set());
  const [isFavDialogOpen, setIsFavDialogOpen] = useState<boolean>(false);

  const toggleSelect = useCallback((gid: number) => {
    setSelectedGids((prev) => {
      const next = new Set(prev);
      if (next.has(gid)) {
        next.delete(gid);
      } else {
        next.add(gid);
      }
      return next;
    });
  }, []);

  const selectAll = () => {
    setSelectedGids(new Set(items.map((g) => g.gid)));
  };

  const clearSelection = () => {
    setSelectedGids(new Set());
  };

  const exitSelectMode = () => {
    setIsSelectMode(false);
    setSelectedGids(new Set());
  };

  // Group items into rows for virtualization
  const rows = useMemo(() => {
    const r: GallerySummary[][] = [];
    for (let i = 0; i < items.length; i += columns) {
      r.push(items.slice(i, i + columns));
    }
    return r;
  }, [items, columns]);

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 350,
    overscan: 2,
  });

  // Infinite scroll trigger
  const virtualRows = rowVirtualizer.getVirtualItems();
  useEffect(() => {
    if (!virtualRows || virtualRows.length === 0) return;
    const lastItem = virtualRows[virtualRows.length - 1];
    if (
      lastItem.index >= rows.length - 1 &&
      hasNextPage &&
      !isFetchingNextPage &&
      fetchNextPage
    ) {
      fetchNextPage();
    }
  }, [virtualRows, rows.length, hasNextPage, isFetchingNextPage, fetchNextPage]);

  // Batch actions
  const handleBatchDownload = async () => {
    if (selectedGids.size === 0) return;
    const selectedGalleries = items
      .filter((g) => selectedGids.has(g.gid))
      .map((g) => ({ gid: g.gid, token: g.token }));

    try {
      await enqueueDownloads({ galleries: selectedGalleries, original: null });
      showToast(
        t("selectedCount", { count: selectedGalleries.length }) + " " + t("downloads"),
        "success",
      );
      exitSelectMode();
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    }
  };

  const handleBatchMoveFavorites = async (slot: number | null) => {
    if (selectedGids.size === 0) return;
    const gids = Array.from(selectedGids);
    try {
      await batchFavorites({ gids, slot });
      showToast(t("saveSuccess"), "success");
      exitSelectMode();
      onRefresh?.();
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    }
  };

  const handleBatchRemoveFavorites = async () => {
    if (selectedGids.size === 0) return;
    if (!confirm(t("confirmBatchDelete"))) return;
    await handleBatchMoveFavorites(null);
  };

  return (
    <div className="relative flex-1 min-h-0 flex flex-col h-full w-full overflow-hidden">
      {/* Top Bar / Select Mode Toggle */}
      <div className="flex items-center justify-between px-6 py-2.5 border-b border-gray-200 dark:border-gray-800 bg-white/50 dark:bg-gray-900/50 backdrop-blur-xs shrink-0 z-10">
        <div className="text-xs text-gray-500 font-medium">
          {t("galleriesLoaded", { n: items.length })}
        </div>

        <button
          type="button"
          onClick={() => {
            if (isSelectMode) {
              exitSelectMode();
            } else {
              setIsSelectMode(true);
            }
          }}
          className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
            isSelectMode
              ? "bg-sky-100 dark:bg-sky-950/60 text-sky-600 dark:text-sky-400"
              : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-gray-100"
          }`}
        >
          {isSelectMode ? (
            <>
              <CheckSquare className="w-4 h-4" />
              <span>{t("cancelSelect")}</span>
            </>
          ) : (
            <>
              <Square className="w-4 h-4" />
              <span>{t("selectMode")}</span>
            </>
          )}
        </button>
      </div>

      {/* Virtualized Container */}
      <div
        ref={parentRef}
        className="flex-1 min-h-0 overflow-y-auto px-6 py-4 pb-24"
      >
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            width: "100%",
            position: "relative",
          }}
        >
          {virtualRows.map((virtualRow) => {
            const rowGalleries = rows[virtualRow.index];
            return (
              <div
                key={virtualRow.key}
                data-index={virtualRow.index}
                ref={rowVirtualizer.measureElement}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                className="pb-4"
              >
                <div
                  className="grid gap-4"
                  style={{
                    gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
                  }}
                >
                  {rowGalleries.map((gallery) => (
                    <GalleryCard
                      key={gallery.gid}
                      gallery={gallery}
                      isSelectMode={isSelectMode}
                      isSelected={selectedGids.has(gallery.gid)}
                      onToggleSelect={toggleSelect}
                    />
                  ))}
                </div>
              </div>
            );
          })}
        </div>

        {/* Loading Indicator for next page */}
        {isFetchingNextPage && (
          <div className="flex items-center justify-center gap-2 py-6 text-xs text-gray-500 font-medium">
            <Loader2 className="w-4 h-4 animate-spin text-sky-500" />
            <span>{t("loadingMore")}</span>
          </div>
        )}

        {!hasNextPage && items.length > 0 && !isLoading && (
          <div className="text-center py-6 text-xs text-gray-400">
            {t("noMoreItems")}
          </div>
        )}
      </div>

      {/* Floating Multi-select Action Bar */}
      {isSelectMode && (
        <div className="fixed bottom-20 md:bottom-8 left-1/2 -translate-x-1/2 z-40 flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-gray-900/95 dark:bg-gray-800/95 text-white shadow-2xl backdrop-blur-md border border-gray-700/50 animate-in slide-in-from-bottom-4 duration-200">
          <div className="text-xs font-semibold mr-2 border-r border-gray-700 pr-3">
            {t("selectedCount", { count: selectedGids.size })}
          </div>

          <button
            type="button"
            onClick={selectedGids.size === items.length ? clearSelection : selectAll}
            className="px-2.5 py-1 text-xs font-medium rounded-lg hover:bg-white/10 transition-colors"
          >
            {selectedGids.size === items.length ? t("clearAll") : t("selectAll")}
          </button>

          <button
            type="button"
            onClick={handleBatchDownload}
            disabled={selectedGids.size === 0}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-xl bg-sky-600 hover:bg-sky-500 disabled:opacity-40 transition-colors"
          >
            <Download className="w-3.5 h-3.5" />
            <span>{t("batchDownload")}</span>
          </button>

          <button
            type="button"
            onClick={() => setIsFavDialogOpen(true)}
            disabled={selectedGids.size === 0}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-xl bg-rose-600 hover:bg-rose-500 disabled:opacity-40 transition-colors"
          >
            <FolderHeart className="w-3.5 h-3.5" />
            <span>{t("batchMoveFavorite")}</span>
          </button>

          <button
            type="button"
            onClick={handleBatchRemoveFavorites}
            disabled={selectedGids.size === 0}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-xl bg-gray-700 hover:bg-gray-600 disabled:opacity-40 transition-colors"
          >
            <HeartCrack className="w-3.5 h-3.5" />
            <span>{t("batchRemoveFavorite")}</span>
          </button>

          <button
            type="button"
            onClick={exitSelectMode}
            className="p-1 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 ml-1 transition-colors"
            aria-label={t("close")}
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Favorite Slot Picker Dialog */}
      <FavoriteDialog
        isOpen={isFavDialogOpen}
        onClose={() => setIsFavDialogOpen(false)}
        title={t("batchMoveFavorite")}
        onConfirm={async (slot) => {
          await handleBatchMoveFavorites(slot);
        }}
      />
    </div>
  );
};
