import { create } from "zustand";
import type { DownloadTask } from "@/bindings/DownloadTask";
import type { DownloadProgress } from "@/bindings/DownloadProgress";
import {
  listDownloads,
  pauseDownload,
  resumeDownload,
  removeDownload,
  pauseAll as ipcPauseAll,
  resumeAll as ipcResumeAll,
} from "@/api/ipc";
import { onDownloadProgress, onDownloadsChanged } from "@/api/events";

interface DownloadsStore {
  tasks: DownloadTask[];
  taskSpeeds: Record<number, number>; // id -> bytesPerSec
  totalSpeed: number; // sum of bytesPerSec
  isLoading: boolean;
  initListeners: () => void;
  fetchTasks: () => Promise<void>;
  pauseTask: (id: number) => Promise<void>;
  resumeTask: (id: number) => Promise<void>;
  removeTask: (id: number, deleteFiles: boolean) => Promise<void>;
  pauseAll: () => Promise<void>;
  resumeAll: () => Promise<void>;
}

let listenersInitialized = false;

export const useDownloadsStore = create<DownloadsStore>((set, get) => ({
  tasks: [],
  taskSpeeds: {},
  totalSpeed: 0,
  isLoading: false,

  initListeners: () => {
    if (listenersInitialized) return;
    listenersInitialized = true;

    // Listen to download progress
    onDownloadProgress((progress: DownloadProgress) => {
      const { tasks, taskSpeeds } = get();
      const updatedSpeeds = {
        ...taskSpeeds,
        [progress.id]: progress.bytesPerSec,
      };

      // Sum speeds for running tasks
      const totalSpeed = Object.values(updatedSpeeds).reduce(
        (acc, val) => acc + (val || 0),
        0,
      );

      const updatedTasks = tasks.map((t) => {
        if (t.id === progress.id) {
          return {
            ...t,
            done: progress.done,
            total: progress.total,
            failed: progress.failed,
            state: progress.state,
            error: progress.error,
          };
        }
        return t;
      });

      set({
        tasks: updatedTasks,
        taskSpeeds: updatedSpeeds,
        totalSpeed,
      });
    });

    // Listen to downloads changed
    onDownloadsChanged(() => {
      get().fetchTasks();
    });
  },

  fetchTasks: async () => {
    try {
      set({ isLoading: true });
      const tasks = await listDownloads();
      set({ tasks, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  pauseTask: async (id: number) => {
    await pauseDownload({ id });
    set((state) => ({
      tasks: state.tasks.map((t) => (t.id === id ? { ...t, state: "paused" } : t)),
      taskSpeeds: { ...state.taskSpeeds, [id]: 0 },
    }));
  },

  resumeTask: async (id: number) => {
    await resumeDownload({ id });
    set((state) => ({
      tasks: state.tasks.map((t) => (t.id === id ? { ...t, state: "running" } : t)),
    }));
  },

  removeTask: async (id: number, deleteFiles: boolean) => {
    await removeDownload({ id, deleteFiles });
    set((state) => {
      const { [id]: _, ...restSpeeds } = state.taskSpeeds;
      return {
        tasks: state.tasks.filter((t) => t.id !== id),
        taskSpeeds: restSpeeds,
      };
    });
  },

  pauseAll: async () => {
    await ipcPauseAll();
    set((state) => ({
      tasks: state.tasks.map((t) => (t.state === "running" ? { ...t, state: "paused" } : t)),
      taskSpeeds: {},
      totalSpeed: 0,
    }));
  },

  resumeAll: async () => {
    await ipcResumeAll();
    set((state) => ({
      tasks: state.tasks.map((t) => (t.state === "paused" ? { ...t, state: "running" } : t)),
    }));
  },
}));

export function formatSpeed(bytesPerSec: number): string {
  if (!bytesPerSec || bytesPerSec <= 0) return "0 B/s";
  if (bytesPerSec < 1024) return `${bytesPerSec} B/s`;
  if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}
