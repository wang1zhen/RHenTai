import React from "react";
import { useQuery } from "@tanstack/react-query";
import { RefreshCw, Flame } from "lucide-react";
import { listGalleries } from "@/api/ipc";
import { GalleryGrid } from "@/components/GalleryGrid";
import { ErrorState } from "@/components/ErrorState";
import { EmptyState } from "@/components/EmptyState";
import { useT } from "@/i18n";

export const PopularPage: React.FC = () => {
  const { t } = useT();

  const { data, isLoading, isError, error, refetch } = useQuery({
    queryKey: ["galleries", "popular"],
    queryFn: () =>
      listGalleries({
        source: { kind: "popular" },
        filter: {
          keyword: "",
          excludedCategories: [],
          minRating: null,
          minPages: null,
          maxPages: null,
          showExpunged: false,
          requireTorrent: false,
        },
        cursor: null,
      }),
  });

  const items = data?.items || [];

  return (
    <div className="flex flex-col h-full w-full overflow-hidden">
      {/* Header */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Flame className="w-5 h-5 text-orange-500 fill-orange-500/20" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("popular")}
          </h1>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex flex-col min-h-0">
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-orange-500" />
          </div>
        ) : isError ? (
          <ErrorState error={error} onRetry={() => refetch()} />
        ) : items.length === 0 ? (
          <EmptyState
            title={t("noResults")}
            description={t("noResultsDesc")}
          />
        ) : (
          <GalleryGrid
            items={items}
            hasNextPage={false}
            isFetchingNextPage={false}
            onRefresh={() => refetch()}
          />
        )}
      </div>
    </div>
  );
};
