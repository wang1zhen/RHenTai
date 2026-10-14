import React from "react";
import type { LucideIcon } from "lucide-react";
import { Inbox } from "lucide-react";

interface EmptyStateProps {
  icon?: LucideIcon;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  icon: Icon = Inbox,
  title,
  description,
  action,
  className = "",
}) => {
  return (
    <div
      className={`flex flex-col items-center justify-center p-8 text-center max-w-sm mx-auto my-auto ${className}`}
    >
      <div className="w-16 h-16 rounded-2xl bg-gray-100 dark:bg-gray-800/60 flex items-center justify-center text-gray-400 dark:text-gray-500 mb-4">
        <Icon className="w-8 h-8" />
      </div>

      <h3 className="text-base font-bold text-gray-900 dark:text-gray-100 mb-1">
        {title}
      </h3>

      {description && (
        <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed mb-5">
          {description}
        </p>
      )}

      {action && (
        <button
          type="button"
          onClick={action.onClick}
          className="inline-flex items-center justify-center px-4 py-2 text-xs font-semibold rounded-xl bg-sky-600 hover:bg-sky-700 text-white shadow-xs transition-colors cursor-pointer"
        >
          {action.label}
        </button>
      )}
    </div>
  );
};
