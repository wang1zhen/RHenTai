import React from "react";
import { NavLink, Outlet } from "react-router";
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
  LogIn,
} from "lucide-react";
import { useT } from "@/i18n";

export const Layout: React.FC = () => {
  const { t } = useT();

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
      {/* Sidebar for >= 768px */}
      <aside className="hidden md:flex flex-col w-64 border-r border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shrink-0">
        <div className="flex items-center justify-between h-16 px-6 border-b border-gray-200 dark:border-gray-800">
          <span className="text-xl font-black tracking-wider bg-linear-to-r from-sky-500 to-indigo-600 bg-clip-text text-transparent">
            RHenTai
          </span>
          <NavLink
            to="/login"
            className="p-2 rounded-lg text-gray-500 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            title={t("login")}
          >
            <LogIn className="w-5 h-5" />
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
                  `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                    isActive
                      ? "bg-sky-50 dark:bg-sky-950/50 text-sky-600 dark:text-sky-400 font-semibold"
                      : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800/60 hover:text-gray-900 dark:hover:text-gray-200"
                  }`
                }
              >
                <Icon className="w-5 h-5 shrink-0" />
                <span>{item.label}</span>
              </NavLink>
            );
          })}
        </nav>
      </aside>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* Top bar */}
        <header className="h-16 border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-gray-900/80 backdrop-blur-md px-6 flex items-center justify-between shrink-0">
          <div className="flex items-center gap-4 flex-1 max-w-md">
            <input
              type="text"
              placeholder={t("search")}
              className="w-full px-3.5 py-1.5 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 focus:outline-hidden focus:ring-2 focus:ring-sky-500"
            />
          </div>
          <div className="flex items-center gap-3">
            <span className="px-2.5 py-1 text-xs font-semibold rounded-full bg-sky-100 dark:bg-sky-900/40 text-sky-700 dark:text-sky-300">
              EH
            </span>
          </div>
        </header>

        {/* Scrollable page body */}
        <main className="flex-1 overflow-y-auto pb-16 md:pb-0">
          <Outlet />
        </main>

        {/* Bottom Tab Bar for < 768px */}
        <nav className="md:hidden fixed bottom-0 left-0 right-0 h-14 border-t border-gray-200 dark:border-gray-800 bg-white/95 dark:bg-gray-900/95 backdrop-blur-md flex items-center justify-around px-2 z-50">
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
                  `flex flex-col items-center justify-center p-1 rounded-md text-xs font-medium transition-colors ${
                    isActive
                      ? "text-sky-600 dark:text-sky-400 font-semibold"
                      : "text-gray-500 hover:text-gray-900 dark:hover:text-gray-200"
                  }`
                }
              >
                <Icon className="w-5 h-5" />
                <span className="text-[10px] mt-0.5">{item.label}</span>
              </NavLink>
            );
          })}
        </nav>
      </div>
    </div>
  );
};
