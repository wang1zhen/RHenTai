import React, { useEffect } from "react";
import { HashRouter, Routes, Route } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Layout } from "@/components/Layout";
import { HomePage } from "@/pages/HomePage";
import { PopularPage } from "@/pages/PopularPage";
import { ToplistPage } from "@/pages/ToplistPage";
import { WatchedPage } from "@/pages/WatchedPage";
import { FavoritesPage } from "@/pages/FavoritesPage";
import { GalleryDetailPage } from "@/pages/GalleryDetailPage";
import { OnlineReaderPage } from "@/pages/OnlineReaderPage";
import { LocalReaderPage } from "@/pages/LocalReaderPage";
import { DownloadsPage } from "@/pages/DownloadsPage";
import { LibraryPage } from "@/pages/LibraryPage";
import { HistoryPage } from "@/pages/HistoryPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { LoginPage } from "@/pages/LoginPage";
import { applyTheme, useThemeStore } from "@/stores/theme";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
      retry: 1,
    },
  },
});

export const App: React.FC = () => {
  const { theme } = useThemeStore();

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<HomePage />} />
            <Route path="/popular" element={<PopularPage />} />
            <Route path="/toplist" element={<ToplistPage />} />
            <Route path="/watched" element={<WatchedPage />} />
            <Route path="/favorites" element={<FavoritesPage />} />
            <Route path="/gallery/:gid/:token" element={<GalleryDetailPage />} />
            <Route path="/read/online/:gid/:token" element={<OnlineReaderPage />} />
            <Route path="/read/local/:id" element={<LocalReaderPage />} />
            <Route path="/downloads" element={<DownloadsPage />} />
            <Route path="/library" element={<LibraryPage />} />
            <Route path="/history" element={<HistoryPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="/login" element={<LoginPage />} />
          </Route>
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  );
};
