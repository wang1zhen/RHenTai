import React from "react";
import type { Category } from "@/bindings/Category";
import { useT, type TranslationKey } from "@/i18n";

export const CATEGORY_COLORS: Record<Category, string> = {
  doujinshi: "#9E2720",
  manga: "#DB6C24",
  artistCg: "#D38F1D",
  gameCg: "#6A936D",
  western: "#AB9F60",
  nonH: "#5FA9CF",
  imageSet: "#325CA2",
  cosplay: "#6A32A2",
  asianPorn: "#A23282",
  misc: "#777777",
  private: "#000000",
  unknown: "#555555",
};

interface CategoryBadgeProps {
  category: Category;
  size?: "sm" | "md" | "lg";
  className?: string;
  onClick?: () => void;
}

export const CategoryBadge: React.FC<CategoryBadgeProps> = ({
  category,
  size = "md",
  className = "",
  onClick,
}) => {
  const { t } = useT();
  const color = CATEGORY_COLORS[category] || CATEGORY_COLORS.unknown;
  const label = t(category as TranslationKey);

  const sizeClasses = {
    sm: "px-1.5 py-0.5 text-[10px] font-medium leading-none rounded",
    md: "px-2 py-0.5 text-xs font-semibold rounded-md",
    lg: "px-3 py-1 text-sm font-bold rounded-lg",
  }[size];

  return (
    <span
      className={`inline-flex items-center justify-center text-white shrink-0 shadow-xs select-none transition-opacity ${
        onClick ? "cursor-pointer hover:opacity-90 active:scale-95" : ""
      } ${sizeClasses} ${className}`}
      style={{ backgroundColor: color }}
      onClick={onClick}
      title={label}
    >
      {label}
    </span>
  );
};
