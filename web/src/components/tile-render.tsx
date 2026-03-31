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

        const coords = tileManager.getSpriteCoords(tile, isBackground);
        
        const tileData = tileManager.getTile(tile.x, tile.y);
        if (!tileData) return;

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
