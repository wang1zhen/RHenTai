import React, { useState, useEffect } from "react";
import { AlertCircle, RefreshCw, Loader2 } from "lucide-react";
import { pageUrl, localPageUrl } from "@/api/rhimg";
import { reloadPageImage } from "@/api/ipc";
import type { FitMode, ReadTarget } from "./types";
import { useT } from "@/i18n";

interface ReaderPageImageProps {
  target: ReadTarget;
  pageIndex: number; // 0-based
  fit: FitMode;
  onLoaded?: () => void;
  className?: string;
}

export const ReaderPageImage: React.FC<ReaderPageImageProps> = ({
  target,
  pageIndex,
  fit,
  onLoaded,
  className = "",
}) => {
  const { t } = useT();
  const [version, setVersion] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [hasError, setHasError] = useState<boolean>(false);
  const [isRetrying, setIsRetrying] = useState<boolean>(false);

  const imageUrl =
    target.kind === "online"
      ? pageUrl(target.gid, target.token, pageIndex, version > 0 ? version : undefined)
      : localPageUrl(target.id, pageIndex);

  const handleRetry = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsRetrying(true);
    setHasError(false);

    try {
      if (target.kind === "online") {
        await reloadPageImage({
          gid: target.gid,
          token: target.token,
          index: pageIndex,
        });
      }
      // Bump version query param
      setVersion((v) => v + 1);
      setIsLoading(true);
    } catch {
      setHasError(true);
    } finally {
      setIsRetrying(false);
    }
  };

  const fitClasses = {
    width: "w-full h-auto max-w-full object-contain block mx-auto",
    height: "h-screen w-auto max-h-screen object-contain block mx-auto",
    original: "max-w-none max-h-none block mx-auto",
  }[fit];

  return (
    <div
      className={`relative min-h-[300px] flex items-center justify-center bg-gray-950/40 select-none ${className}`}
      data-page-index={pageIndex}
    >
      {/* Loading state */}
      {isLoading && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 text-gray-500 bg-gray-950/20 z-10">
          <Loader2 className="w-8 h-8 animate-spin text-sky-500" />
          <span className="text-xs font-medium">Page {pageIndex + 1}</span>
        </div>
      )}

      {/* Error state */}
      {hasError ? (
        <div className="flex flex-col items-center justify-center p-8 gap-3 text-center my-12 bg-gray-900/80 rounded-2xl border border-rose-900/50 max-w-sm z-20">
          <AlertCircle className="w-8 h-8 text-rose-500" />
          <p className="text-xs text-rose-400 font-semibold">
            {t("pageError")} (Page {pageIndex + 1})
          </p>
          <button
            type="button"
            data-qa="reader-retry"
            onClick={handleRetry}
            disabled={isRetrying}
            className="inline-flex items-center gap-2 px-4 py-2 text-xs font-semibold rounded-xl bg-rose-600 hover:bg-rose-500 text-white shadow-xs cursor-pointer transition-colors"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isRetrying ? "animate-spin" : ""}`} />
            <span>{t("retry")}</span>
          </button>
        </div>
      ) : (
        <img
          key={imageUrl}
          data-qa="reader-page"
          src={imageUrl}
          alt={`Page ${pageIndex + 1}`}
          loading="lazy"
          decoding="async"
          className={`${fitClasses} transition-opacity duration-200 ${
            isLoading ? "opacity-0" : "opacity-100"
          }`}
          onLoad={() => {
            setIsLoading(false);
            setHasError(false);
            onLoaded?.();
          }}
          onError={() => {
            setIsLoading(false);
            setHasError(true);
          }}
        />
      )}
    </div>
  );
};
