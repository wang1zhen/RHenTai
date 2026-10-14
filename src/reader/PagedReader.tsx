import React, { useRef, useEffect } from "react";
import { ReaderPageImage } from "./ReaderPageImage";
import type { FitMode, ReadingDirection, ReadTarget } from "./types";
import { pageUrl, localPageUrl } from "@/api/rhimg";

interface PagedReaderProps {
  target: ReadTarget;
  pageCount: number;
  currentPage: number; // 1-indexed
  direction: ReadingDirection;
  fit: FitMode;
  preload: number;
  onPageChange: (page: number) => void;
  onToggleOverlay: () => void;
}

export const PagedReader: React.FC<PagedReaderProps> = ({
  target,
  pageCount,
  currentPage,
  direction,
  fit,
  preload,
  onPageChange,
  onToggleOverlay,
}) => {
  const touchStartX = useRef<number | null>(null);

  // Preload neighboring images in background
  useEffect(() => {
    const indicesToPreload: number[] = [];
    for (let i = 1; i <= preload; i++) {
      if (currentPage + i <= pageCount) indicesToPreload.push(currentPage + i - 1);
      if (currentPage - i >= 1) indicesToPreload.push(currentPage - i - 1);
    }

    for (const idx of indicesToPreload) {
      const url =
        target.kind === "online"
          ? pageUrl(target.gid, target.token, idx)
          : localPageUrl(target.id, idx);
      const img = new Image();
      img.src = url;
    }
  }, [currentPage, pageCount, preload, target]);

  const handlePrev = () => {
    if (currentPage > 1) {
      onPageChange(currentPage - 1);
    }
  };

  const handleNext = () => {
    if (currentPage < pageCount) {
      onPageChange(currentPage + 1);
    }
  };

  // Click thirds: left third, center third, right third
  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const width = e.currentTarget.clientWidth;
    const clickX = e.clientX;

    const oneThird = width / 3;
    const twoThirds = (width * 2) / 3;

    if (clickX < oneThird) {
      // Left third clicked
      if (direction === "ltr") {
        handlePrev();
      } else {
        handleNext();
      }
    } else if (clickX > twoThirds) {
      // Right third clicked
      if (direction === "ltr") {
        handleNext();
      } else {
        handlePrev();
      }
    } else {
      // Middle third clicked: toggle overlay
      onToggleOverlay();
    }
  };

  // Touch swipe support
  const handleTouchStart = (e: React.TouchEvent) => {
    touchStartX.current = e.touches[0].clientX;
  };

  const handleTouchEnd = (e: React.TouchEvent) => {
    if (touchStartX.current === null) return;
    const diff = e.changedTouches[0].clientX - touchStartX.current;
    const threshold = 50;

    if (Math.abs(diff) > threshold) {
      if (diff > 0) {
        // Swiped right
        if (direction === "ltr") {
          handlePrev();
        } else {
          handleNext();
        }
      } else {
        // Swiped left
        if (direction === "ltr") {
          handleNext();
        } else {
          handlePrev();
        }
      }
    }
    touchStartX.current = null;
  };

  return (
    <div
      onClick={handleClick}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
      className="h-full w-full flex items-center justify-center bg-black overflow-hidden relative select-none cursor-pointer"
    >
      <ReaderPageImage
        target={target}
        pageIndex={currentPage - 1}
        fit={fit}
        className="max-h-full"
      />
    </div>
  );
};
