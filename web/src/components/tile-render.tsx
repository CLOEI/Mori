import { api, type ItemRecord } from "@/lib/api";
import { useEffect, useState } from "react";
import { textureCacheManager } from "@/lib/texture-cache";
import { TileManager, type TileData } from "@/lib/tile-manager";

interface TileRenderProps {
  tile: TileData;
  tileManager: TileManager;
  isBackground?: boolean;
  className?: string;
}

export function TileRender({ 
  tile, 
  tileManager, 
  isBackground = false,
  className = ""
}: TileRenderProps) {
  const [src, setSrc] = useState<string | null>(null);
  const [item, setItem] = useState<ItemRecord | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function loadTileTexture() {
      try {
        const itemId = isBackground ? tile.bgItemId : tile.fgItemId;
        
        if (itemId === 0) {
          setSrc(null);
          return;
        }

        // Get sprite coordinates from tile manager
        const coords = tileManager.getSpriteCoords(tile, isBackground);
        
        // Get the item data to know which texture file to use
        const tileData = tileManager.getTile(tile.x, tile.y);
        if (!tileData) return;

        // You'll need to get the item from somewhere - perhaps pass it or have TileManager expose it
        // For now, we'll use a simple approach
        const itemsData = await api.getItemsByIds([itemId]);
        const itemRecord = itemsData[0];
        
        if (!itemRecord || cancelled) return;
        
        setItem(itemRecord);

        const croppedUrl = await textureCacheManager.getCroppedTile(
          itemRecord.texture_file_name,
          coords.x,
          coords.y
        );

        if (!cancelled) {
          setSrc(croppedUrl);
        }
      } catch (error) {
        console.error("Failed to load tile texture:", error);
      }
    }

    loadTileTexture();
    
    return () => {
      cancelled = true;
      if (item) {
        const coords = tileManager.getSpriteCoords(tile, isBackground);
        textureCacheManager.releaseTile(
          item.texture_file_name,
          coords.x,
          coords.y
        );
      }
    };
  }, [tile, tileManager, isBackground]);

  if (!src) {
    return null;
  }

  return (
    <img
      src={src}
      alt={item?.name || "tile"}
      className={className}
      style={{ imageRendering: "pixelated" }}
    />
  );
}
