import React from "react";
import { useParams } from "react-router";
import { useT } from "@/i18n";

export const GalleryDetailPage: React.FC = () => {
  const { gid, token } = useParams<{ gid: string; token: string }>();
  const { t } = useT();

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold tracking-tight mb-4">{t("galleryDetail")}</h1>
      <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs space-y-2">
        <p className="text-gray-600 dark:text-gray-400">
          GID: <span className="font-mono text-gray-900 dark:text-gray-100">{gid}</span>
        </p>
        <p className="text-gray-600 dark:text-gray-400">
          Token: <span className="font-mono text-gray-900 dark:text-gray-100">{token}</span>
        </p>
      </div>
    </div>
  );
};
