import React from "react";
import { useT } from "@/i18n";

export const PopularPage: React.FC = () => {
  const { t } = useT();
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold tracking-tight mb-4">{t("popular")}</h1>
      <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs">
        <p className="text-gray-600 dark:text-gray-400">Popular galleries across the site.</p>
      </div>
    </div>
  );
};
