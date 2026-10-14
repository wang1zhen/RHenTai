import React, { useState } from "react";
import {
  ArrowLeft,
  Settings as SettingsIcon,
  Sliders,
  ChevronLeft,
  ChevronRight,
  Maximize2,
  X,
} from "lucide-react";
import type { FitMode, ReaderMode, ReadingDirection } from "./types";
import { useT } from "@/i18n";

interface ReaderOverlayProps {
  isVisible: boolean;
  title: string;
  currentPage: number; // 1-indexed
  totalPages: number;
  mode: ReaderMode;
  direction: ReadingDirection;
  fit: FitMode;
  preload: number;
  onPageChange: (page: number) => void;
  onModeChange: (mode: ReaderMode) => void;
  onDirectionChange: (direction: ReadingDirection) => void;
  onFitChange: (fit: FitMode) => void;
  onPreloadChange: (preload: number) => void;
  onBack: () => void;
}

export const ReaderOverlay: React.FC<ReaderOverlayProps> = ({
  isVisible,
  title,
  currentPage,
  totalPages,
  mode,
  direction,
  fit,
  preload,
  onPageChange,
  onModeChange,
  onDirectionChange,
  onFitChange,
  onPreloadChange,
  onBack,
}) => {
  const { t } = useT();
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 z-40 pointer-events-none flex flex-col justify-between select-none">
      {/* Top Header Bar */}
      <div className="pointer-events-auto flex items-center justify-between px-6 py-4 bg-gradient-to-b from-black/85 via-black/60 to-transparent text-white transition-opacity duration-200">
        <div className="flex items-center gap-3 min-w-0 flex-1 mr-4">
          <button
            type="button"
            onClick={onBack}
            className="p-2 rounded-xl bg-white/10 hover:bg-white/20 text-white backdrop-blur-md transition-colors cursor-pointer shrink-0"
            aria-label={t("back")}
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <h2 className="text-sm font-bold truncate drop-shadow-md">{title}</h2>
        </div>

        <div className="flex items-center gap-3 shrink-0">
          <span className="text-xs font-semibold px-2.5 py-1 rounded-lg bg-white/10 backdrop-blur-md">
            {currentPage} / {totalPages}
          </span>
          <button
            type="button"
            onClick={() => setIsSettingsOpen(!isSettingsOpen)}
            className={`p-2 rounded-xl backdrop-blur-md transition-colors cursor-pointer ${
              isSettingsOpen ? "bg-sky-500 text-white" : "bg-white/10 hover:bg-white/20 text-white"
            }`}
            aria-label={t("readingSettings")}
          >
            <SettingsIcon className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Settings Modal / Popover */}
      {isSettingsOpen && (
        <div className="pointer-events-auto self-end mr-6 mb-2 w-80 p-5 rounded-2xl bg-gray-900/95 text-white border border-gray-800 shadow-2xl backdrop-blur-xl animate-in slide-in-from-top-4 duration-150 space-y-4">
          <div className="flex items-center justify-between pb-2 border-b border-gray-800">
            <h3 className="text-xs font-bold uppercase tracking-wider text-gray-400">
              {t("readingSettings")}
            </h3>
            <button
              type="button"
              onClick={() => setIsSettingsOpen(false)}
              className="p-1 rounded-md text-gray-400 hover:text-white"
              aria-label={t("close")}
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          {/* Mode */}
          <div>
            <label className="block text-[11px] font-semibold text-gray-400 mb-1.5">
              {t("readerMode")}
            </label>
            <div className="grid grid-cols-2 gap-1.5 p-1 bg-gray-800/80 rounded-xl">
              <button
                type="button"
                onClick={() => onModeChange("vertical")}
                className={`py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                  mode === "vertical"
                    ? "bg-sky-600 text-white shadow-xs"
                    : "text-gray-400 hover:text-white"
                }`}
              >
                {t("vertical")}
              </button>
              <button
                type="button"
                onClick={() => onModeChange("paged")}
                className={`py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                  mode === "paged"
                    ? "bg-sky-600 text-white shadow-xs"
                    : "text-gray-400 hover:text-white"
                }`}
              >
                {t("paged")}
              </button>
            </div>
          </div>

          {/* Direction (if paged) */}
          {mode === "paged" && (
            <div>
              <label className="block text-[11px] font-semibold text-gray-400 mb-1.5">
                {t("direction")}
              </label>
              <div className="grid grid-cols-2 gap-1.5 p-1 bg-gray-800/80 rounded-xl">
                <button
                  type="button"
                  onClick={() => onDirectionChange("ltr")}
                  className={`py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    direction === "ltr"
                      ? "bg-sky-600 text-white shadow-xs"
                      : "text-gray-400 hover:text-white"
                  }`}
                >
                  {t("ltr")}
                </button>
                <button
                  type="button"
                  onClick={() => onDirectionChange("rtl")}
                  className={`py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    direction === "rtl"
                      ? "bg-sky-600 text-white shadow-xs"
                      : "text-gray-400 hover:text-white"
                  }`}
                >
                  {t("rtl")}
                </button>
              </div>
            </div>
          )}

          {/* Fit Mode */}
          <div>
            <label className="block text-[11px] font-semibold text-gray-400 mb-1.5">
              {t("fit")}
            </label>
            <div className="grid grid-cols-3 gap-1.5 p-1 bg-gray-800/80 rounded-xl text-center">
              {(["width", "height", "original"] as FitMode[]).map((f) => (
                <button
                  key={f}
                  type="button"
                  onClick={() => onFitChange(f)}
                  className={`py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer ${
                    fit === f
                      ? "bg-sky-600 text-white shadow-xs"
                      : "text-gray-400 hover:text-white"
                  }`}
                >
                  {f === "width"
                    ? t("fitWidth")
                    : f === "height"
                      ? t("fitHeight")
                      : t("fitOriginal")}
                </button>
              ))}
            </div>
          </div>

          {/* Preload */}
          <div>
            <div className="flex justify-between text-[11px] font-semibold text-gray-400 mb-1.5">
              <span>{t("preload")}</span>
              <span>{preload}</span>
            </div>
            <input
              type="range"
              min="1"
              max="10"
              value={preload}
              onChange={(e) => onPreloadChange(Number(e.target.value))}
              className="w-full accent-sky-500 cursor-pointer"
            />
          </div>
        </div>
      )}

      {/* Bottom Slider Bar */}
      <div className="pointer-events-auto flex items-center gap-4 px-6 py-4 bg-gradient-to-t from-black/85 via-black/60 to-transparent text-white transition-opacity duration-200">
        <button
          type="button"
          onClick={() => onPageChange(Math.max(1, currentPage - 1))}
          disabled={currentPage <= 1}
          className="p-1.5 rounded-lg bg-white/10 hover:bg-white/20 disabled:opacity-30 cursor-pointer transition-colors"
          aria-label={t("prevPage")}
        >
          <ChevronLeft className="w-5 h-5" />
        </button>

        <input
          type="range"
          min="1"
          max={totalPages}
          value={currentPage}
          onChange={(e) => onPageChange(Number(e.target.value))}
          className="flex-1 accent-sky-500 h-1.5 rounded-lg bg-white/20 cursor-pointer"
        />

        <button
          type="button"
          onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
          disabled={currentPage >= totalPages}
          className="p-1.5 rounded-lg bg-white/10 hover:bg-white/20 disabled:opacity-30 cursor-pointer transition-colors"
          aria-label={t("nextPage")}
        >
          <ChevronRight className="w-5 h-5" />
        </button>

        <span className="text-xs font-bold w-12 text-center shrink-0">
          {currentPage} / {totalPages}
        </span>
      </div>
    </div>
  );
};
