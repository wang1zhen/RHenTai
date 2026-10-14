import React from "react";

interface ProgressBarProps {
  done: number;
  total: number;
  color?: "sky" | "emerald" | "amber" | "rose" | "gray";
  showLabel?: boolean;
  className?: string;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  done,
  total,
  color = "sky",
  showLabel = false,
  className = "",
}) => {
  const percent = total > 0 ? Math.min(100, Math.round((done / total) * 100)) : 0;

  const colorClasses = {
    sky: "bg-sky-500 dark:bg-sky-400",
    emerald: "bg-emerald-500 dark:bg-emerald-400",
    amber: "bg-amber-500 dark:bg-amber-400",
    rose: "bg-rose-500 dark:bg-rose-400",
    gray: "bg-gray-400 dark:bg-gray-500",
  }[color];

  return (
    <div className={`w-full ${className}`}>
      <div className="w-full h-2 rounded-full bg-gray-100 dark:bg-gray-800 overflow-hidden">
        <div
          className={`h-full transition-all duration-300 ease-out rounded-full ${colorClasses}`}
          style={{ width: `${percent}%` }}
        />
      </div>
      {showLabel && (
        <div className="flex justify-between items-center text-[10px] font-medium text-gray-500 dark:text-gray-400 mt-1">
          <span>
            {done} / {total}
          </span>
          <span>{percent}%</span>
        </div>
      )}
    </div>
  );
};
