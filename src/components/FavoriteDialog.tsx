import React, { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { X, Heart, Trash2 } from "lucide-react";
import { favoriteCategories } from "@/api/ipc";
import type { FavoriteCategory } from "@/bindings/FavoriteCategory";
import { useT } from "@/i18n";

interface FavoriteDialogProps {
  isOpen: boolean;
  onClose: () => void;
  initialSlot?: number | null;
  initialNote?: string | null;
  onConfirm: (slot: number | null, note: string) => Promise<void>;
  title?: string;
}

export const FavoriteDialog: React.FC<FavoriteDialogProps> = ({
  isOpen,
  onClose,
  initialSlot = 0,
  initialNote = "",
  onConfirm,
  title,
}) => {
  const { t } = useT();
  const [selectedSlot, setSelectedSlot] = useState<number | null>(initialSlot ?? 0);
  const [note, setNote] = useState<string>(initialNote ?? "");
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setSelectedSlot(initialSlot ?? 0);
      setNote(initialNote ?? "");
      setIsSubmitting(false);
    }
  }, [isOpen, initialSlot, initialNote]);

  const { data: categories = [], isLoading } = useQuery<FavoriteCategory[]>({
    queryKey: ["favoriteCategories"],
    queryFn: favoriteCategories,
    enabled: isOpen,
  });

  if (!isOpen) return null;

  // Ensure 10 slots 0..9 are shown even if not returned
  const slots: FavoriteCategory[] = Array.from({ length: 10 }, (_, i) => {
    const found = categories.find((c) => c.slot === i);
    return (
      found || {
        slot: i,
        name: `Favorite ${i}`,
        count: 0,
      }
    );
  });

  const handleSave = async () => {
    try {
      setIsSubmitting(true);
      await onConfirm(selectedSlot, note.trim());
      onClose();
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRemove = async () => {
    try {
      setIsSubmitting(true);
      await onConfirm(null, "");
      onClose();
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs animate-in fade-in duration-200"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-md rounded-2xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 shadow-2xl p-6 text-gray-900 dark:text-gray-100"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between pb-4 border-b border-gray-200 dark:border-gray-800 mb-4">
          <div className="flex items-center gap-2">
            <Heart className="w-5 h-5 text-rose-500 fill-current" />
            <h2 className="text-lg font-bold">{title || t("favoriteSlot")}</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label={t("close")}
            className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Slot Picker */}
        <div className="mb-4">
          <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {t("favoriteSlot")}
          </label>
          {isLoading ? (
            <div className="py-8 text-center text-sm text-gray-500">
              {t("loading")}
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-2 max-h-56 overflow-y-auto pr-1">
              {slots.map((cat) => {
                const isSelected = selectedSlot === cat.slot;
                return (
                  <button
                    key={cat.slot}
                    type="button"
                    onClick={() => setSelectedSlot(cat.slot)}
                    className={`flex items-center justify-between px-3 py-2 rounded-xl text-left text-xs font-medium border transition-all cursor-pointer ${
                      isSelected
                        ? "bg-rose-50 dark:bg-rose-950/40 border-rose-500 text-rose-600 dark:text-rose-400 font-semibold shadow-xs"
                        : "border-gray-200 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800/60 text-gray-700 dark:text-gray-300"
                    }`}
                  >
                    <span className="truncate mr-2">
                      <span className="opacity-60 mr-1.5">{cat.slot}:</span>
                      {cat.name}
                    </span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-500 shrink-0">
                      {cat.count}
                    </span>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* Note Field */}
        <div className="mb-6">
          <label className="block text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {t("favoriteNote")}
          </label>
          <textarea
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder={t("favoriteNotePlaceholder")}
            maxLength={200}
            rows={3}
            className="w-full px-3 py-2 text-sm rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/80 focus:outline-hidden focus:ring-2 focus:ring-rose-500 transition-all resize-none"
          />
          <div className="text-right text-[10px] text-gray-400 mt-1">
            {note.length} / 200
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center justify-between gap-3 pt-2">
          {initialSlot !== null && initialSlot !== undefined ? (
            <button
              type="button"
              onClick={handleRemove}
              disabled={isSubmitting}
              className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-semibold text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/50 rounded-xl transition-colors cursor-pointer"
            >
              <Trash2 className="w-4 h-4" />
              {t("removeFromFavorites")}
            </button>
          ) : (
            <div />
          )}

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="px-4 py-2 text-xs font-semibold rounded-xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              onClick={handleSave}
              disabled={isSubmitting}
              className="px-5 py-2 text-xs font-semibold rounded-xl bg-rose-600 hover:bg-rose-700 active:scale-95 text-white shadow-xs transition-all cursor-pointer"
            >
              {isSubmitting ? t("loading") : t("saveFavorite")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
