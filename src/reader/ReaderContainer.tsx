import React, { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "react-router";
import { VerticalReader } from "./VerticalReader";
import { PagedReader } from "./PagedReader";
import { ReaderOverlay } from "./ReaderOverlay";
import type { FitMode, ReaderMode, ReadingDirection, ReaderProps } from "./types";
import { recordProgress, getSettings } from "@/api/ipc";

export const ReaderContainer: React.FC<ReaderProps> = ({
  target,
  title,
  thumb,
  category,
  pageCount,
  initialPage = 1,
}) => {
  const navigate = useNavigate();

  const [currentPage, setCurrentPage] = useState<number>(Math.max(1, initialPage));
  const [isOverlayVisible, setIsOverlayVisible] = useState<boolean>(true);

  // Settings
  const [mode, setMode] = useState<ReaderMode>("vertical");
  const [direction, setDirection] = useState<ReadingDirection>("ltr");
  const [fit, setFit] = useState<FitMode>("width");
  const [preload, setPreload] = useState<number>(3);

  // Fetch initial reader defaults from settings
  useEffect(() => {
    getSettings()
      .then((s) => {
        if (s.reader) {
          setMode(s.reader.mode);
          setDirection(s.reader.direction);
          setFit(s.reader.fit);
          setPreload(s.reader.preload);
        }
      })
      .catch(() => {});
  }, []);

  // Debounced record_progress (1 second)
  const progressTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (progressTimeoutRef.current) {
      clearTimeout(progressTimeoutRef.current);
    }

    progressTimeoutRef.current = setTimeout(() => {
      recordProgress({
        target,
        page: currentPage,
        pageCount,
        title,
        thumb,
        category,
      }).catch((err) => {
        console.warn("Failed to record read progress:", err);
      });
    }, 1000);

    return () => {
      if (progressTimeoutRef.current) {
        clearTimeout(progressTimeoutRef.current);
      }
    };
  }, [target, currentPage, pageCount, title, thumb, category]);

  // Page navigation helpers
  const goToNextPage = useCallback(() => {
    setCurrentPage((p) => Math.min(pageCount, p + 1));
  }, [pageCount]);

  const goToPrevPage = useCallback(() => {
    setCurrentPage((p) => Math.max(1, p - 1));
  }, []);

  // Keyboard navigation
  // Keyboard: ← → ↑ ↓ PgUp PgDn Space Home End Esc
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if inside an input/textarea
      if (
        document.activeElement instanceof HTMLInputElement ||
        document.activeElement instanceof HTMLTextAreaElement
      ) {
        return;
      }

      switch (e.key) {
        case "ArrowLeft":
          e.preventDefault();
          if (mode === "paged" && direction === "rtl") {
            goToNextPage();
          } else {
            goToPrevPage();
          }
          break;
        case "ArrowRight":
          e.preventDefault();
          if (mode === "paged" && direction === "rtl") {
            goToPrevPage();
          } else {
            goToNextPage();
          }
          break;
        case "ArrowUp":
        case "PageUp":
          e.preventDefault();
          goToPrevPage();
          break;
        case "ArrowDown":
        case "PageDown":
        case " ":
          e.preventDefault();
          goToNextPage();
          break;
        case "Home":
          e.preventDefault();
          setCurrentPage(1);
          break;
        case "End":
          e.preventDefault();
          setCurrentPage(pageCount);
          break;
        case "Escape":
          e.preventDefault();
          setIsOverlayVisible((v) => !v);
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [mode, direction, pageCount, goToNextPage, goToPrevPage]);

  const handleBack = () => {
    if (window.history.length > 1) {
      navigate(-1);
    } else {
      navigate("/");
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-black h-screen w-screen overflow-hidden flex flex-col">
      <ReaderOverlay
        isVisible={isOverlayVisible}
        title={title}
        currentPage={currentPage}
        totalPages={pageCount}
        mode={mode}
        direction={direction}
        fit={fit}
        preload={preload}
        onPageChange={setCurrentPage}
        onModeChange={setMode}
        onDirectionChange={setDirection}
        onFitChange={setFit}
        onPreloadChange={setPreload}
        onBack={handleBack}
      />

      <div className="flex-1 w-full h-full overflow-hidden">
        {mode === "vertical" ? (
          <VerticalReader
            target={target}
            pageCount={pageCount}
            currentPage={currentPage}
            fit={fit}
            preload={preload}
            onPageVisible={setCurrentPage}
            onToggleOverlay={() => setIsOverlayVisible((v) => !v)}
          />
        ) : (
          <PagedReader
            target={target}
            pageCount={pageCount}
            currentPage={currentPage}
            direction={direction}
            fit={fit}
            preload={preload}
            onPageChange={setCurrentPage}
            onToggleOverlay={() => setIsOverlayVisible((v) => !v)}
          />
        )}
      </div>
    </div>
  );
};
