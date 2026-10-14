import React from "react";
import { useT } from "@/i18n";

export const LoginPage: React.FC = () => {
  const { t } = useT();
  return (
    <div className="p-6 max-w-xl mx-auto">
      <h1 className="text-2xl font-bold tracking-tight mb-6">{t("login")}</h1>
      <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs">
        <p className="text-gray-600 dark:text-gray-400">
          Login using webview, session cookies, or password.
        </p>
      </div>
    </div>
  );
};
