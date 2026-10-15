import React, { useState } from "react";
import { useInfiniteQuery } from "@tanstack/react-query";
import { Trophy, RefreshCw } from "lucide-react";
import { listGalleries } from "@/api/ipc";
import type { ToplistPeriod } from "@/bindings/ToplistPeriod";
import { GalleryGrid } from "@/components/GalleryGrid";
import { ErrorState } from "@/components/ErrorState";
import { EmptyState } from "@/components/EmptyState";
import { useT, type TranslationKey } from "@/i18n";

const PERIODS: ToplistPeriod[] = ["all", "year", "month", "day"];

export const ToplistPage: React.FC = () => {
  const { t } = useT();
  const [period, setPeriod] = useState<ToplistPeriod>("all");

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
    queryKey: ["galleries", "toplist", period],
    queryFn: ({ pageParam = null }: { pageParam?: string | null }) =>
      listGalleries({
        source: { kind: "toplist", period },
        filter: {
          keyword: "",
          excludedCategories: [],
          minRating: null,
          minPages: null,
          maxPages: null,
          showExpunged: false,
          requireTorrent: false,
        },
        cursor: pageParam,
      }),
    getNextPageParam: (lastPage) => lastPage.next ?? undefined,
    initialPageParam: null as string | null,
  });

  const allItems = data?.pages.flatMap((page) => page.items) || [];

  return (
    <div className="flex-1 min-h-0 flex flex-col w-full overflow-hidden">
      {/* Header & Tabs */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <Trophy className="w-5 h-5 text-amber-500 fill-amber-500/20" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("toplist")}
          </h1>
        </div>

        {/* Period Tabs */}
        <div className="flex items-center gap-1 p-1 bg-gray-100 dark:bg-gray-800 rounded-xl">
          {PERIODS.map((p) => {
            const isActive = period === p;
            return (
              <button
                key={p}
                type="button"
                onClick={() => setPeriod(p)}
                className={`px-3 py-1 text-xs font-semibold rounded-lg transition-all cursor-pointer ${
                  isActive
                    ? "bg-white dark:bg-gray-900 text-amber-600 dark:text-amber-400 shadow-xs"
                    : "text-gray-500 hover:text-gray-900 dark:hover:text-gray-100"
                }`}
              >
                {t(p as TranslationKey)}
              </button>
            );
          })}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex flex-col min-h-0">
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-amber-500" />
          </div>
        ) : isError ? (
          <ErrorState error={error} onRetry={() => refetch()} />
        ) : allItems.length === 0 ? (
          <EmptyState
            title={t("noResults")}
            description={t("noResultsDesc")}
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
