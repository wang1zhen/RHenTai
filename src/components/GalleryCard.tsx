import React, { useState } from "react";
import { useNavigate } from "react-router";
import {
  Heart,
  FileText,
  User,
  Calendar,
  CheckSquare,
  Square,
  ImageOff,
  RefreshCw,
} from "lucide-react";
import type { GallerySummary } from "@/bindings/GallerySummary";
import { CategoryBadge } from "@/components/CategoryBadge";
import { RatingStars } from "@/components/RatingStars";
import { thumbUrl } from "@/api/rhimg";
import { useT } from "@/i18n";

interface GalleryCardProps {
  gallery: GallerySummary;
  isSelectMode?: boolean;
  isSelected?: boolean;
  onToggleSelect?: (gid: number) => void;
  className?: string;
}

export const GalleryCard: React.FC<GalleryCardProps> = ({
  gallery,
  isSelectMode = false,
  isSelected = false,
  onToggleSelect,
  className = "",
}) => {
  const { t } = useT();
  const navigate = useNavigate();
  const [hasError, setHasError] = useState(false);
  const [retryCount, setRetryCount] = useState(0);

  const handleClick = (e: React.MouseEvent) => {
    if (isSelectMode) {
      e.preventDefault();
      onToggleSelect?.(gallery.gid);
    } else {
      navigate(`/gallery/${gallery.gid}/${gallery.token}`);
    }
  };

  const formattedDate = new Date(gallery.posted * 1000).toLocaleDateString(
    undefined,
    { year: "numeric", month: "2-digit", day: "2-digit" },
  );

  return (
    <div
      data-qa="gallery-card"
      onClick={handleClick}
      className={`group relative flex flex-col rounded-2xl overflow-hidden border transition-all duration-200 cursor-pointer ${
        isSelected
          ? "border-sky-500 ring-2 ring-sky-500/30 bg-sky-50/40 dark:bg-sky-950/20 shadow-md"
          : "border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 hover:border-gray-300 dark:hover:border-gray-700 hover:shadow-lg hover:-translate-y-0.5"
      } ${className}`}
    >
      {/* Thumbnail Container */}
      <div className="relative aspect-7/10 w-full overflow-hidden bg-gray-100 dark:bg-gray-800">
        {hasError ? (
          <div
            role="button"
            tabIndex={0}
            title={t("retry")}
            aria-label={t("retry")}
            onClick={(e) => {
              e.stopPropagation();
              setHasError(false);
              setRetryCount((c) => c + 1);
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
            key={retryCount}
            src={retryCount > 0 ? `${thumbUrl(gallery.thumb)}?v=${retryCount}` : thumbUrl(gallery.thumb)}
            alt={gallery.title}
            decoding="async"
            onError={() => setHasError(true)}
            className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
          />
        )}

        {/* Top Badges */}
        <div className="absolute top-2 left-2 flex items-center gap-1.5 z-10">
          <CategoryBadge category={gallery.category} size="sm" />
        </div>

        {/* Favorite marker */}
        {gallery.favoriteSlot !== null && (
          <div
            className="absolute top-2 right-2 p-1.5 rounded-full bg-rose-500/90 text-white shadow-md z-10"
            title={`${t("favoriteSlot")}: ${gallery.favoriteSlot}`}
          >
            <Heart className="w-3.5 h-3.5 fill-current" />
          </div>
        )}

        {/* Select mode checkbox */}
        {isSelectMode && (
          <div
            className="absolute top-2 right-2 z-20 p-1 rounded-lg bg-black/60 text-white"
            onClick={(e) => {
              e.stopPropagation();
              onToggleSelect?.(gallery.gid);
            }}
          >
            {isSelected ? (
              <CheckSquare className="w-5 h-5 text-sky-400 fill-sky-400/20" />
            ) : (
              <Square className="w-5 h-5 text-white/80" />
            )}
          </div>
        )}

        {/* Page Count overlay at bottom-right of image */}
        <div className="absolute bottom-2 right-2 px-1.5 py-0.5 rounded-md bg-black/70 text-white text-[10px] font-medium flex items-center gap-1">
          <FileText className="w-3 h-3" />
          <span>{gallery.fileCount} {t("pages")}</span>
        </div>
      </div>

      {/* Info Container */}
      <div className="flex-1 flex flex-col p-3">
        {/* Title */}
        <h3
          className="font-medium text-xs sm:text-sm text-gray-900 dark:text-gray-100 line-clamp-2 leading-snug mb-2 group-hover:text-sky-600 dark:group-hover:text-sky-400 transition-colors"
          title={gallery.title}
        >
          {gallery.title}
        </h3>

        {/* Rating and metadata row */}
        <div className="mt-auto pt-1 space-y-1.5 border-t border-gray-100 dark:border-gray-800/80">
          <div className="flex items-center justify-between">
            <RatingStars rating={gallery.rating} size="sm" showScore />
            <span className="text-[10px] text-gray-400 flex items-center gap-1">
              <Calendar className="w-2.5 h-2.5" />
              {formattedDate}
            </span>
          </div>

          {gallery.uploader && (
            <div className="flex items-center gap-1 text-[11px] text-gray-500 dark:text-gray-400 truncate">
              <User className="w-3 h-3 shrink-0 opacity-70" />
              <span className="truncate">{gallery.uploader}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
