import React, { useRef, useEffect, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ReaderPageImage } from "./ReaderPageImage";
import type { FitMode, ReadTarget } from "./types";

interface VerticalReaderProps {
  target: ReadTarget;
  pageCount: number;
  currentPage: number; // 1-indexed
  fit: FitMode;
  preload: number;
  onPageVisible: (page: number) => void;
  onToggleOverlay: () => void;
}

export const VerticalReader: React.FC<VerticalReaderProps> = ({
  target,
  pageCount,
  currentPage,
  fit,
  preload,
  onPageVisible,
  onToggleOverlay,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const isScrollingToPageRef = useRef<boolean>(false);

  const virtualizer = useVirtualizer({
    count: pageCount,
    getScrollElement: () => containerRef.current,
    estimateSize: () => 1000,
    overscan: preload,
  });

  // Jump to currentPage if triggered from slider/outside
  useEffect(() => {
    if (isScrollingToPageRef.current) return;
    const targetIndex = currentPage - 1;
    const virtualItems = virtualizer.getVirtualItems();
    const isCurrentlyVisible = virtualItems.some((v) => v.index === targetIndex);
    if (!isCurrentlyVisible && targetIndex >= 0 && targetIndex < pageCount) {
      virtualizer.scrollToIndex(targetIndex, { align: "start" });
    }
  }, [currentPage, pageCount, virtualizer]);

  // Track which page is most visible in the viewport
  const handleScroll = useCallback(() => {
    if (!containerRef.current) return;
    const items = virtualizer.getVirtualItems();
    if (items.length === 0) return;

    const scrollTop = containerRef.current.scrollTop;
    const clientHeight = containerRef.current.clientHeight;
    const middleY = scrollTop + clientHeight / 2;

    let closestPage = items[0].index + 1;
    let minDistance = Infinity;

    for (const item of items) {
      const itemMiddle = item.start + item.size / 2;
      const dist = Math.abs(itemMiddle - middleY);
      if (dist < minDistance) {
        minDistance = dist;
        closestPage = item.index + 1;
      }
    }

    if (closestPage !== currentPage) {
      isScrollingToPageRef.current = true;
      onPageVisible(closestPage);
      setTimeout(() => {
        isScrollingToPageRef.current = false;
      }, 100);
    }
  }, [virtualizer, currentPage, onPageVisible]);

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      onClick={onToggleOverlay}
      className="h-full w-full overflow-y-auto bg-black scrollbar-none"
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => (
          <div
            key={virtualItem.key}
            data-index={virtualItem.index}
            ref={virtualizer.measureElement}
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              transform: `translateY(${virtualItem.start}px)`,
            }}
          >
            <ReaderPageImage
              target={target}
              pageIndex={virtualItem.index}
              fit={fit}
            />
          </div>
        ))}
      </div>
    </div>
  );
};
