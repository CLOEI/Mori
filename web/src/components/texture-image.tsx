import { type ItemRecord } from "@/lib/api";
import { useEffect, useState } from "react";
import { textureCacheManager } from "@/lib/texture-cache";
import { TILE_RENDER_TYPE, LUT_4BIT, LUT_8BIT } from "@/lib/constants";

// Calculate sprite coordinates for an isolated item (no neighbors)
function getIsolatedSpriteCoords(item: ItemRecord): { x: number; y: number } {
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

    default:
      // Unknown render type - use base coordinates
      break;
  }

  return { x: coordX, y: coordY };
}

export function TextureImage({ item }: { item: ItemRecord }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function loadTexture() {
      try {
        // Calculate sprite coordinates based on render type
        const coords = getIsolatedSpriteCoords(item);
        
        const croppedUrl = await textureCacheManager.getCroppedTile(
          item.texture_file_name,
          coords.x,
          coords.y
        );

        if (!cancelled) {
          setSrc(croppedUrl);
        }
      } catch (error) {
        console.error("Failed to load texture:", error);
      }
    }

    loadTexture();
    
    return () => {
      cancelled = true;
      const coords = getIsolatedSpriteCoords(item);
      textureCacheManager.releaseTile(
        item.texture_file_name,
        coords.x,
        coords.y
      );
    };
  }, [item]);

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
        />
      ) : (
        <div className="w-full h-full flex items-center justify-center text-muted-foreground text-xs">
          Loading...
        </div>
      )}
    </div>
  );
}
