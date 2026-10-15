import React, { useState } from "react";
import { useNavigate } from "react-router";
import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import { Heart, RefreshCw, LogIn } from "lucide-react";
import { listGalleries, favoriteCategories } from "@/api/ipc";
import type { FavoriteCategory } from "@/bindings/FavoriteCategory";
import { useAuthStore } from "@/stores/auth";
import { GalleryGrid } from "@/components/GalleryGrid";
import { ErrorState } from "@/components/ErrorState";
import { EmptyState } from "@/components/EmptyState";
import { useT } from "@/i18n";

export const FavoritesPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const { status, isLoading: authLoading } = useAuthStore();

  const isLoggedIn = status?.loggedIn ?? false;
  const [selectedSlot, setSelectedSlot] = useState<number | null>(null);

  // Fetch slot tabs with names and counts
  const { data: categories = [], refetch: refetchCategories } = useQuery<FavoriteCategory[]>({
    queryKey: ["favoriteCategories"],
    queryFn: favoriteCategories,
    enabled: isLoggedIn,
  });

  // Ensure 10 slots
  const slots: FavoriteCategory[] = Array.from({ length: 10 }, (_, i) => {
    const found = categories.find((c) => c.slot === i);
    return (
      found || {
        slot: i,
        name: `Favorite ${i}`,
        count: 0,
      }
    );
  });

  const totalCount = categories.reduce((sum, c) => sum + c.count, 0);

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
    queryKey: ["galleries", "favorites", selectedSlot],
    queryFn: ({ pageParam = null }: { pageParam?: string | null }) =>
      listGalleries({
        source: { kind: "favorites", slot: selectedSlot },
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
    <div className="flex-1 min-h-0 flex flex-col w-full overflow-hidden">
      {/* Header */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Heart className="w-5 h-5 text-rose-500 fill-rose-500/20" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("favorites")}
          </h1>
        </div>
      </div>

      {/* Slot Tabs */}
      {isLoggedIn && (
        <div className="px-6 py-2 border-b border-gray-200 dark:border-gray-800 bg-white/40 dark:bg-gray-900/40 shrink-0 flex items-center gap-1.5 overflow-x-auto scrollbar-none select-none">
          {/* All tab */}
          <button
            type="button"
            data-qa="favorites-tab"
            onClick={() => setSelectedSlot(null)}
            className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold whitespace-nowrap transition-all cursor-pointer ${
              selectedSlot === null
                ? "bg-rose-500 text-white shadow-xs"
                : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
            }`}
          >
            <span>{t("allFavorites")}</span>
            <span
              className={`text-[10px] px-1.5 py-0.2 rounded-full ${
                selectedSlot === null
                  ? "bg-white/20 text-white"
                  : "bg-gray-200 dark:bg-gray-800 text-gray-500"
              }`}
            >
              {totalCount}
            </span>
          </button>

          {/* Slots 0-9 */}
          {slots.map((s) => {
            const isSelected = selectedSlot === s.slot;
            return (
              <button
                key={s.slot}
                type="button"
                data-qa="favorites-tab"
                onClick={() => setSelectedSlot(s.slot)}
                className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold whitespace-nowrap transition-all cursor-pointer ${
                  isSelected
                    ? "bg-rose-500 text-white shadow-xs"
                    : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
                }`}
              >
                <span>{s.name}</span>
                <span
                  className={`text-[10px] px-1.5 py-0.2 rounded-full ${
                    isSelected
                      ? "bg-white/20 text-white"
                      : "bg-gray-200 dark:bg-gray-800 text-gray-500"
                  }`}
                >
                  {s.count}
                </span>
              </button>
            );
          })}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 flex flex-col min-h-0">
        {!authLoading && !isLoggedIn ? (
          <EmptyState
            icon={LogIn}
            title={t("loginRequiredFavorites")}
            description={t("loginRequiredFavoritesDesc")}
            action={{
              label: t("goToLogin"),
              onClick: () => navigate("/login"),
            }}
          />
        ) : isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-rose-500" />
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
            onRefresh={() => {
              refetch();
              refetchCategories();
            }}
          />
        )}
      </div>
    </div>
  );
};
