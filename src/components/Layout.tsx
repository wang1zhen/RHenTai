import React, { useState, useEffect } from "react";
import { NavLink, Outlet, useNavigate, useLocation } from "react-router";
import {
  Home,
  Flame,
  Trophy,
  Eye,
  Heart,
  Download,
  BookOpen,
  Clock,
  Settings as SettingsIcon,
  Search,
  User,
  LogIn,
} from "lucide-react";
import { useT } from "@/i18n";
import { useAuthStore } from "@/stores/auth";
import { useDownloadsStore } from "@/stores/downloads";
import { ToastContainer } from "@/components/Toasts";
import { getSettings, updateSettings } from "@/api/ipc";
import type { Site } from "@/bindings/Site";

export const Layout: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const location = useLocation();

  const { status, checkAuth, initListener: initAuthListener } = useAuthStore();
  const { initListeners: initDownloadListeners } = useDownloadsStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [currentSite, setCurrentSite] = useState<Site>("eh");

  useEffect(() => {
    initAuthListener();
    initDownloadListeners();
    checkAuth();
    getSettings()
      .then((s) => setCurrentSite(s.site))
      .catch(() => {});
  }, [initAuthListener, initDownloadListeners, checkAuth]);

  // Sync search input from URL param when on home page
  useEffect(() => {
    if (location.pathname === "/") {
      const params = new URLSearchParams(location.search);
      const q = params.get("q");
      if (q !== null) {
        setSearchQuery(q);
      }
    }
  }, [location.pathname, location.search]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = searchQuery.trim();
    if (trimmed) {
      navigate(`/?q=${encodeURIComponent(trimmed)}`);
    } else {
      navigate("/");
    }
  };

  const handleToggleSite = async () => {
    const nextSite: Site = currentSite === "eh" ? "ex" : "eh";
    try {
      const s = await getSettings();
      await updateSettings({ ...s, site: nextSite });
      setCurrentSite(nextSite);
      // Reload or refresh view
      window.location.reload();
    } catch {
      // Fallback
      setCurrentSite(nextSite);
    }
  };

  const navItems = [
    { to: "/", label: t("home"), icon: Home },
    { to: "/popular", label: t("popular"), icon: Flame },
    { to: "/toplist", label: t("toplist"), icon: Trophy },
    { to: "/watched", label: t("watched"), icon: Eye },
    { to: "/favorites", label: t("favorites"), icon: Heart },
    { to: "/downloads", label: t("downloads"), icon: Download },
    { to: "/library", label: t("library"), icon: BookOpen },
    { to: "/history", label: t("history"), icon: Clock },
    { to: "/settings", label: t("settings"), icon: SettingsIcon },
  ];

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-gray-50 text-gray-900 dark:bg-gray-950 dark:text-gray-100">
      {/* Toast notifications */}
      <ToastContainer />

      {/* Sidebar for >= 768px */}
      <aside className="hidden md:flex flex-col w-64 border-r border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shrink-0 select-none">
        <div className="flex items-center justify-between h-16 px-6 border-b border-gray-200 dark:border-gray-800">
          <NavLink to="/" className="flex items-center gap-2">
            <span className="text-xl font-black tracking-wider bg-linear-to-r from-sky-500 via-indigo-500 to-rose-500 bg-clip-text text-transparent">
              RHenTai
            </span>
          </NavLink>
        </div>

        <nav className="flex-1 overflow-y-auto p-4 space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === "/"}
                className={({ isActive }) =>
                  `flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-semibold transition-all ${
                    isActive
                      ? "bg-sky-500 text-white shadow-md shadow-sky-500/20"
                      : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800/60 hover:text-gray-900 dark:hover:text-gray-100"
                  }`
                }
              >
                <Icon className="w-4 h-4 shrink-0" />
                <span>{item.label}</span>
              </NavLink>
            );
          })}
        </nav>

        {/* Bottom account info in sidebar */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-800">
          {status?.loggedIn ? (
            <div className="flex items-center justify-between p-2 rounded-xl bg-gray-50 dark:bg-gray-800/60">
              <div className="flex items-center gap-2.5 min-w-0">
                <div className="w-8 h-8 rounded-lg bg-sky-500/10 text-sky-500 flex items-center justify-center font-bold text-xs shrink-0">
                  <User className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <p className="text-xs font-bold truncate">
                    {status.username || `User ${status.memberId}`}
                  </p>
                  <p className="text-[10px] text-gray-400">
                    {status.exAccess ? t("exAccess") : t("noExAccess")}
                  </p>
                </div>
              </div>
            </div>
          ) : (
            <NavLink
              to="/login"
              className="flex items-center justify-center gap-2 w-full py-2 px-3 rounded-xl bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-xs font-semibold text-gray-700 dark:text-gray-300 transition-colors"
            >
              <LogIn className="w-4 h-4" />
              <span>{t("login")}</span>
            </NavLink>
          )}
        </div>
      </aside>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* Top bar */}
        <header className="h-16 border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-gray-900/80 backdrop-blur-md px-6 flex items-center justify-between shrink-0 gap-4 z-20">
          {/* Search box */}
          <form
            onSubmit={handleSearchSubmit}
            className="flex items-center flex-1 max-w-lg relative"
          >
            <Search className="w-4 h-4 text-gray-400 absolute left-3.5 pointer-events-none" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t("searchPlaceholder")}
              className="w-full pl-9 pr-4 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/80 focus:outline-hidden focus:ring-2 focus:ring-sky-500 transition-all placeholder:text-gray-400"
            />
          </form>

          {/* Right controls: Site switch and Account badge */}
          <div className="flex items-center gap-3">
            {/* EH / EX Switch */}
            <button
              type="button"
              onClick={handleToggleSite}
              aria-label="Toggle site"
              className={`px-3 py-1.5 text-xs font-bold rounded-xl transition-all cursor-pointer shadow-xs select-none ${
                currentSite === "ex"
                  ? "bg-purple-600 hover:bg-purple-700 text-white"
                  : "bg-sky-600 hover:bg-sky-700 text-white"
              }`}
              title={currentSite === "ex" ? "ExHentai (里站)" : "E-Hentai (表站)"}
            >
              {currentSite === "ex" ? "EX" : "EH"}
            </button>

            {/* Mobile login icon */}
            <NavLink
              to={status?.loggedIn ? "/settings" : "/login"}
              className="md:hidden p-2 rounded-xl bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300"
              aria-label={t("account")}
            >
              <User className="w-4 h-4" />
            </NavLink>
          </div>
        </header>

        {/* Scrollable page body */}
        <main className="flex-1 flex flex-col min-h-0 overflow-hidden relative pb-16 md:pb-0">
          <Outlet />
        </main>

        {/* Bottom Tab Bar for < 768px */}
        <nav className="md:hidden fixed bottom-0 left-0 right-0 h-14 border-t border-gray-200 dark:border-gray-800 bg-white/95 dark:bg-gray-900/95 backdrop-blur-md flex items-center justify-around px-1 z-30">
          {[
            navItems[0], // Home
            navItems[1], // Popular
            navItems[4], // Favorites
            navItems[5], // Downloads
            navItems[6], // Library
            navItems[8], // Settings
          ].map((item) => {
            const Icon = item.icon;
            return (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === "/"}
                className={({ isActive }) =>
                  `flex flex-col items-center justify-center p-1 rounded-lg text-xs font-medium transition-colors ${
                    isActive
                      ? "text-sky-600 dark:text-sky-400 font-bold"
                      : "text-gray-500 hover:text-gray-900 dark:hover:text-gray-200"
                  }`
                }
              >
                <Icon className="w-4 h-4" />
                <span className="text-[10px] mt-0.5">{item.label}</span>
              </NavLink>
            );
          })}
        </nav>
      </div>
    </div>
  );
};
