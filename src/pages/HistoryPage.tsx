import React, { useState } from "react";
import { useNavigate } from "react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Clock, Trash2, BookOpen, RefreshCw, X, Calendar } from "lucide-react";
import { listHistory, clearHistory } from "@/api/ipc";
import { thumbUrl, localPageUrl } from "@/api/rhimg";
import { CategoryBadge } from "@/components/CategoryBadge";
import { EmptyState } from "@/components/EmptyState";
import { ErrorState } from "@/components/ErrorState";
import { useToastStore } from "@/components/Toasts";
import { useT } from "@/i18n";
import type { HistoryEntry } from "@/bindings/HistoryEntry";

export const HistoryPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { showToast } = useToastStore();

  const [isConfirmClearOpen, setIsConfirmClearOpen] = useState(false);
  const [isClearing, setIsClearing] = useState(false);

  const {
    data: history = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<HistoryEntry[]>({
    queryKey: ["history"],
    queryFn: listHistory,
  });

  const handleClearHistory = async () => {
    try {
      setIsClearing(true);
      await clearHistory();
      await queryClient.invalidateQueries({ queryKey: ["history"] });
      setIsConfirmClearOpen(false);
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    } finally {
      setIsClearing(false);
    }
  };

  const handleEntryClick = (entry: HistoryEntry) => {
    if (entry.target.kind === "online") {
      navigate(
        `/read/online/${entry.target.gid}/${entry.target.token}?page=${entry.lastPage}`,
      );
    } else {
      navigate(`/read/local/${entry.target.id}?page=${entry.lastPage}`);
    }
  };

  return (
    <div className="flex-1 min-h-0 flex flex-col w-full overflow-hidden">
      {/* Header */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Clock className="w-5 h-5 text-indigo-500" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("readingHistory")}
          </h1>
          <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-500">
            {history.length}
          </span>
        </div>

        {history.length > 0 && (
          <button
            type="button"
            onClick={() => setIsConfirmClearOpen(true)}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors cursor-pointer"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>{t("clearHistory")}</span>
          </button>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 overflow-y-auto p-4 sm:p-6 pb-24 space-y-3">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <RefreshCw className="w-8 h-8 animate-spin text-indigo-500" />
          </div>
        ) : isError ? (
          <ErrorState error={error} onRetry={() => refetch()} />
        ) : history.length === 0 ? (
          <EmptyState
            icon={Clock}
            title={t("noHistory")}
            description={t("noHistoryDesc")}
          />
        ) : (
          history.map((entry, index) => {
            const coverUrl =
              entry.thumb
                ? thumbUrl(entry.thumb)
                : entry.target.kind === "local"
                  ? localPageUrl(entry.target.id, 0)
                  : "";

            const formattedTime = new Date(entry.readAt * 1000).toLocaleString();

            return (
              <div
                key={`${entry.title}-${index}`}
                onClick={() => handleEntryClick(entry)}
                className="flex items-center gap-4 p-3.5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 hover:border-indigo-400 dark:hover:border-indigo-500 shadow-xs hover:shadow-md transition-all cursor-pointer group"
              >
                {/* Cover */}
                <div className="w-14 h-20 rounded-xl overflow-hidden bg-gray-100 dark:bg-gray-800 shrink-0 border border-gray-100 dark:border-gray-800">
                  {coverUrl ? (
                    <img
                      src={coverUrl}
                      alt={entry.title}
                      className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-gray-400">
                      <BookOpen className="w-5 h-5" />
                    </div>
                  )}
                </div>

                {/* Details */}
                <div className="flex-1 min-w-0 space-y-1.5">
                  <div className="flex items-center gap-2">
                    {entry.category && (
                      <CategoryBadge category={entry.category} size="sm" />
                    )}
                    <span className="text-[10px] px-1.5 py-0.5 rounded font-mono font-semibold bg-gray-100 dark:bg-gray-800 text-gray-500">
                      {entry.target.kind === "online" ? "EH" : t("folder")}
                    </span>
                  </div>

                  <h3
                    className="font-bold text-xs sm:text-sm text-gray-900 dark:text-gray-100 truncate group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors"
                    title={entry.title}
                  >
                    {entry.title}
                  </h3>

                  <div className="flex flex-wrap items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                    <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                      {t("readProgress", {
                        page: entry.lastPage,
                        total: entry.pageCount,
                      })}
                    </span>
                    <span className="flex items-center gap-1 text-[11px] text-gray-400">
                      <Calendar className="w-3 h-3" />
                      {formattedTime}
                    </span>
                  </div>
                </div>

                {/* Continue Action */}
                <div className="p-2 text-gray-400 group-hover:text-indigo-500 shrink-0">
                  <BookOpen className="w-5 h-5" />
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Clear History Confirmation Dialog */}
      {isConfirmClearOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs"
          onClick={() => setIsConfirmClearOpen(false)}
        >
          <div
            className="w-full max-w-sm rounded-2xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 p-6 space-y-4 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between">
              <h3 className="text-base font-bold text-gray-900 dark:text-gray-100">
                {t("clearHistory")}
              </h3>
              <button
                type="button"
                onClick={() => setIsConfirmClearOpen(false)}
                className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                aria-label={t("close")}
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <p className="text-xs text-gray-600 dark:text-gray-300">
              {t("confirmClearHistory")}
            </p>

            <div className="flex justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={() => setIsConfirmClearOpen(false)}
                disabled={isClearing}
                className="px-4 py-2 text-xs font-semibold rounded-xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"
              >
                {t("cancel")}
              </button>
              <button
                type="button"
                onClick={handleClearHistory}
                disabled={isClearing}
                className="px-4 py-2 text-xs font-semibold rounded-xl bg-rose-600 hover:bg-rose-700 text-white shadow-xs"
              >
                {isClearing ? t("loading") : t("confirm")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
