import React, { useState } from "react";
import { useNavigate } from "react-router";
import {
  LogIn,
  Globe,
  Cookie,
  KeyRound,
  ShieldAlert,
  Loader2,
  CheckCircle2,
} from "lucide-react";
import { loginWeb, loginCookies, loginPassword } from "@/api/ipc";
import { useAuthStore } from "@/stores/auth";
import { useToastStore } from "@/components/Toasts";
import { useT } from "@/i18n";
import type { IpcError } from "@/bindings/IpcError";

type LoginTab = "web" | "cookie" | "password";

export const LoginPage: React.FC = () => {
  const { t } = useT();
  const navigate = useNavigate();
  const { showToast } = useToastStore();
  const { checkAuth } = useAuthStore();

  const [activeTab, setActiveTab] = useState<LoginTab>("web");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isCloudflare, setIsCloudflare] = useState<boolean>(false);

  // Cookie form
  const [memberId, setMemberId] = useState("");
  const [passHash, setPassHash] = useState("");
  const [igneous, setIgneous] = useState("");

  // Password form
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const handleWebLogin = async () => {
    try {
      setIsLoading(true);
      setErrorMessage(null);
      setIsCloudflare(false);
      await loginWeb();
      showToast(t("loginWindowHint"), "info", 5000);
    } catch (err: unknown) {
      handleError(err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCookieLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!memberId.trim() || !passHash.trim()) return;

    try {
      setIsLoading(true);
      setErrorMessage(null);
      setIsCloudflare(false);
      await loginCookies({
        memberId: memberId.trim(),
        passHash: passHash.trim(),
        igneous: igneous.trim() || null,
      });
      await checkAuth();
      showToast(t("loginSuccess"), "success");
      navigate(-1);
    } catch (err: unknown) {
      handleError(err);
    } finally {
      setIsLoading(false);
    }
  };

  const handlePasswordLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim() || !password.trim()) return;

    try {
      setIsLoading(true);
      setErrorMessage(null);
      setIsCloudflare(false);
      await loginPassword({
        username: username.trim(),
        password: password.trim(),
      });
      await checkAuth();
      showToast(t("loginSuccess"), "success");
      navigate(-1);
    } catch (err: unknown) {
      handleError(err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleError = (err: unknown) => {
    if (err && typeof err === "object" && "kind" in err) {
      const ipcErr = err as IpcError;
      if (ipcErr.kind === "cloudflare") {
        setIsCloudflare(true);
      }
      setErrorMessage(ipcErr.message);
    } else if (err instanceof Error) {
      setErrorMessage(err.message);
    } else {
      setErrorMessage(String(err));
    }
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-full p-4 sm:p-6 pb-20">
      <div className="w-full max-w-md rounded-3xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-xl overflow-hidden p-6 sm:p-8 space-y-6 animate-in fade-in zoom-in-95 duration-200">
        {/* Title */}
        <div className="text-center space-y-1.5">
          <div className="w-12 h-12 rounded-2xl bg-sky-500/10 text-sky-500 flex items-center justify-center mx-auto mb-3">
            <LogIn className="w-6 h-6" />
          </div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-gray-100">
            {t("login")}
          </h1>
          <p className="text-xs text-gray-500">E-Hentai / ExHentai</p>
        </div>

        {/* Tab selection */}
        <div className="grid grid-cols-3 gap-1 p-1 bg-gray-100 dark:bg-gray-800 rounded-2xl text-xs font-semibold">
          <button
            type="button"
            data-qa="web-login"
            onClick={() => {
              setActiveTab("web");
              setErrorMessage(null);
              setIsCloudflare(false);
            }}
            className={`py-2 rounded-xl transition-all cursor-pointer flex flex-col sm:flex-row items-center justify-center gap-1.5 ${
              activeTab === "web"
                ? "bg-white dark:bg-gray-900 text-sky-600 dark:text-sky-400 shadow-xs"
                : "text-gray-500 hover:text-gray-900 dark:hover:text-gray-100"
            }`}
          >
            <Globe className="w-3.5 h-3.5" />
            <span>{t("webLogin").split(" ")[0]}</span>
          </button>

          <button
            type="button"
            data-qa="cookie-login"
            onClick={() => {
              setActiveTab("cookie");
              setErrorMessage(null);
              setIsCloudflare(false);
            }}
            className={`py-2 rounded-xl transition-all cursor-pointer flex flex-col sm:flex-row items-center justify-center gap-1.5 ${
              activeTab === "cookie"
                ? "bg-white dark:bg-gray-900 text-sky-600 dark:text-sky-400 shadow-xs"
                : "text-gray-500 hover:text-gray-900 dark:hover:text-gray-100"
            }`}
          >
            <Cookie className="w-3.5 h-3.5" />
            <span>{t("cookieLogin").split(" ")[0]}</span>
          </button>

          <button
            type="button"
            data-qa="password-login"
            onClick={() => {
              setActiveTab("password");
              setErrorMessage(null);
              setIsCloudflare(false);
            }}
            className={`py-2 rounded-xl transition-all cursor-pointer flex flex-col sm:flex-row items-center justify-center gap-1.5 ${
              activeTab === "password"
                ? "bg-white dark:bg-gray-900 text-sky-600 dark:text-sky-400 shadow-xs"
                : "text-gray-500 hover:text-gray-900 dark:hover:text-gray-100"
            }`}
          >
            <KeyRound className="w-3.5 h-3.5" />
            <span>{t("passwordLogin").split(" ")[0]}</span>
          </button>
        </div>

        {/* Cloudflare Warning Banner */}
        {isCloudflare && (
          <div className="p-4 rounded-2xl bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 space-y-1 animate-in fade-in duration-200">
            <div className="flex items-center gap-2 font-bold text-xs">
              <ShieldAlert className="w-4 h-4 shrink-0" />
              <span>{t("cloudflareHintTitle")}</span>
            </div>
            <p className="text-xs text-amber-700/80 dark:text-amber-300/80 leading-relaxed">
              {t("cloudflareHintDesc")}
            </p>
          </div>
        )}

        {/* Error message */}
        {errorMessage && !isCloudflare && (
          <div className="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400 text-xs font-medium">
            {errorMessage}
          </div>
        )}

        {/* Tab 1: Web Login */}
        {activeTab === "web" && (
          <div className="space-y-4 text-center">
            <p className="text-xs text-gray-500 leading-relaxed">
              {t("loginWindowHint")}
            </p>
            <button
              type="button"
              data-qa="open-login-window"
              onClick={handleWebLogin}
              disabled={isLoading}
              className="w-full py-2.5 px-4 rounded-xl font-bold text-xs bg-sky-600 hover:bg-sky-500 active:scale-98 text-white shadow-md shadow-sky-600/20 transition-all cursor-pointer flex items-center justify-center gap-2"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Globe className="w-4 h-4" />
              )}
              <span>{t("openLoginWindow")}</span>
            </button>
          </div>
        )}

        {/* Tab 2: Cookie Login */}
        {activeTab === "cookie" && (
          <form onSubmit={handleCookieLogin} className="space-y-3.5">
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
                {t("memberId")}
              </label>
              <input
                type="text"
                required
                value={memberId}
                onChange={(e) => setMemberId(e.target.value)}
                placeholder="1234567"
                className="w-full px-3 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
                {t("passHash")}
              </label>
              <input
                type="text"
                required
                value={passHash}
                onChange={(e) => setPassHash(e.target.value)}
                placeholder="32-hex characters"
                className="w-full px-3 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
                {t("igneous")}
              </label>
              <input
                type="text"
                value={igneous}
                onChange={(e) => setIgneous(e.target.value)}
                placeholder={t("igneousOptional")}
                className="w-full px-3 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono"
              />
            </div>

            <button
              type="submit"
              data-qa="cookie-login"
              disabled={isLoading}
              className="w-full mt-2 py-2.5 px-4 rounded-xl font-bold text-xs bg-sky-600 hover:bg-sky-500 active:scale-98 text-white shadow-md shadow-sky-600/20 transition-all cursor-pointer flex items-center justify-center gap-2"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <LogIn className="w-4 h-4" />
              )}
              <span>{t("login")}</span>
            </button>
          </form>
        )}

        {/* Tab 3: Password Login */}
        {activeTab === "password" && (
          <form onSubmit={handlePasswordLogin} className="space-y-3.5">
            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
                {t("username")}
              </label>
              <input
                type="text"
                required
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="Username"
                className="w-full px-3 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500 transition-colors"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
                {t("password")}
              </label>
              <input
                type="password"
                required
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                className="w-full px-3 py-2 text-xs rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500 transition-colors"
              />
            </div>

            <button
              type="submit"
              data-qa="password-login"
              disabled={isLoading}
              className="w-full mt-2 py-2.5 px-4 rounded-xl font-bold text-xs bg-sky-600 hover:bg-sky-500 active:scale-98 text-white shadow-md shadow-sky-600/20 transition-all cursor-pointer flex items-center justify-center gap-2"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <LogIn className="w-4 h-4" />
              )}
              <span>{t("login")}</span>
            </button>
          </form>
        )}
      </div>
    </div>
  );
};
