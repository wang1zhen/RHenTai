import { create } from "zustand";

export type Language = "zh-CN" | "en";

export const translations = {
  "zh-CN": {
    home: "首页",
    popular: "热门",
    toplist: "排行",
    watched: "关注",
    favorites: "收藏",
    downloads: "下载",
    library: "书库",
    history: "历史",
    settings: "设置",
    login: "登录",
    search: "搜索...",
    site: "站点",
    theme: "主题",
    language: "语言",
    reader: "阅读器",
    onlineReader: "在线阅读",
    localReader: "本地阅读",
    galleryDetail: "画廊详情",
    system: "跟随系统",
    light: "浅色",
    dark: "深色",
  },
  en: {
    home: "Home",
    popular: "Popular",
    toplist: "Toplist",
    watched: "Watched",
    favorites: "Favorites",
    downloads: "Downloads",
    library: "Library",
    history: "History",
    settings: "Settings",
    login: "Login",
    search: "Search...",
    site: "Site",
    theme: "Theme",
    language: "Language",
    reader: "Reader",
    onlineReader: "Online Reader",
    localReader: "Local Reader",
    galleryDetail: "Gallery Detail",
    system: "System",
    light: "Light",
    dark: "Dark",
  },
} as const;

export type TranslationKey = keyof (typeof translations)["zh-CN"];

interface I18nStore {
  language: Language;
  setLanguage: (lang: Language) => void;
}

export const useI18nStore = create<I18nStore>((set) => ({
  language: "zh-CN",
  setLanguage: (language) => set({ language }),
}));

export function useT() {
  const { language, setLanguage } = useI18nStore();
  const t = (key: TranslationKey): string => {
    return translations[language][key] || translations["zh-CN"][key] || key;
  };
  return { t, language, setLanguage };
}
