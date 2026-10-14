import React from "react";
import { useParams, useSearchParams } from "react-router";
import { useQuery } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import { galleryDetail, listHistory } from "@/api/ipc";
import { ReaderContainer } from "@/reader/ReaderContainer";
import { ErrorState } from "@/components/ErrorState";

export const OnlineReaderPage: React.FC = () => {
  const { gid: gidStr, token } = useParams<{ gid: string; token: string }>();
  const [searchParams] = useSearchParams();
  const gid = Number(gidStr);

  const pageQuery = searchParams.get("page");
  const requestedPage = pageQuery ? parseInt(pageQuery, 10) : undefined;

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

  // Check history for resume page if not explicitly provided in URL
  const { data: history = [] } = useQuery({
    queryKey: ["history"],
    queryFn: listHistory,
    enabled: requestedPage === undefined,
  });

  if (isLoading) {
    return (
      <div className="fixed inset-0 z-50 bg-black flex items-center justify-center">
        <RefreshCw className="w-8 h-8 animate-spin text-sky-500" />
      </div>
    );
  }

  if (isError || !detail) {
    return (
      <div className="fixed inset-0 z-50 bg-gray-950 flex items-center justify-center">
        <ErrorState error={error} onRetry={() => refetch()} />
      </div>
    );
  }

  const historyEntry = history.find(
    (h) => h.target.kind === "online" && h.target.gid === gid,
  );

  const startPage = requestedPage || historyEntry?.lastPage || 1;

  return (
    <ReaderContainer
      target={{ kind: "online", gid, token: token || "" }}
      title={detail.summary.title}
      thumb={detail.summary.thumb}
      category={detail.summary.category}
      pageCount={detail.summary.fileCount}
      initialPage={startPage}
    />
  );
};
