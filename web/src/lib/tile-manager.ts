import { type ItemRecord } from "./api";
import { TILE_RENDER_TYPE, LUT_4BIT, LUT_8BIT, TILE_FLAGS } from "./constants";

export interface TileData {
  fgItemId: number;
  bgItemId: number;
  flags: number;
  x: number;
  y: number;
}

export interface WorldData {
  width: number;
  height: number;
  tiles: TileData[];
}

export interface SpriteCoords {
  x: number;
  y: number;
}

export class TileManager {
  private worldData: WorldData;
  private itemsDatabase: Map<number, ItemRecord>;

  constructor(worldData: WorldData, items: ItemRecord[]) {
    this.worldData = worldData;
    this.itemsDatabase = new Map(items.map((item) => [item.id, item]));
  }

  public getTile(x: number, y: number): TileData | null {
    if (x < 0 || y < 0 || x >= this.worldData.width || y >= this.worldData.height) {
      return null;
    }
    const index = y * this.worldData.width + x;
    return this.worldData.tiles[index] || null;
  }

  private isSameTile(sourceTile: TileData, targetX: number, targetY: number, isBackground: boolean): boolean {
    const targetTile = this.getTile(targetX, targetY);
    if (!targetTile) return false;

    const sourceItemId = isBackground ? sourceTile.bgItemId : sourceTile.fgItemId;
    const targetItemId = isBackground ? targetTile.bgItemId : targetTile.fgItemId;

    return sourceItemId === targetItemId && sourceItemId !== 0;
  }

  public getSpriteCoords(tile: TileData, isBackground: boolean): SpriteCoords {
    const itemId = isBackground ? tile.bgItemId : tile.fgItemId;
    const item = this.itemsDatabase.get(itemId);

    if (!item || itemId === 0) {
      return { x: 0, y: 0 };
    }

    let coordX = item.texture_x;
    let coordY = item.texture_y;

    const tileX = tile.x;
    const tileY = tile.y;

    switch (item.render_type) {
      case TILE_RENDER_TYPE.FILLER:
      case TILE_RENDER_TYPE.SINGLE:
        break;

      case TILE_RENDER_TYPE.DIRECT8: {
        const topLeft = (tileX > 0 && tileY > 0) 
          ? this.isSameTile(tile, tileX - 1, tileY - 1, isBackground) : false;
        const top = (tileY > 0) 
          ? this.isSameTile(tile, tileX, tileY - 1, isBackground) : false;
        const topRight = (tileX < this.worldData.width - 1 && tileY > 0) 
          ? this.isSameTile(tile, tileX + 1, tileY - 1, isBackground) : false;
        const right = (tileX < this.worldData.width - 1) 
          ? this.isSameTile(tile, tileX + 1, tileY, isBackground) : false;
        const bottomRight = (tileX < this.worldData.width - 1 && tileY < this.worldData.height - 1) 
          ? this.isSameTile(tile, tileX + 1, tileY + 1, isBackground) : false;
        const bottom = (tileY < this.worldData.height - 1) 
          ? this.isSameTile(tile, tileX, tileY + 1, isBackground) : false;
        const bottomLeft = (tileX > 0 && tileY < this.worldData.height - 1) 
          ? this.isSameTile(tile, tileX - 1, tileY + 1, isBackground) : false;
        const left = (tileX > 0) 
          ? this.isSameTile(tile, tileX - 1, tileY, isBackground) : false;

        const mask = 
          (topLeft ? 1 << 0 : 0) |
          (top ? 1 << 1 : 0) |
          (topRight ? 1 << 2 : 0) |
          (right ? 1 << 3 : 0) |
          (bottomRight ? 1 << 4 : 0) |
          (bottom ? 1 << 5 : 0) |
          (bottomLeft ? 1 << 6 : 0) |
          (left ? 1 << 7 : 0);

        const lutVisual = LUT_8BIT[mask];
        const offsetX = lutVisual % 8;
        const offsetY = Math.floor(lutVisual / 8);
        
        if (coordX + offsetX < 32 && coordY + offsetY < 32) {
          coordX += offsetX;
          coordY += offsetY;
        }
        break;
      }

      case TILE_RENDER_TYPE.HORIZONTAL: {
        const right = this.isSameTile(tile, tileX + 1, tileY, isBackground);
        const left = this.isSameTile(tile, tileX - 1, tileY, isBackground);

        let offset = 0;
        if (right) {
          offset = left ? 1 : 0;
        } else {
          offset = left ? 2 : 3;
        }
        if (coordX + offset < 32) {
          coordX += offset;
        }
        break;
      }

      case TILE_RENDER_TYPE.ATTACH_TO_WALL_5: {
        // This would require checking for solid tiles below
        break;
      }

      case TILE_RENDER_TYPE.DIRECT4: {
        const top = (tileY > 0) 
          ? this.isSameTile(tile, tileX, tileY - 1, isBackground) : false;
        const left = (tileX > 0) 
          ? this.isSameTile(tile, tileX - 1, tileY, isBackground) : false;
        const right = (tileX < this.worldData.width - 1) 
          ? this.isSameTile(tile, tileX + 1, tileY, isBackground) : false;
        const bottom = (tileY < this.worldData.height - 1) 
          ? this.isSameTile(tile, tileX, tileY + 1, isBackground) : false;

        const mask = 
          (top ? 1 << 0 : 0) |
          (left ? 1 << 1 : 0) |
          (right ? 1 << 2 : 0) |
          (bottom ? 1 << 3 : 0);

        const lutVisual = LUT_4BIT[mask];
        const offsetX = lutVisual % 8;
        const offsetY = Math.floor(lutVisual / 8);
        
        // Validate bounds
        if (coordX + offsetX < 32 && coordY + offsetY < 32) {
          coordX += offsetX;
          coordY += offsetY;
        }
        break;
      }

      case TILE_RENDER_TYPE.RANDOM: {
        const random = this.pseudoRandom(tileX, tileY);
        const offset = random % 4;
        if (coordX + offset < 32) {
          coordX += offset;
        }
        break;
      }

      case TILE_RENDER_TYPE.VERTICAL: {
        const top = this.isSameTile(tile, tileX, tileY - 1, isBackground);
        const bottom = this.isSameTile(tile, tileX, tileY + 1, isBackground);

        let offset = 0;
        if (top) {
          offset = bottom ? 1 : 2;
        } else {
          offset = bottom ? 0 : 3;
        }
        if (coordX + offset < 32) {
          coordX += offset;
        }
        break;
      }

      case TILE_RENDER_TYPE.CAVE_PLAT: {
        // STORAGE_SMART_EDGE_HORIZ_CAVE - Similar to horizontal but different context
        const right = this.isSameTile(tile, tileX + 1, tileY, isBackground);
        const left = this.isSameTile(tile, tileX - 1, tileY, isBackground);

        let offset = 0;
        if (right) {
          offset = left ? 1 : 0;
        } else {
          offset = left ? 2 : 3;
        }
        if (coordX + offset < 32) {
          coordX += offset;
        }
        break;
      }

      case TILE_RENDER_TYPE.ATTACH_TO_WALL_4: {
        // STORAGE_SMART_CLING2 - Similar to DIRECT4 but for items that attach to walls
        const top = (tileY > 0) 
          ? this.isSameTile(tile, tileX, tileY - 1, isBackground) : false;
        const left = (tileX > 0) 
          ? this.isSameTile(tile, tileX - 1, tileY, isBackground) : false;
        const right = (tileX < this.worldData.width - 1) 
          ? this.isSameTile(tile, tileX + 1, tileY, isBackground) : false;
        const bottom = (tileY < this.worldData.height - 1) 
          ? this.isSameTile(tile, tileX, tileY + 1, isBackground) : false;

        const mask = 
          (top ? 1 << 0 : 0) |
          (left ? 1 << 1 : 0) |
          (right ? 1 << 2 : 0) |
          (bottom ? 1 << 3 : 0);

        const lutVisual = LUT_4BIT[mask];
        const offsetX = lutVisual % 8;
        const offsetY = Math.floor(lutVisual / 8);
        
        // Validate bounds
        if (coordX + offsetX < 32 && coordY + offsetY < 32) {
          coordX += offsetX;
          coordY += offsetY;
        }
        break;
      }

      case TILE_RENDER_TYPE.DIAGONAL: {
        break;
      }

      default:
        break;
    }

    if ((tile.flags & TILE_FLAGS.IS_ON) !== 0) {
      if (coordX + 1 < 32) {
        coordX += 1;
      }
    }

    return { x: coordX, y: coordY };
  }

  private pseudoRandom(x: number, y: number): number {
    const seed = x * 7919 + y * 7919;
    return Math.abs(seed) % 4;
  }

  public updateWorldData(worldData: WorldData): void {
    this.worldData = worldData;
  }

  public updateItems(items: ItemRecord[]): void {
    this.itemsDatabase = new Map(items.map((item) => [item.id, item]));
  }
}
