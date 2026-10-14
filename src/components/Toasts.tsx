import React from "react";
import { create } from "zustand";
import { CheckCircle2, AlertCircle, Info, X } from "lucide-react";

export type ToastType = "success" | "error" | "info";

export interface ToastItem {
  id: string;
  type: ToastType;
  message: string;
}

interface ToastStore {
  toasts: ToastItem[];
  showToast: (message: string, type?: ToastType, durationMs?: number) => void;
  removeToast: (id: string) => void;
}

export const useToastStore = create<ToastStore>((set, get) => ({
  toasts: [],
  showToast: (message, type = "info", durationMs = 3000) => {
    const id = Math.random().toString(36).slice(2, 9);
    set({ toasts: [...get().toasts, { id, type, message }] });

    setTimeout(() => {
      get().removeToast(id);
    }, durationMs);
  },
  removeToast: (id) => {
    set({ toasts: get().toasts.filter((t) => t.id !== id) });
  },
}));

export const ToastContainer: React.FC = () => {
  const { toasts, removeToast } = useToastStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-16 md:bottom-6 right-6 z-50 flex flex-col gap-2 pointer-events-none max-w-sm w-full">
      {toasts.map((toast) => {
        const Icon =
          toast.type === "success"
            ? CheckCircle2
            : toast.type === "error"
              ? AlertCircle
              : Info;

        const borderClass =
          toast.type === "success"
            ? "border-emerald-500/40 text-emerald-600 dark:text-emerald-400 bg-white/95 dark:bg-gray-900/95"
            : toast.type === "error"
              ? "border-rose-500/40 text-rose-600 dark:text-rose-400 bg-white/95 dark:bg-gray-900/95"
              : "border-sky-500/40 text-sky-600 dark:text-sky-400 bg-white/95 dark:bg-gray-900/95";

        return (
          <div
            key={toast.id}
            className={`pointer-events-auto flex items-center justify-between gap-3 p-3.5 rounded-xl border shadow-xl backdrop-blur-md animate-in slide-in-from-bottom-2 duration-200 ${borderClass}`}
          >
            <div className="flex items-center gap-2.5 min-w-0">
              <Icon className="w-4 h-4 shrink-0" />
              <p className="text-xs font-medium text-gray-800 dark:text-gray-200 truncate">
                {toast.message}
              </p>
            </div>
            <button
              type="button"
              onClick={() => removeToast(toast.id)}
              className="p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors shrink-0"
              aria-label="Close notification"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        );
      })}
    </div>
  );
};
