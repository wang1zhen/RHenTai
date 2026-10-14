import type { Category } from "@/bindings/Category";
import type { FitMode } from "@/bindings/FitMode";
import type { ReaderMode } from "@/bindings/ReaderMode";
import type { ReadingDirection } from "@/bindings/ReadingDirection";
import type { ReadTarget } from "@/bindings/ReadTarget";

export type { FitMode, ReaderMode, ReadingDirection, ReadTarget };

export interface ReaderProps {
  target: ReadTarget;
  title: string;
  thumb: string | null;
  category: Category | null;
  pageCount: number;
  initialPage?: number;
}
