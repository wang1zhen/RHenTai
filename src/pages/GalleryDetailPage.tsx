import React, { useState } from "react";
import { useParams, useNavigate } from "react-router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  BookOpen,
  Download,
  Heart,
  ExternalLink,
  Calendar,
  User,
  FileText,
  Star,
  Globe,
  HardDrive,
  MessageSquare,
  ChevronLeft,
  ChevronRight,
  RefreshCw,
  FolderOpen,
} from "lucide-react";
import {
  galleryDetail,
  galleryThumbnails,
  setFavorite,
  enqueueDownloads,
  listHistory,
} from "@/api/ipc";
import type { ThumbnailPage } from "@/bindings/ThumbnailPage";
import { CategoryBadge } from "@/components/CategoryBadge";
import { RatingStars } from "@/components/RatingStars";
import { TagChip } from "@/components/TagChip";
import { SpriteThumb } from "@/components/SpriteThumb";
import { FavoriteDialog } from "@/components/FavoriteDialog";
import { ErrorState } from "@/components/ErrorState";
import { thumbUrl } from "@/api/rhimg";
import { useToastStore } from "@/components/Toasts";
import { useT } from "@/i18n";

export const GalleryDetailPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { showToast } = useToastStore();

  const { gid: gidStr, token } = useParams<{ gid: string; token: string }>();
  const gid = Number(gidStr);

  const [currentThumbPage, setCurrentThumbPage] = useState<number>(0);
  const [isFavDialogOpen, setIsFavDialogOpen] = useState<boolean>(false);

  // Fetch gallery detail
  const {
    data: detail,
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery({
    queryKey: ["galleryDetail", gid, token],
    queryFn: () => galleryDetail({ gid, token: token || "" }),
    enabled: !isNaN(gid) && Boolean(token),
  });

  // Fetch thumbnail pages if p > 0
  const { data: pagedThumbs, isLoading: isThumbsLoading } = useQuery<ThumbnailPage>({
    queryKey: ["galleryThumbnails", gid, token, currentThumbPage],
    queryFn: () => galleryThumbnails({ gid, token: token || "", page: currentThumbPage }),
    enabled: currentThumbPage > 0 && !isNaN(gid) && Boolean(token),
  });

  // Check history for continue reading
  const { data: history = [] } = useQuery({
    queryKey: ["history"],
    queryFn: listHistory,
  });

  const historyEntry = history.find(
    (h) => h.target.kind === "online" && h.target.gid === gid,
  );

  const thumbnailsToDisplay =
    currentThumbPage === 0
      ? detail?.thumbnails?.pages || []
      : pagedThumbs?.pages || [];

  const handleDownload = async () => {
    if (!detail) return;
    try {
      await enqueueDownloads({
        galleries: [{ gid, token: token || "" }],
        original: null,
      });
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    }
  };

  const handleFavoriteConfirm = async (slot: number | null, note: string) => {
    try {
      await setFavorite({ gid, token: token || "", slot, note });
      queryClient.invalidateQueries({ queryKey: ["galleryDetail", gid, token] });
      queryClient.invalidateQueries({ queryKey: ["favoriteCategories"] });
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    }
  };

  const handleOpenBrowser = () => {
    const url = `https://e-hentai.org/g/${gid}/${token}/`;
    window.open(url, "_blank");
  };

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center h-full">
        <RefreshCw className="w-8 h-8 animate-spin text-sky-500" />
      </div>
    );
  }

  if (isError || !detail) {
    return <ErrorState error={error} onRetry={() => refetch()} />;
  }

  const { summary } = detail;
  const isFavorited = summary.favoriteSlot !== null;

  return (
    <div className="max-w-6xl mx-auto px-4 sm:px-6 py-6 pb-24 space-y-8 animate-in fade-in duration-200">
      {/* Top Section: Cover & Metadata */}
      <div className="flex flex-col md:flex-row gap-6 lg:gap-8 items-start">
        {/* Cover Image */}
        <div className="w-full sm:w-64 md:w-72 lg:w-80 shrink-0 mx-auto md:mx-0">
          <div className="relative aspect-7/10 rounded-2xl overflow-hidden shadow-xl border border-gray-200 dark:border-gray-800 bg-gray-100 dark:bg-gray-800 group">
            <img
              src={thumbUrl(summary.thumb)}
              alt={summary.title}
              loading="lazy"
              decoding="async"
              className="w-full h-full object-cover"
            />
            <div className="absolute top-3 left-3">
              <CategoryBadge category={summary.category} size="md" />
            </div>
            {isFavorited && (
              <div className="absolute top-3 right-3 p-2 rounded-full bg-rose-500/90 text-white shadow-lg backdrop-blur-xs">
                <Heart className="w-4 h-4 fill-current" />
              </div>
            )}
          </div>
        </div>

        {/* Info & Actions */}
        <div className="flex-1 flex flex-col min-w-0 w-full">
          {/* Titles */}
          <div className="space-y-1 mb-4">
            <h1 className="text-lg sm:text-xl md:text-2xl font-bold text-gray-900 dark:text-gray-100 leading-snug">
              {summary.title}
            </h1>
            {summary.titleJpn && (
              <h2 className="text-sm sm:text-base font-normal text-gray-500 dark:text-gray-400">
                {summary.titleJpn}
              </h2>
            )}
          </div>

          {/* Action Buttons Row */}
          <div className="flex flex-wrap items-center gap-2.5 mb-6">
            {/* Read Button */}
            <button
              type="button"
              data-qa="read-button"
              onClick={() => navigate(`/read/online/${gid}/${token}?page=1`)}
              className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-sky-600 hover:bg-sky-500 active:scale-95 text-white font-bold text-xs shadow-md shadow-sky-600/20 transition-all cursor-pointer"
            >
              <BookOpen className="w-4 h-4" />
              <span>{t("read")}</span>
            </button>

            {/* Continue Reading Button if in history */}
            {historyEntry && historyEntry.lastPage > 1 && (
              <button
                type="button"
                data-qa="continue-button"
                onClick={() =>
                  navigate(`/read/online/${gid}/${token}?page=${historyEntry.lastPage}`)
                }
                className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-indigo-50 dark:bg-indigo-950/50 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 text-indigo-600 dark:text-indigo-400 font-semibold text-xs border border-indigo-200 dark:border-indigo-800 transition-colors cursor-pointer"
              >
                <span>
                  {t("continueReading")} ({historyEntry.lastPage} / {summary.fileCount})
                </span>
              </button>
            )}

            {/* Download Button */}
            <button
              type="button"
              data-qa="download-button"
              onClick={handleDownload}
              className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 font-semibold text-xs shadow-xs transition-colors cursor-pointer"
            >
              <Download className="w-4 h-4" />
              <span>{t("download")}</span>
            </button>

            {/* Favorite Button */}
            <button
              type="button"
              data-qa="favorite-button"
              onClick={() => setIsFavDialogOpen(true)}
              className={`inline-flex items-center gap-2 px-4 py-2.5 rounded-xl font-semibold text-xs border transition-all cursor-pointer ${
                isFavorited
                  ? "bg-rose-50 dark:bg-rose-950/50 border-rose-500 text-rose-600 dark:text-rose-400"
                  : "border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200"
              }`}
            >
              <Heart
                className={`w-4 h-4 ${isFavorited ? "fill-current" : ""}`}
              />
              <span>
                {isFavorited ? `${t("favorited")} (${summary.favoriteSlot})` : t("favorite")}
              </span>
            </button>

            {/* Open in Browser */}
            <button
              type="button"
              onClick={handleOpenBrowser}
              className="inline-flex items-center gap-2 p-2.5 rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-500 hover:text-gray-900 dark:hover:text-gray-100 transition-colors cursor-pointer"
              title={t("openInBrowser")}
              aria-label={t("openInBrowser")}
            >
              <ExternalLink className="w-4 h-4" />
            </button>
          </div>

          {/* Metadata Table */}
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 p-4 rounded-2xl bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-800 text-xs">
            <div className="flex items-center gap-2">
              <User className="w-4 h-4 text-gray-400 shrink-0" />
              <div className="truncate">
                <span className="text-gray-400 mr-1">{t("uploader")}:</span>
                <span className="font-semibold">{summary.uploader || "Anonymous"}</span>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Calendar className="w-4 h-4 text-gray-400 shrink-0" />
              <div>
                <span className="text-gray-400 mr-1">{t("posted")}:</span>
                <span className="font-semibold">
                  {new Date(summary.posted * 1000).toLocaleDateString()}
                </span>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <FileText className="w-4 h-4 text-gray-400 shrink-0" />
              <div>
                <span className="text-gray-400 mr-1">{t("pages")}:</span>
                <span className="font-semibold">{summary.fileCount} {t("pages")}</span>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Star className="w-4 h-4 text-amber-400 shrink-0 fill-current" />
              <div className="flex items-center gap-1.5">
                <RatingStars rating={summary.rating} size="sm" showScore />
                {detail.ratingCount && (
                  <span className="text-gray-400 text-[10px]">
                    ({detail.ratingCount})
                  </span>
                )}
              </div>
            </div>

            {detail.language && (
              <div className="flex items-center gap-2">
                <Globe className="w-4 h-4 text-gray-400 shrink-0" />
                <div>
                  <span className="text-gray-400 mr-1">{t("language")}:</span>
                  <span className="font-semibold">{detail.language}</span>
                </div>
              </div>
            )}

            {detail.fileSize && (
              <div className="flex items-center gap-2">
                <HardDrive className="w-4 h-4 text-gray-400 shrink-0" />
                <div>
                  <span className="text-gray-400 mr-1">{t("fileSize")}:</span>
                  <span className="font-semibold">{detail.fileSize}</span>
                </div>
              </div>
            )}

            {detail.favoritedCount !== null && detail.favoritedCount !== undefined && (
              <div className="flex items-center gap-2">
                <Heart className="w-4 h-4 text-rose-500 shrink-0" />
                <div>
                  <span className="text-gray-400 mr-1">{t("favorite")}:</span>
                  <span className="font-semibold">
                    {detail.favoritedCount} {t("favoritedTimes")}
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* Tags Section */}
          <div className="mt-6 space-y-3">
            <h3 className="text-xs font-bold uppercase tracking-wider text-gray-400">
              {t("tags")}
            </h3>
            <div className="space-y-2">
              {detail.tagGroups.length > 0 ? (
                detail.tagGroups.map((group) => (
                  <div
                    key={group.namespace}
                    className="flex flex-wrap items-center gap-1.5"
                  >
                    <span className="text-xs font-bold text-gray-500 dark:text-gray-400 min-w-16">
                      {group.namespace}:
                    </span>
                    <div className="flex flex-wrap gap-1.5">
                      {group.tags.map((tag) => (
                        <TagChip
                          key={tag}
                          tag={{ namespace: group.namespace, name: tag }}
                        />
                      ))}
                    </div>
                  </div>
                ))
              ) : summary.tags.length > 0 ? (
                <div className="flex flex-wrap gap-1.5">
                  {summary.tags.map((tag) => (
                    <TagChip key={`${tag.namespace}:${tag.name}`} tag={tag} />
                  ))}
                </div>
              ) : (
                <p className="text-xs text-gray-400 italic">{t("noTags")}</p>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Thumbnails Section */}
      <div className="space-y-4 pt-6 border-t border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-gray-900 dark:text-gray-100 flex items-center gap-2">
            <FileText className="w-4 h-4 text-sky-500" />
            <span>
              {t("thumbnailPage")} ({detail.thumbnails?.pages?.length || 0} /{" "}
              {summary.fileCount})
            </span>
          </h3>

          {/* Thumbnail pagination controls if multiple pages */}
          {detail.thumbnailPageCount > 1 && (
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setCurrentThumbPage((p) => Math.max(0, p - 1))}
                disabled={currentThumbPage === 0 || isThumbsLoading}
                className="p-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 disabled:opacity-40"
                aria-label={t("prevPage")}
              >
                <ChevronLeft className="w-4 h-4" />
              </button>
              <span className="text-xs font-semibold px-2">
                {currentThumbPage + 1} / {detail.thumbnailPageCount}
              </span>
              <button
                type="button"
                onClick={() =>
                  setCurrentThumbPage((p) =>
                    Math.min(detail.thumbnailPageCount - 1, p + 1),
                  )
                }
                disabled={
                  currentThumbPage >= detail.thumbnailPageCount - 1 ||
                  isThumbsLoading
                }
                className="p-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 disabled:opacity-40"
                aria-label={t("nextPage")}
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          )}
        </div>

        {/* Thumbnail Grid */}
        <div className="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 gap-3">
          {thumbnailsToDisplay.map((p) => (
            <div
              key={p.index}
              onClick={() =>
                navigate(`/read/online/${gid}/${token}?page=${p.index + 1}`)
              }
              className="group relative rounded-xl overflow-hidden border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 hover:border-sky-500 dark:hover:border-sky-500 shadow-xs hover:shadow-md transition-all cursor-pointer"
            >
              <SpriteThumb sprite={p.thumb} alt={p.fileName || `Page ${p.index + 1}`} />
              <div className="p-1.5 text-center text-[11px] font-medium text-gray-500 dark:text-gray-400 group-hover:text-sky-600 dark:group-hover:text-sky-400">
                {p.index + 1}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Comments Section */}
      <div className="space-y-4 pt-6 border-t border-gray-200 dark:border-gray-800">
        <h3 className="text-sm font-bold text-gray-900 dark:text-gray-100 flex items-center gap-2">
          <MessageSquare className="w-4 h-4 text-sky-500" />
          <span>
            {t("comments")} ({detail.comments.length})
          </span>
        </h3>

        {detail.comments.length > 0 ? (
          <div className="space-y-3">
            {detail.comments.map((comment, index) => (
              <div
                key={index}
                className="p-4 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-2 shadow-xs"
              >
                <div className="flex items-center justify-between text-xs">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-gray-900 dark:text-gray-100">
                      {comment.author}
                    </span>
                    {comment.isUploader && (
                      <span className="px-1.5 py-0.5 text-[10px] font-bold rounded bg-sky-100 dark:bg-sky-950/60 text-sky-600 dark:text-sky-400">
                        {t("uploader")}
                      </span>
                    )}
                  </div>
                  <span className="text-gray-400 text-[11px]">
                    {comment.posted}
                  </span>
                </div>
                <div className="text-xs text-gray-700 dark:text-gray-300 leading-relaxed whitespace-pre-line">
                  {comment.body}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-xs text-gray-400 italic">{t("noComments")}</p>
        )}
      </div>

      {/* Favorite Dialog */}
      <FavoriteDialog
        isOpen={isFavDialogOpen}
        onClose={() => setIsFavDialogOpen(false)}
        initialSlot={summary.favoriteSlot}
        initialNote={summary.favoriteNote}
        onConfirm={handleFavoriteConfirm}
      />
    </div>
  );
};
