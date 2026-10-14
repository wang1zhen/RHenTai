import React from "react";
import { AlertTriangle, RefreshCw, ShieldAlert, LogIn, WifiOff } from "lucide-react";
import type { IpcError } from "@/bindings/IpcError";
import type { ErrorKind } from "@/bindings/ErrorKind";
import { useT, type TranslationKey } from "@/i18n";
import { useNavigate } from "react-router";

interface ErrorStateProps {
  error: IpcError | Error | null | unknown;
  onRetry?: () => void;
  className?: string;
}

export const ErrorState: React.FC<ErrorStateProps> = ({
  error,
  onRetry,
  className = "",
}) => {
  const { t } = useT();
  const navigate = useNavigate();

  let kind: ErrorKind | "unknown" = "unknown";
  let message = "";

  if (error && typeof error === "object" && "kind" in error) {
    const ipcErr = error as IpcError;
    kind = ipcErr.kind;
    message = ipcErr.message;
  } else if (error instanceof Error) {
    message = error.message;
  } else if (typeof error === "string") {
    message = error;
  }

  const kindHeadlineKey: Record<ErrorKind, TranslationKey> = {
    network: "errorNetwork",
    http: "errorHttp",
    parse: "errorParse",
    cloudflare: "errorCloudflare",
    banned: "errorBanned",
    loginRequired: "errorLoginRequired",
    loginFailed: "errorLoginFailed",
    noExAccess: "errorNoExAccess",
    imageLimit: "errorImageLimit",
    notFound: "errorNotFound",
    invalidInput: "errorInvalidInput",
    io: "errorIo",
    db: "errorDb",
    cancelled: "errorCancelled",
    internal: "errorInternal",
  };

  const headline =
    kind in kindHeadlineKey
      ? t(kindHeadlineKey[kind as ErrorKind])
      : t("unknownError");

  // Icon selection
  const Icon = (() => {
    if (kind === "network") return WifiOff;
    if (kind === "cloudflare" || kind === "banned") return ShieldAlert;
    if (kind === "loginRequired" || kind === "loginFailed") return LogIn;
    return AlertTriangle;
  })();

  return (
    <div
      className={`flex flex-col items-center justify-center p-8 text-center max-w-md mx-auto my-auto ${className}`}
    >
      <div className="w-14 h-14 rounded-2xl bg-rose-100 dark:bg-rose-950/50 flex items-center justify-center text-rose-600 dark:text-rose-400 mb-4 shadow-xs">
        <Icon className="w-7 h-7" />
      </div>

      <h3 className="text-base font-bold text-gray-900 dark:text-gray-100 mb-1.5">
        {headline}
      </h3>

      {message && (
        <p className="text-xs text-gray-500 dark:text-gray-400 font-mono bg-gray-100 dark:bg-gray-800/80 px-3 py-1.5 rounded-lg max-w-full break-all mb-5">
          {message}
        </p>
      )}

      <div className="flex items-center gap-3">
        {kind === "loginRequired" && (
          <button
            type="button"
            onClick={() => navigate("/login")}
            className="inline-flex items-center gap-2 px-4 py-2 text-xs font-semibold rounded-xl bg-sky-600 hover:bg-sky-700 text-white shadow-xs transition-colors cursor-pointer"
          >
            <LogIn className="w-4 h-4" />
            {t("goToLogin")}
          </button>
        )}

        {onRetry && (
          <button
            type="button"
            onClick={onRetry}
            className="inline-flex items-center gap-2 px-4 py-2 text-xs font-semibold rounded-xl bg-gray-900 hover:bg-gray-800 dark:bg-gray-100 dark:hover:bg-white text-white dark:text-gray-900 shadow-xs transition-colors cursor-pointer"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            {t("retryAction")}
          </button>
        )}
      </div>
    </div>
  );
};
