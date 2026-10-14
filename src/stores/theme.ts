import { create } from "zustand";

export type ThemeMode = "system" | "light" | "dark";

interface ThemeStore {
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;
}

export function applyTheme(theme: ThemeMode) {
  const root = document.documentElement;
  if (
    theme === "dark" ||
    (theme === "system" &&
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches)
  ) {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

export const useThemeStore = create<ThemeStore>((set) => ({
  theme: "system",
  setTheme: (theme) => {
    set({ theme });
    applyTheme(theme);
  },
}));
