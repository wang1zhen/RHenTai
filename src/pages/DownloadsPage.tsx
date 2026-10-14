import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import {
  Download,
  Pause,
  Play,
  Trash2,
  FolderOpen,
  BookOpen,
  Activity,
  AlertCircle,
  X,
  CheckCircle2,
} from "lucide-react";
import { useDownloadsStore, formatSpeed } from "@/stores/downloads";
import { CategoryBadge } from "@/components/CategoryBadge";
import { ProgressBar } from "@/components/ProgressBar";
import { EmptyState } from "@/components/EmptyState";
import { openPath } from "@/api/ipc";
import { thumbUrl } from "@/api/rhimg";
import { useToastStore } from "@/components/Toasts";
import { useT, type TranslationKey } from "@/i18n";
import type { DownloadTask } from "@/bindings/DownloadTask";

export const DownloadsPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const { showToast } = useToastStore();

  const {
    tasks,
    totalSpeed,
    taskSpeeds,
    fetchTasks,
    pauseTask,
    resumeTask,
    removeTask,
    pauseAll,
    resumeAll,
  } = useDownloadsStore();

  const [taskToRemove, setTaskToRemove] = useState<DownloadTask | null>(null);
  const [deleteFilesOnDisk, setDeleteFilesOnDisk] = useState<boolean>(false);
  const [isRemoving, setIsRemoving] = useState<boolean>(false);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const handleOpenFolder = async (dir: string) => {
    try {
      await openPath({ path: dir });
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    }
  };

  const handleConfirmRemove = async () => {
    if (!taskToRemove) return;
    try {
      setIsRemoving(true);
      await removeTask(taskToRemove.id, deleteFilesOnDisk);
      setTaskToRemove(null);
      showToast(t("saveSuccess"), "success");
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
      showToast(msg, "error");
    } finally {
      setIsRemoving(false);
    }
  };

  const runningCount = tasks.filter((t) => t.state === "running").length;

  return (
    <div className="flex flex-col h-full w-full overflow-hidden">
      {/* Header */}
      <div className="px-4 sm:px-6 py-3 border-b border-gray-200 dark:border-gray-800 bg-white/70 dark:bg-gray-900/70 backdrop-blur-md shrink-0 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2.5 shrink-0 min-w-0">
          <Download className="w-5 h-5 text-sky-500 shrink-0" />
          <h1 className="text-base font-bold text-gray-900 dark:text-gray-100 whitespace-nowrap">
            {t("downloadTasks")}
          </h1>
          {runningCount > 0 && (
            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-sky-50 dark:bg-sky-950/60 border border-sky-200 dark:border-sky-800 text-sky-600 dark:text-sky-400 text-xs font-semibold shrink-0 whitespace-nowrap">
              <Activity className="w-3.5 h-3.5 animate-pulse" />
              <span>{formatSpeed(totalSpeed)}</span>
            </div>
          )}
        </div>

        {/* Global Controls */}
        <div className="flex items-center gap-2 shrink-0">
          <button
            type="button"
            onClick={() => pauseAll()}
            disabled={tasks.length === 0}
            title={t("pauseAll")}
            aria-label={t("pauseAll")}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-40 transition-colors cursor-pointer shrink-0 whitespace-nowrap"
          >
            <Pause className="w-3.5 h-3.5 shrink-0" />
            <span className="hidden md:inline whitespace-nowrap">{t("pauseAll")}</span>
          </button>
          <button
            type="button"
            onClick={() => resumeAll()}
            disabled={tasks.length === 0}
            title={t("resumeAll")}
            aria-label={t("resumeAll")}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-700 text-white disabled:opacity-40 transition-colors cursor-pointer shadow-xs shrink-0 whitespace-nowrap"
          >
            <Play className="w-3.5 h-3.5 shrink-0" />
            <span className="hidden md:inline whitespace-nowrap">{t("resumeAll")}</span>
          </button>
        </div>
      </div>

      {/* Task List */}
      <div className="flex-1 overflow-y-auto p-4 sm:p-6 pb-24 space-y-3">
        {tasks.length === 0 ? (
          <EmptyState
            icon={Download}
            title={t("noDownloads")}
            description={t("noDownloadsDesc")}
          />
        ) : (
          tasks.map((task) => {
            const currentSpeed = taskSpeeds[task.id] || 0;
            const isFinished = task.state === "completed";
            const isPaused = task.state === "paused";
            const isFailed = task.state === "failed";
            const isRunning = task.state === "running";

            const stateColor = isFinished
              ? "emerald"
              : isFailed
                ? "rose"
                : isPaused
                  ? "amber"
                  : "sky";

            return (
              <div
                key={task.id}
                data-qa="download-row"
                className="flex flex-col sm:flex-row items-start sm:items-center gap-4 p-4 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-xs hover:border-gray-300 dark:hover:border-gray-700 transition-colors"
              >
                {/* Thumb */}
                <div className="w-16 h-22 sm:w-18 sm:h-24 rounded-xl overflow-hidden bg-gray-100 dark:bg-gray-800 shrink-0 border border-gray-100 dark:border-gray-800">
                  <img
                    src={thumbUrl(task.thumb)}
                    alt={task.title}
                    loading="lazy"
                    decoding="async"
                    className="w-full h-full object-cover"
                  />
                </div>

                {/* Info & Progress */}
                <div className="flex-1 min-w-0 w-full space-y-2">
                  <div className="flex items-center gap-2">
                    <CategoryBadge category={task.category} size="sm" />
                    <h3
                      className="text-xs sm:text-sm font-bold text-gray-900 dark:text-gray-100 truncate flex-1"
                      title={task.title}
                    >
                      {task.title}
                    </h3>
                  </div>

                  {/* Progress bar */}
                  <ProgressBar
                    done={task.done}
                    total={task.total}
                    color={stateColor}
                  />

                  {/* Status row */}
                  <div className="flex flex-wrap items-center justify-between text-xs text-gray-500 dark:text-gray-400 gap-2">
                    <div className="flex items-center gap-3">
                      <span
                        className={`inline-flex items-center gap-1 font-semibold text-[11px] ${
                          isFinished
                            ? "text-emerald-500"
                            : isFailed
                              ? "text-rose-500"
                              : isPaused
                                ? "text-amber-500"
                                : "text-sky-500"
                        }`}
                      >
                        {isFinished && <CheckCircle2 className="w-3.5 h-3.5" />}
                        {isFailed && <AlertCircle className="w-3.5 h-3.5" />}
                        {t(task.state as TranslationKey)}
                      </span>

                      <span>
                        {task.done} / {task.total} {t("pages")}
                      </span>

                      {isRunning && currentSpeed > 0 && (
                        <span className="font-mono text-sky-600 dark:text-sky-400">
                          {formatSpeed(currentSpeed)}
                        </span>
                      )}
                    </div>

                    {task.error && (
                      <span className="text-[11px] text-rose-500 font-mono truncate max-w-xs">
                        {task.error}
                      </span>
                    )}
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-1.5 self-end sm:self-center shrink-0">
                  {/* Pause / Resume button */}
                  {isRunning ? (
                    <button
                      type="button"
                      onClick={() => pauseTask(task.id)}
                      className="p-2 rounded-xl text-gray-500 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                      title={t("pause")}
                      aria-label={t("pause")}
                    >
                      <Pause className="w-4 h-4" />
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={() => resumeTask(task.id)}
                      disabled={isFinished}
                      className="p-2 rounded-xl text-gray-500 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-20 transition-colors"
                      title={t("resume")}
                      aria-label={t("resume")}
                    >
                      <Play className="w-4 h-4" />
                    </button>
                  )}

                  {/* Read button */}
                  <button
                    type="button"
                    onClick={() =>
                      navigate(`/read/online/${task.gid}/${task.token}?page=1`)
                    }
                    className="p-2 rounded-xl text-sky-600 hover:bg-sky-50 dark:hover:bg-sky-950/50 transition-colors"
                    title={t("read")}
                    aria-label={t("read")}
                  >
                    <BookOpen className="w-4 h-4" />
                  </button>

                  {/* Open folder button */}
                  <button
                    type="button"
                    onClick={() => handleOpenFolder(task.dir)}
                    className="p-2 rounded-xl text-gray-500 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                    title={t("openFolder")}
                    aria-label={t("openFolder")}
                  >
                    <FolderOpen className="w-4 h-4" />
                  </button>

                  {/* Remove button */}
                  <button
                    type="button"
                    onClick={() => {
                      setTaskToRemove(task);
                      setDeleteFilesOnDisk(false);
                    }}
                    className="p-2 rounded-xl text-gray-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors"
                    title={t("remove")}
                    aria-label={t("remove")}
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Remove Download Dialog */}
      {taskToRemove && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs"
          onClick={() => setTaskToRemove(null)}
        >
          <div
            className="w-full max-w-sm rounded-2xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 p-6 space-y-4 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between">
              <h3 className="text-base font-bold text-gray-900 dark:text-gray-100">
                {t("confirmRemoveDownload")}
              </h3>
              <button
                type="button"
                onClick={() => setTaskToRemove(null)}
                className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                aria-label={t("close")}
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <p className="text-xs text-gray-600 dark:text-gray-300 line-clamp-2">
              {taskToRemove.title}
            </p>

            <label className="flex items-start gap-2.5 p-3 rounded-xl bg-gray-50 dark:bg-gray-800/60 cursor-pointer">
              <input
                type="checkbox"
                checked={deleteFilesOnDisk}
                onChange={(e) => setDeleteFilesOnDisk(e.target.checked)}
                className="mt-0.5 w-4 h-4 rounded border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-rose-600 dark:text-rose-500 accent-rose-500 focus:ring-2 focus:ring-rose-500/20 cursor-pointer"
              />
              <div className="text-xs">
                <span className="font-semibold block text-gray-900 dark:text-gray-100">
                  {t("deleteFiles")}
                </span>
                <span className="text-gray-500 text-[11px] block mt-0.5">
                  {t("deleteFilesDesc")}
                </span>
              </div>
            </label>

            <div className="flex justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={() => setTaskToRemove(null)}
                disabled={isRemoving}
                className="px-4 py-2 text-xs font-semibold rounded-xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"
              >
                {t("cancel")}
              </button>
              <button
                type="button"
                onClick={handleConfirmRemove}
                disabled={isRemoving}
                className="px-4 py-2 text-xs font-semibold rounded-xl bg-rose-600 hover:bg-rose-700 text-white shadow-xs"
              >
                {isRemoving ? t("loading") : t("confirm")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
