import { create } from "zustand";
import type { AuthStatus } from "@/bindings/AuthStatus";
import { authStatus, logout as ipcLogout } from "@/api/ipc";
import { onAuthChanged } from "@/api/events";

interface AuthStore {
  status: AuthStatus | null;
  isLoading: boolean;
  initListener: () => void;
  checkAuth: () => Promise<AuthStatus | null>;
  logout: () => Promise<void>;
}

let listenerInitialized = false;

export const useAuthStore = create<AuthStore>((set, get) => ({
  status: null,
  isLoading: true,

  initListener: () => {
    if (listenerInitialized) return;
    listenerInitialized = true;
    onAuthChanged((newStatus) => {
      set({ status: newStatus });
    });
  },

  checkAuth: async () => {
    try {
      set({ isLoading: true });
      const res = await authStatus();
      set({ status: res, isLoading: false });
      return res;
    } catch {
      set({ status: null, isLoading: false });
      return null;
    }
  },

  logout: async () => {
    try {
      const res = await ipcLogout();
      set({ status: res });
    } catch {
      set({
        status: {
          loggedIn: false,
          memberId: null,
          username: null,
          exAccess: false,
        },
      });
    }
  },
}));
