import { type ItemRecord } from "@/lib/api";
import { useEffect, useState, memo } from "react";
import { textureCacheManager } from "@/lib/texture-cache";
import { TILE_RENDER_TYPE, LUT_4BIT, LUT_8BIT, TILE_FLAGS } from "@/lib/constants";

function getIsolatedSpriteCoords(item: ItemRecord, flags: number = 0): { x: number; y: number } {
  let coordX = item.texture_x;
  let coordY = item.texture_y;

  switch (item.render_type) {
    case TILE_RENDER_TYPE.FILLER:
    case TILE_RENDER_TYPE.SINGLE:
      break;

    case TILE_RENDER_TYPE.DIRECT8: {
      const mask = 0;
      const lutVisual = LUT_8BIT[mask];
      coordX += lutVisual % 8;
      coordY += Math.floor(lutVisual / 8);
      break;
    }

    case TILE_RENDER_TYPE.HORIZONTAL: {
      coordX += 3;
      break;
    }

    case TILE_RENDER_TYPE.ATTACH_TO_WALL_5: {
      break;
    }

    case TILE_RENDER_TYPE.DIRECT4: {
      const mask = 0;
      const lutVisual = LUT_4BIT[mask];
      coordX += lutVisual % 8;
      coordY += Math.floor(lutVisual / 8);
      break;
    }

    case TILE_RENDER_TYPE.RANDOM: {
      coordX += 0;
      break;
    }

    case TILE_RENDER_TYPE.VERTICAL: {
      coordX += 3;
      break;
    }

    case TILE_RENDER_TYPE.CAVE_PLAT: {
      coordX += 3;
      break;
    }

    case TILE_RENDER_TYPE.ATTACH_TO_WALL_4: {
      const mask = 0;
      const lutVisual = LUT_4BIT[mask];
      coordX += lutVisual % 8;
      coordY += Math.floor(lutVisual / 8);
      break;
    }

    case TILE_RENDER_TYPE.DIAGONAL: {
      break;
    }

    default:
      break;
  }

  if ((flags & TILE_FLAGS.IS_ON) !== 0) {
    if (coordX + 1 < 32) {
      coordX += 1;
    }
  }

  return { x: coordX, y: coordY };
}

export const TextureImage = memo(function TextureImage({ item, flags = 0, onLoad }: { item: ItemRecord; flags?: number; onLoad?: () => void }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function loadTexture() {
      try {
        const coords = getIsolatedSpriteCoords(item, flags);
        const croppedUrl = await textureCacheManager.getCroppedTile(
          item.texture_file_name,
          coords.x,
          coords.y
        );

        if (!cancelled) {
          setSrc(croppedUrl);
          onLoad?.();
        }
      } catch (error) {
        console.error("Failed to load texture:", error);
      }
    }

    loadTexture();
    
    return () => {
      cancelled = true;
      const coords = getIsolatedSpriteCoords(item, flags);
      textureCacheManager.releaseTile(
        item.texture_file_name,
        coords.x,
        coords.y
      );
    };
  }, [item.id, item.texture_file_name, item.texture_x, item.texture_y, flags]);

  return (
    <div className="w-full h-full flex items-center justify-center bg-muted rounded">
      {src ? (
        <img
          src={src}
          alt={item.name}
          className="max-w-full max-h-full object-contain"
          style={{ imageRendering: "pixelated" }}
          width="128"
          height="128"
          loading="lazy"
          decoding="async"
        />
      ) : (
        <div className="w-full h-full flex items-center justify-center text-muted-foreground text-xs">
          Loading...
        </div>
      )}
    </div>
  );
});
