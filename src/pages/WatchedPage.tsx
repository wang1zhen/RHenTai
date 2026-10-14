import React from "react";
import { useNavigate } from "react-router";
import { useInfiniteQuery } from "@tanstack/react-query";
import { Eye, RefreshCw, LogIn } from "lucide-react";
import { listGalleries } from "@/api/ipc";
import { useAuthStore } from "@/stores/auth";
import { GalleryGrid } from "@/components/GalleryGrid";
import { ErrorState } from "@/components/ErrorState";
import { EmptyState } from "@/components/EmptyState";
import { useT } from "@/i18n";

export const WatchedPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const { status, isLoading: authLoading } = useAuthStore();

  const isLoggedIn = status?.loggedIn ?? false;

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
    queryKey: ["galleries", "watched"],
    queryFn: ({ pageParam = null }: { pageParam?: string | null }) =>
      listGalleries({
        source: { kind: "watched" },
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
    enabled: isLoggedIn,
  });

  const allItems = data?.pages.flatMap((page) => page.items) || [];

  return (
    <div className="flex flex-col h-full w-full overflow-hidden">
      {/* Header */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Eye className="w-5 h-5 text-indigo-500 fill-indigo-500/20" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("watched")}
          </h1>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex flex-col min-h-0">
        {!authLoading && !isLoggedIn ? (
          <EmptyState
            icon={LogIn}
            title={t("loginRequiredWatched")}
            description={t("loginRequiredWatchedDesc")}
            action={{
              label: t("goToLogin"),
              onClick: () => navigate("/login"),
            }}
          />
        ) : isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-indigo-500" />
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
