import React from "react";
import { useT } from "@/i18n";

export const ToplistPage: React.FC = () => {
  const { t } = useT();
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold tracking-tight mb-4">{t("toplist")}</h1>
      <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs">
        <p className="text-gray-600 dark:text-gray-400">Top ranked galleries by day, month, year, or all-time.</p>
      </div>
    </div>
  );
};
