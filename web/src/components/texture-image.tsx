import { type ItemRecord } from "@/lib/api";
import { useEffect, useState, memo } from "react";
import { textureCacheManager } from "@/lib/texture-cache";
import { TILE_RENDER_TYPE, LUT_4BIT, LUT_8BIT, TILE_FLAGS } from "@/lib/constants";

// Calculate sprite coordinates for an isolated item (no neighbors)
function getIsolatedSpriteCoords(item: ItemRecord, flags: number = 0): { x: number; y: number } {
  let coordX = item.texture_x;
  let coordY = item.texture_y;

  switch (item.render_type) {
    case TILE_RENDER_TYPE.FILLER:
    case TILE_RENDER_TYPE.SINGLE:
      // No modification needed - use base coordinates
      break;

    case TILE_RENDER_TYPE.DIRECT8: {
      // All neighbors are false (isolated tile)
      const mask = 0; // No neighbors
      const lutVisual = LUT_8BIT[mask];
      coordX += lutVisual % 8;
      coordY += Math.floor(lutVisual / 8);
      break;
    }

    case TILE_RENDER_TYPE.HORIZONTAL: {
      // No left or right neighbors - isolated
      coordX += 3; // Isolated state
      break;
    }

    case TILE_RENDER_TYPE.ATTACH_TO_WALL_5: {
      // This would require checking for solid tiles below - use base for preview
      break;
    }

    case TILE_RENDER_TYPE.DIRECT4: {
      // No neighbors (top, left, right, bottom all false)
      const mask = 0;
      const lutVisual = LUT_4BIT[mask];
      coordX += lutVisual % 8;
      coordY += Math.floor(lutVisual / 8);
      break;
    }

    case TILE_RENDER_TYPE.RANDOM: {
      // Use first variant for consistency
      coordX += 0;
      break;
    }

    case TILE_RENDER_TYPE.VERTICAL: {
      // No top or bottom neighbors - isolated
      coordX += 3; // Isolated state
      break;
    }

    case TILE_RENDER_TYPE.CAVE_PLAT: {
      // Similar to HORIZONTAL
      coordX += 3; // Isolated state
      break;
    }

    case TILE_RENDER_TYPE.ATTACH_TO_WALL_4: {
      // No neighbors
      const mask = 0;
      const lutVisual = LUT_4BIT[mask];
      coordX += lutVisual % 8;
      coordY += Math.floor(lutVisual / 8);
      break;
    }

    case TILE_RENDER_TYPE.DIAGONAL: {
      // Diagonal render types use base coordinates
      break;
    }

    default:
      // Unknown render type - use base coordinates
      break;
  }

  // Apply IS_ON flag offset if active
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
        // Calculate sprite coordinates based on render type
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
