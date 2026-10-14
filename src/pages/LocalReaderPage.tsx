import React from "react";
import { useParams, useSearchParams } from "react-router";
import { useQuery } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import { localPages, listLibrary, listHistory } from "@/api/ipc";
import { ReaderContainer } from "@/reader/ReaderContainer";
import { ErrorState } from "@/components/ErrorState";

export const LocalReaderPage: React.FC = () => {
  const { id = "" } = useParams<{ id: string }>();
  const [searchParams] = useSearchParams();

  const pageQuery = searchParams.get("page");
  const requestedPage = pageQuery ? parseInt(pageQuery, 10) : undefined;

  // Fetch local pages count
  const {
    data: pagesData,
    isLoading: isPagesLoading,
    isError,
    error,
    refetch,
  } = useQuery({
    queryKey: ["localPages", id],
    queryFn: () => localPages({ id }),
    enabled: Boolean(id),
  });

  // Fetch library to get title and category
  const { data: library = [] } = useQuery({
    queryKey: ["library"],
    queryFn: listLibrary,
  });

  // Check history for resume page if not given
  const { data: history = [] } = useQuery({
    queryKey: ["history"],
    queryFn: listHistory,
    enabled: requestedPage === undefined,
  });

  if (isPagesLoading) {
    return (
      <div className="fixed inset-0 z-50 bg-black flex items-center justify-center">
        <RefreshCw className="w-8 h-8 animate-spin text-sky-500" />
      </div>
    );
  }

  if (isError || !pagesData) {
    return (
      <div className="fixed inset-0 z-50 bg-gray-950 flex items-center justify-center">
        <ErrorState error={error} onRetry={() => refetch()} />
      </div>
    );
  }

  const libItem = library.find((l) => l.id === id);
  const historyEntry = history.find(
    (h) => h.target.kind === "local" && h.target.id === id,
  );

  const title = libItem?.title || historyEntry?.title || `Local Gallery ${id}`;
  const category = libItem?.category || historyEntry?.category || null;
  const startPage = requestedPage || historyEntry?.lastPage || libItem?.lastReadPage || 1;

  return (
    <ReaderContainer
      target={{ kind: "local", id }}
      title={title}
      thumb={null}
      category={category}
      pageCount={pagesData.count}
      initialPage={startPage}
    />
  );
};
