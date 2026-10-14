import React from "react";
import { Star } from "lucide-react";

interface RatingStarsProps {
  rating: number; // 0.0 to 5.0
  showScore?: boolean;
  size?: "sm" | "md" | "lg";
  className?: string;
}

export const RatingStars: React.FC<RatingStarsProps> = ({
  rating,
  showScore = false,
  size = "md",
  className = "",
}) => {
  const clampedRating = Math.max(0, Math.min(5, rating));

  const starSizes = {
    sm: "w-3 h-3",
    md: "w-3.5 h-3.5",
    lg: "w-4 h-4",
  }[size];

  return (
    <div className={`inline-flex items-center gap-1 text-amber-400 select-none ${className}`}>
      <div className="flex items-center">
        {[0, 1, 2, 3, 4].map((index) => {
          const fillAmount = Math.max(0, Math.min(1, clampedRating - index));
          return (
            <div key={index} className="relative inline-block">
              {/* Background empty star */}
              <Star
                className={`${starSizes} text-gray-300 dark:text-gray-700 stroke-none fill-current`}
              />
              {/* Foreground filled star with clip-path */}
              {fillAmount > 0 && (
                <div
                  className="absolute inset-0 overflow-hidden"
                  style={{ width: `${fillAmount * 100}%` }}
                >
                  <Star className={`${starSizes} text-amber-400 stroke-none fill-current`} />
                </div>
              )}
            </div>
          );
        })}
      </div>
      {showScore && (
        <span className="text-xs font-semibold text-gray-700 dark:text-gray-300 ml-1">
          {clampedRating.toFixed(2)}
        </span>
      )}
    </div>
  );
};
