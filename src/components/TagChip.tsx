import React from "react";
import { useNavigate } from "react-router";
import type { Tag } from "@/bindings/Tag";

interface TagChipProps {
  tag: Tag;
  onClick?: (query: string) => void;
  className?: string;
}

export const TagChip: React.FC<TagChipProps> = ({ tag, onClick, className = "" }) => {
  const navigate = useNavigate();

  const formattedQuery = tag.namespace
    ? `${tag.namespace}:"${tag.name}$"`
    : `"${tag.name}$"`;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onClick) {
      onClick(formattedQuery);
    } else {
      navigate(`/?q=${encodeURIComponent(formattedQuery)}`);
    }
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      title={formattedQuery}
      aria-label={`Search tag ${formattedQuery}`}
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-md transition-colors bg-gray-100 hover:bg-sky-100 dark:bg-gray-800 dark:hover:bg-sky-950/60 text-gray-700 hover:text-sky-600 dark:text-gray-300 dark:hover:text-sky-400 border border-transparent hover:border-sky-300 dark:hover:border-sky-800 cursor-pointer ${className}`}
    >
      {tag.namespace && (
        <span className="text-gray-400 dark:text-gray-500 font-normal">
          {tag.namespace}:
        </span>
      )}
      <span>{tag.name}</span>
    </button>
  );
};
