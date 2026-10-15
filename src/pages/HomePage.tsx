import React, { useState, useEffect } from "react";
import { useSearchParams } from "react-router";
import { useInfiniteQuery } from "@tanstack/react-query";
import { SlidersHorizontal, Check, RefreshCw, X } from "lucide-react";
import { listGalleries } from "@/api/ipc";
import type { Category } from "@/bindings/Category";
import type { SearchFilter } from "@/bindings/SearchFilter";
import { GalleryGrid } from "@/components/GalleryGrid";
import { ErrorState } from "@/components/ErrorState";
import { EmptyState } from "@/components/EmptyState";
import { CATEGORY_COLORS } from "@/components/CategoryBadge";
import { useT, type TranslationKey } from "@/i18n";

const ALL_CATEGORIES: Category[] = [
  "doujinshi",
  "manga",
  "artistCg",
  "gameCg",
  "western",
  "nonH",
  "imageSet",
  "cosplay",
  "asianPorn",
  "misc",
];

export const HomePage: React.FC = () => {
  const { t } = useT();
  const [searchParams, setSearchParams] = useSearchParams();
  const keywordParam = searchParams.get("q") || "";

  // Filter state
  const [keyword, setKeyword] = useState<string>(keywordParam);
  const [excludedCategories, setExcludedCategories] = useState<Category[]>([]);
  const [minRating, setMinRating] = useState<number | null>(null);
  const [minPages, setMinPages] = useState<number | null>(null);
  const [maxPages, setMaxPages] = useState<number | null>(null);
  const [showExpunged, setShowExpunged] = useState<boolean>(false);
  const [requireTorrent, setRequireTorrent] = useState<boolean>(false);

  const [isFilterOpen, setIsFilterOpen] = useState<boolean>(false);

  useEffect(() => {
    setKeyword(keywordParam);
  }, [keywordParam]);

  // Construct search filter
  const filter: SearchFilter = {
    keyword,
    excludedCategories,
    minRating,
    minPages,
    maxPages,
    showExpunged,
    requireTorrent,
  };

  const {
    data,
    isLoading,
    isError,
    error,
    refetch,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["galleries", "front", filter],
    queryFn: ({ pageParam = null }: { pageParam?: string | null }) =>
      listGalleries({
        source: { kind: "front" },
        filter,
        cursor: pageParam,
      }),
    getNextPageParam: (lastPage) => lastPage.next ?? undefined,
    initialPageParam: null as string | null,
  });

  const allItems = data?.pages.flatMap((page) => page.items) || [];

  const toggleCategory = (cat: Category) => {
    setExcludedCategories((prev) => {
      if (prev.includes(cat)) {
        return prev.filter((c) => c !== cat);
      } else {
        return [...prev, cat];
      }
    });
  };

  const resetFilters = () => {
    setExcludedCategories([]);
    setMinRating(null);
    setMinPages(null);
    setMaxPages(null);
    setShowExpunged(false);
    setRequireTorrent(false);
  };

  return (
    <div className="flex-1 min-h-0 flex flex-col w-full overflow-hidden">
      {/* Category Filter Chips Bar */}
      <div className="px-3 sm:px-6 py-2.5 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between gap-2 overflow-hidden select-none">
        <div className="flex items-center gap-1.5 flex-nowrap overflow-x-auto py-1 scrollbar-none min-w-0 flex-1">
          {ALL_CATEGORIES.map((cat) => {
            const isExcluded = excludedCategories.includes(cat);
            const isIncluded = !isExcluded;
            const color = CATEGORY_COLORS[cat];

            return (
              <button
                key={cat}
                type="button"
                onClick={() => toggleCategory(cat)}
                className={`shrink-0 whitespace-nowrap inline-flex items-center gap-1 px-2.5 py-1 text-xs font-semibold rounded-lg transition-all cursor-pointer border ${
                  isIncluded
                    ? "text-white shadow-xs"
                    : "border-gray-200 dark:border-gray-800 text-gray-400 bg-gray-100/50 dark:bg-gray-800/40 opacity-50"
                }`}
                style={{
                  backgroundColor: isIncluded ? color : undefined,
                  borderColor: isIncluded ? color : undefined,
                }}
              >
                {isIncluded && <Check className="w-3 h-3" />}
                <span>{t(cat as TranslationKey)}</span>
              </button>
            );
          })}
        </div>

        {/* Advanced Filter Popover Button */}
        <button
          type="button"
          onClick={() => setIsFilterOpen(!isFilterOpen)}
          aria-label={t("advancedFilter")}
          className={`shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-xl border transition-colors cursor-pointer ${
            isFilterOpen || minRating || minPages || maxPages || showExpunged || requireTorrent
              ? "bg-sky-50 dark:bg-sky-950/50 border-sky-500 text-sky-600 dark:text-sky-400"
              : "border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
          }`}
          title={t("advancedFilter")}
        >
          <SlidersHorizontal className="w-3.5 h-3.5" />
          <span className="hidden sm:inline">{t("advancedFilter")}</span>
        </button>
      </div>

      {/* Advanced Filter Modal/Popover */}
      {isFilterOpen && (
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-lg animate-in slide-in-from-top-2 duration-150 shrink-0 z-20">
          <div className="max-w-4xl mx-auto flex flex-col gap-4">
            <div className="flex items-center justify-between pb-2 border-b border-gray-100 dark:border-gray-800">
              <h4 className="text-sm font-bold">{t("advancedFilter")}</h4>
              <button
                type="button"
                onClick={() => setIsFilterOpen(false)}
                className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                aria-label={t("close")}
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 text-xs">
              {/* Min Rating */}
              <div>
                <label className="block font-medium text-gray-500 dark:text-gray-400 mb-1.5">
                  {t("minRating")}
                </label>
                <select
                  value={minRating ?? ""}
                  onChange={(e) =>
                    setMinRating(e.target.value ? Number(e.target.value) : null)
                  }
                  className="w-full px-3 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500 transition-colors"
                >
                  <option value="">{t("anyRating")}</option>
                  <option value="2">2 {t("starsOrMore")}</option>
                  <option value="3">3 {t("starsOrMore")}</option>
                  <option value="4">4 {t("starsOrMore")}</option>
                  <option value="5">5 {t("starsOrMore")}</option>
                </select>
              </div>

              {/* Min Pages */}
              <div>
                <label className="block font-medium text-gray-500 dark:text-gray-400 mb-1.5">
                  {t("minPages")}
                </label>
                <input
                  type="number"
                  min="0"
                  value={minPages ?? ""}
                  onChange={(e) =>
                    setMinPages(e.target.value ? Number(e.target.value) : null)
                  }
                  placeholder="0"
                  className="w-full px-3 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500 transition-colors"
                />
              </div>

              {/* Max Pages */}
              <div>
                <label className="block font-medium text-gray-500 dark:text-gray-400 mb-1.5">
                  {t("maxPages")}
                </label>
                <input
                  type="number"
                  min="0"
                  value={maxPages ?? ""}
                  onChange={(e) =>
                    setMaxPages(e.target.value ? Number(e.target.value) : null)
                  }
                  placeholder="∞"
                  className="w-full px-3 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500 transition-colors"
                />
              </div>

              {/* Toggles */}
              <div className="flex flex-col justify-end gap-2">
                <label className="flex items-center gap-2 cursor-pointer font-medium text-gray-700 dark:text-gray-300">
                  <input
                    type="checkbox"
                    checked={showExpunged}
                    onChange={(e) => setShowExpunged(e.target.checked)}
                    className="w-4 h-4 rounded border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-sky-600 dark:text-sky-500 accent-sky-500 focus:ring-2 focus:ring-sky-500/20 cursor-pointer"
                  />
                  <span>{t("showExpunged")}</span>
                </label>

                <label className="flex items-center gap-2 cursor-pointer font-medium text-gray-700 dark:text-gray-300">
                  <input
                    type="checkbox"
                    checked={requireTorrent}
                    onChange={(e) => setRequireTorrent(e.target.checked)}
                    className="w-4 h-4 rounded border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-sky-600 dark:text-sky-500 accent-sky-500 focus:ring-2 focus:ring-sky-500/20 cursor-pointer"
                  />
                  <span>{t("requireTorrent")}</span>
                </label>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-gray-100 dark:border-gray-800">
              <button
                type="button"
                onClick={resetFilters}
                className="px-3 py-1.5 text-xs font-semibold text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
              >
                {t("reset")}
              </button>
              <button
                type="button"
                onClick={() => setIsFilterOpen(false)}
                className="px-4 py-1.5 text-xs font-semibold rounded-lg bg-sky-600 hover:bg-sky-700 text-white shadow-xs"
              >
                {t("apply")}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Content Area */}
      <div className="flex-1 flex flex-col min-h-0">
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-sky-500" />
          </div>
        ) : isError ? (
          <ErrorState error={error} onRetry={() => refetch()} />
        ) : allItems.length === 0 ? (
          <EmptyState
            title={t("noResults")}
            description={t("noResultsDesc")}
            action={{
              label: t("reset"),
              onClick: () => {
                resetFilters();
                setSearchParams({});
              },
            }}
          />
        ) : (
          <GalleryGrid
            items={allItems}
            hasNextPage={hasNextPage}
            isFetchingNextPage={isFetchingNextPage}
            fetchNextPage={fetchNextPage}
            onRefresh={() => refetch()}
          />
        )}
      </div>
    </div>
  );
};
