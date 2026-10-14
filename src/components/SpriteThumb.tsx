import React from "react";
import type { ThumbSprite } from "@/bindings/ThumbSprite";
import { thumbUrl } from "@/api/rhimg";

interface SpriteThumbProps {
  sprite: ThumbSprite;
  alt?: string;
  className?: string;
}

export const SpriteThumb: React.FC<SpriteThumbProps> = ({
  sprite,
  alt = "",
  className = "",
}) => {
  const imageUrl = thumbUrl(sprite.url);

  return (
    <div
      className={`relative overflow-hidden bg-gray-100 dark:bg-gray-800 flex items-center justify-center ${className}`}
      style={{ aspectRatio: `${sprite.width} / ${sprite.height}` }}
      title={alt}
    >
      <svg
        viewBox={`0 0 ${sprite.width} ${sprite.height}`}
        className="w-full h-full block select-none"
        preserveAspectRatio="xMidYMid meet"
      >
        <foreignObject width={sprite.width} height={sprite.height} x="0" y="0">
          <div
            style={{
              width: `${sprite.width}px`,
              height: `${sprite.height}px`,
              backgroundImage: `url(${imageUrl})`,
              backgroundPosition: `${sprite.offsetX}px ${sprite.offsetY}px`,
              backgroundRepeat: "no-repeat",
            }}
          />
        </foreignObject>
      </svg>
    </div>
  );
};
