import React from "react";
import { useT } from "@/i18n";
import { useThemeStore } from "@/stores/theme";

export const SettingsPage: React.FC = () => {
  const { t, language, setLanguage } = useT();
  const { theme, setTheme } = useThemeStore();

  return (
    <div className="p-6 max-w-4xl">
      <h1 className="text-2xl font-bold tracking-tight mb-6">{t("settings")}</h1>
      <div className="space-y-6">
        <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs">
          <h2 className="text-lg font-semibold mb-4">{t("theme")}</h2>
          <div className="flex gap-4">
            {(["system", "light", "dark"] as const).map((mode) => (
              <button
                key={mode}
                onClick={() => setTheme(mode)}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  theme === mode
                    ? "bg-sky-600 text-white"
                    : "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700"
                }`}
              >
                {t(mode)}
              </button>
            ))}
          </div>
        </div>

        <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 shadow-xs">
          <h2 className="text-lg font-semibold mb-4">{t("language")}</h2>
          <div className="flex gap-4">
            <button
              onClick={() => setLanguage("zh-CN")}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                language === "zh-CN"
                  ? "bg-sky-600 text-white"
                  : "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700"
              }`}
            >
              简体中文
            </button>
            <button
              onClick={() => setLanguage("en")}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                language === "en"
                  ? "bg-sky-600 text-white"
                  : "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700"
              }`}
            >
              English
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
