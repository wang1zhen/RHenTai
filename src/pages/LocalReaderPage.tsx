import React from "react";
import { useParams, useSearchParams } from "react-router";
import { useT } from "@/i18n";

export const LocalReaderPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [searchParams] = useSearchParams();
  const page = searchParams.get("page") ?? "0";
  const { t } = useT();

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold tracking-tight mb-4">{t("localReader")}</h1>
      <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs space-y-2">
        <p className="text-gray-600 dark:text-gray-400">
          Reading local gallery {id} at page {page}.
        </p>
      </div>
    </div>
  );
};
