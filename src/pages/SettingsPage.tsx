import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Settings as SettingsIcon,
  Globe,
  Download,
  BookOpen,
  Palette,
  Folder,
  Trash2,
  HardDrive,
  User,
  LogIn,
  LogOut,
  FolderPlus,
  RefreshCw,
  Save,
  CheckCircle2,
} from "lucide-react";
import {
  getSettings,
  updateSettings,
  clearImageCache,
} from "@/api/ipc";
import { useAuthStore } from "@/stores/auth";
import { useThemeStore } from "@/stores/theme";
import { useI18nStore, useT } from "@/i18n";
import { useToastStore } from "@/components/Toasts";
import type { Settings } from "@/bindings/Settings";
import type { Site } from "@/bindings/Site";
import type { Theme } from "@/bindings/Theme";
import type { UiLanguage } from "@/bindings/UiLanguage";
import type { ReaderMode } from "@/bindings/ReaderMode";
import type { ReadingDirection } from "@/bindings/ReadingDirection";
import type { FitMode } from "@/bindings/FitMode";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

const formControlClass =
  "w-full px-3 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500 transition-colors";

const formCheckboxClass =
  "w-4 h-4 rounded border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-emerald-600 dark:text-emerald-500 accent-emerald-500 focus:ring-2 focus:ring-emerald-500/20 cursor-pointer";

export const SettingsPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { showToast } = useToastStore();

  const { status, logout } = useAuthStore();
  const { theme, setTheme } = useThemeStore();
  const { language, setLanguage } = useI18nStore();

  const [form, setForm] = useState<Settings | null>(null);
  const [newLibPath, setNewLibPath] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [isClearingCache, setIsClearingCache] = useState(false);

  const { data: initialSettings, isLoading } = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
  });

  useEffect(() => {
    if (initialSettings) {
      setForm(initialSettings);
    }
  }, [initialSettings]);

  if (isLoading || !form) {
    return (
      <div className="flex-1 flex items-center justify-center h-full">
        <RefreshCw className="w-8 h-8 animate-spin text-sky-500" />
      </div>
    );
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      setIsSaving(true);
      await updateSettings(form);
      queryClient.invalidateQueries({ queryKey: ["settings"] });
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    } finally {
      setIsSaving(false);
    }
  };

  const handleBrowseDownloadDir = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const res = await open({ directory: true, multiple: false });
      if (typeof res === "string") {
        setForm({ ...form, downloadDir: res });
      }
    } catch {
      const path = window.prompt("Enter download directory path:", form.downloadDir);
      if (path) setForm({ ...form, downloadDir: path });
    }
  };

  const handleAddLibraryPath = async () => {
    if (!newLibPath.trim()) return;
    if (form.libraryDirs.includes(newLibPath.trim())) return;
    setForm({
      ...form,
      libraryDirs: [...form.libraryDirs, newLibPath.trim()],
    });
    setNewLibPath("");
  };

  const handleRemoveLibraryPath = (path: string) => {
    setForm({
      ...form,
      libraryDirs: form.libraryDirs.filter((p) => p !== path),
    });
  };

  const handleClearCache = async () => {
    try {
      setIsClearingCache(true);
      const res = await clearImageCache();
      showToast(
        t("cacheClearedToast", { bytes: formatBytes(res.bytes) }),
        "success",
      );
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    } finally {
      setIsClearingCache(false);
    }
  };

  return (
    <div className="flex-1 min-h-0 flex flex-col w-full overflow-hidden">
      {/* Header */}
      <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <SettingsIcon className="w-5 h-5 text-gray-500" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100">
            {t("settings")}
          </h1>
        </div>

        <button
          type="button"
          onClick={handleSave}
          disabled={isSaving}
          className="inline-flex items-center gap-1.5 px-4 py-1.5 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-700 text-white shadow-xs transition-colors cursor-pointer"
        >
          <Save className="w-3.5 h-3.5" />
          <span>{isSaving ? t("loading") : t("save")}</span>
        </button>
      </div>

      {/* Form Content */}
      <form
        onSubmit={handleSave}
        className="flex-1 min-h-0 overflow-y-auto p-4 sm:p-6 pb-28 space-y-6 max-w-4xl mx-auto w-full"
      >
        {/* Account Section */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <User className="w-4 h-4 text-sky-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("accountSettings")}
            </h2>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-4">
            <div>
              {status?.loggedIn ? (
                <div className="space-y-1">
                  <p className="text-xs font-bold text-gray-900 dark:text-gray-100">
                    {t("loggedInAs")}: {status.username || `Member ${status.memberId}`}
                  </p>
                  <p className="text-xs text-gray-500">
                    {status.exAccess ? t("exAccess") : t("noExAccess")}
                  </p>
                </div>
              ) : (
                <div className="space-y-1">
                  <p className="text-xs font-bold text-gray-900 dark:text-gray-100">
                    {t("notLoggedIn")}
                  </p>
                  <p className="text-xs text-gray-400">
                    {t("loginRequiredFavoritesDesc")}
                  </p>
                </div>
              )}
            </div>

            {status?.loggedIn ? (
              <button
                type="button"
                onClick={() => logout()}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold border border-rose-200 dark:border-rose-900/60 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer"
              >
                <LogOut className="w-3.5 h-3.5" />
                <span>{t("logout")}</span>
              </button>
            ) : (
              <button
                type="button"
                onClick={() => navigate("/login")}
                className="inline-flex items-center gap-1.5 px-4 py-1.5 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-700 text-white shadow-xs transition-colors cursor-pointer"
              >
                <LogIn className="w-3.5 h-3.5" />
                <span>{t("login")}</span>
              </button>
            )}
          </div>
        </section>

        {/* Appearance & Language (immediate update) */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <Palette className="w-4 h-4 text-purple-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("appearanceSettings")}
            </h2>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {/* Theme */}
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("theme")}
              </label>
              <select
                value={form.theme}
                onChange={(e) => {
                  const val = e.target.value as Theme;
                  setForm({ ...form, theme: val });
                  setTheme(val);
                }}
                className={formControlClass}
              >
                <option value="system">{t("systemTheme")}</option>
                <option value="light">{t("lightTheme")}</option>
                <option value="dark">{t("darkTheme")}</option>
              </select>
            </div>

            {/* Language */}
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("language")}
              </label>
              <select
                value={form.language}
                onChange={(e) => {
                  const val = e.target.value as UiLanguage;
                  setForm({ ...form, language: val });
                  setLanguage(val);
                }}
                className={formControlClass}
              >
                <option value="zh-CN">简体中文 (zh-CN)</option>
                <option value="en">English (en)</option>
              </select>
            </div>
          </div>
        </section>

        {/* Site & Network Settings */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <Globe className="w-4 h-4 text-sky-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("siteSettings")}
            </h2>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {/* Site */}
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("site")}
              </label>
              <select
                value={form.site}
                onChange={(e) => setForm({ ...form, site: e.target.value as Site })}
                className={formControlClass}
              >
                <option value="eh">E-Hentai (表站)</option>
                <option value="ex">ExHentai (里站)</option>
              </select>
            </div>

            {/* Proxy */}
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("proxySettings")}
              </label>
              <input
                type="text"
                value={form.proxy ?? ""}
                onChange={(e) =>
                  setForm({ ...form, proxy: e.target.value.trim() || null })
                }
                placeholder={t("proxyPlaceholder")}
                className={formControlClass}
              />
            </div>
          </div>
        </section>

        {/* Download Settings */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <Download className="w-4 h-4 text-emerald-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("downloadSettings")}
            </h2>
          </div>

          <div className="space-y-4">
            {/* Download Dir */}
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("downloadDir")}
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={form.downloadDir}
                  onChange={(e) => setForm({ ...form, downloadDir: e.target.value })}
                  className={`flex-1 ${formControlClass}`}
                />
                <button
                  type="button"
                  onClick={handleBrowseDownloadDir}
                  className="px-3 py-2 text-xs font-semibold rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                >
                  {t("browse")}
                </button>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {/* Concurrency */}
              <div>
                <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                  {t("concurrency")}: {form.downloadConcurrency}
                </label>
                <input
                  type="range"
                  min="1"
                  max="10"
                  value={form.downloadConcurrency}
                  onChange={(e) =>
                    setForm({
                      ...form,
                      downloadConcurrency: Number(e.target.value),
                    })
                  }
                  className="w-full accent-emerald-500"
                />
              </div>

              {/* Download original */}
              <div className="flex items-center pt-4">
                <label className="flex items-center gap-2.5 cursor-pointer text-xs font-medium text-gray-800 dark:text-gray-200">
                  <input
                    type="checkbox"
                    checked={form.downloadOriginal}
                    onChange={(e) =>
                      setForm({ ...form, downloadOriginal: e.target.checked })
                    }
                    className={formCheckboxClass}
                  />
                  <span>{t("downloadOriginal")}</span>
                </label>
              </div>
            </div>
          </div>
        </section>

        {/* Default Reader Settings */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <BookOpen className="w-4 h-4 text-indigo-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("readerSettings")}
            </h2>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 text-xs">
            {/* Mode */}
            <div>
              <label className="block font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("readerMode")}
              </label>
              <select
                value={form.reader.mode}
                onChange={(e) =>
                  setForm({
                    ...form,
                    reader: { ...form.reader, mode: e.target.value as ReaderMode },
                  })
                }
                className={formControlClass}
              >
                <option value="vertical">{t("vertical")}</option>
                <option value="paged">{t("paged")}</option>
              </select>
            </div>

            {/* Direction */}
            <div>
              <label className="block font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("direction")}
              </label>
              <select
                value={form.reader.direction}
                onChange={(e) =>
                  setForm({
                    ...form,
                    reader: {
                      ...form.reader,
                      direction: e.target.value as ReadingDirection,
                    },
                  })
                }
                className={formControlClass}
              >
                <option value="ltr">{t("ltr")}</option>
                <option value="rtl">{t("rtl")}</option>
              </select>
            </div>

            {/* Fit */}
            <div>
              <label className="block font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("fit")}
              </label>
              <select
                value={form.reader.fit}
                onChange={(e) =>
                  setForm({
                    ...form,
                    reader: { ...form.reader, fit: e.target.value as FitMode },
                  })
                }
                className={formControlClass}
              >
                <option value="width">{t("fitWidth")}</option>
                <option value="height">{t("fitHeight")}</option>
                <option value="original">{t("fitOriginal")}</option>
              </select>
            </div>

            {/* Preload */}
            <div>
              <label className="block font-semibold text-gray-500 dark:text-gray-400 mb-1.5">
                {t("preload")}
              </label>
              <input
                type="number"
                min="1"
                max="10"
                value={form.reader.preload}
                onChange={(e) =>
                  setForm({
                    ...form,
                    reader: {
                      ...form.reader,
                      preload: Math.max(1, Number(e.target.value)),
                    },
                  })
                }
                className={formControlClass}
              />
            </div>
          </div>
        </section>

        {/* Library Folders */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <Folder className="w-4 h-4 text-amber-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("libraryFolders")}
            </h2>
          </div>

          <div className="space-y-3">
            <div className="flex gap-2">
              <input
                type="text"
                value={newLibPath}
                onChange={(e) => setNewLibPath(e.target.value)}
                placeholder="/path/to/manga/folder"
                className={`flex-1 ${formControlClass}`}
              />
              <button
                type="button"
                onClick={handleAddLibraryPath}
                className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-semibold rounded-xl bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              >
                <FolderPlus className="w-3.5 h-3.5" />
                <span>{t("addPath")}</span>
              </button>
            </div>

            {form.libraryDirs.length > 0 && (
              <div className="space-y-1.5">
                {form.libraryDirs.map((dir) => (
                  <div
                    key={dir}
                    className="flex items-center justify-between p-2.5 rounded-xl bg-gray-50 dark:bg-gray-800/50 text-xs font-mono"
                  >
                    <span className="truncate mr-2 text-gray-700 dark:text-gray-300">
                      {dir}
                    </span>
                    <button
                      type="button"
                      onClick={() => handleRemoveLibraryPath(dir)}
                      className="p-1 rounded-md text-gray-400 hover:text-rose-500 transition-colors"
                      title={t("removePath")}
                      aria-label={t("removePath")}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>

        {/* Cache Management */}
        <section className="p-5 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 space-y-4 shadow-xs">
          <div className="flex items-center gap-2 pb-3 border-b border-gray-100 dark:border-gray-800">
            <HardDrive className="w-4 h-4 text-rose-500" />
            <h2 className="text-sm font-bold text-gray-900 dark:text-gray-100">
              {t("cacheSettings")}
            </h2>
          </div>

          <div className="flex items-center justify-between">
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {t("clearImageCache")}
            </p>
            <button
              type="button"
              onClick={handleClearCache}
              disabled={isClearingCache}
              className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-semibold rounded-xl border border-rose-200 dark:border-rose-900/60 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer"
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span>{isClearingCache ? t("loading") : t("clearImageCache")}</span>
            </button>
          </div>
        </section>
      </form>
    </div>
  );
};
