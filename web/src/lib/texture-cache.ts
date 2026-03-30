import { external_api } from "@/lib/api";
import { RTTEX } from "@/lib/rttex";
import { cropTileFromAtlas } from "@/lib/utils";

interface TextureEntry {
  blob: Blob;
  objectUrl: string;
  refCount: number;
}

interface TileEntry {
  objectUrl: string;
  refCount: number;
}

class TextureCacheManager {
  private loadingTextures = new Map<string, Promise<TextureEntry>>();
  private textures = new Map<string, TextureEntry>();
  private tiles = new Map<string, TileEntry>();
  private cacheApi: Cache | null = null;

  private async getCacheApi(): Promise<Cache> {
    if (!this.cacheApi) {
      this.cacheApi = await caches.open("texture-cache");
    }
    return this.cacheApi;
  }

  private async getTexture(textureFileName: string): Promise<TextureEntry> {
    const cached = this.textures.get(textureFileName);
    if (cached) {
      return cached;
    }

    const loading = this.loadingTextures.get(textureFileName);
    if (loading) {
      return loading;
    }

    const loadPromise = this._loadTexture(textureFileName);
    this.loadingTextures.set(textureFileName, loadPromise);

    try {
      const entry = await loadPromise;
      this.textures.set(textureFileName, entry);
      return entry;
    } finally {
      this.loadingTextures.delete(textureFileName);
    }
  }

  private async _loadTexture(textureFileName: string): Promise<TextureEntry> {
    const cache = await this.getCacheApi();
    const cacheKey = `https://texture-cache.local/${textureFileName}`;
    const cachedResponse = await cache.match(cacheKey);

    let pngBlob: Blob;

    if (cachedResponse) {
      pngBlob = await cachedResponse.blob();
    } else {
      const rttexBlob = await external_api.stiledevs.getGameTextureAssets(
        textureFileName
      );

      const arrayBuffer = await rttexBlob.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      const pngData = await RTTEX.decode(uint8Array);
      // @ts-expect-error something is wrong with the types, but it works
      pngBlob = new Blob([pngData], { type: "image/png" });

      await cache.put(cacheKey, new Response(pngBlob.slice()));
    }

    const objectUrl = URL.createObjectURL(pngBlob);

    return {
      blob: pngBlob,
      objectUrl,
      refCount: 0,
    };
  }

  async getCroppedTile(
    textureFileName: string,
    textureX: number,
    textureY: number
  ): Promise<string> {
    const tileKey = `${textureFileName}:${textureX}:${textureY}`;

    const cached = this.tiles.get(tileKey);
    if (cached) {
      cached.refCount++;
      return cached.objectUrl;
    }

    const textureEntry = await this.getTexture(textureFileName);
    textureEntry.refCount++;

    const croppedUrl = await cropTileFromAtlas(
      textureEntry.objectUrl,
      textureX,
      textureY
    );

    const tileEntry: TileEntry = {
      objectUrl: croppedUrl,
      refCount: 1,
    };
    this.tiles.set(tileKey, tileEntry);

    return croppedUrl;
  }

  releaseTile(
    textureFileName: string,
    textureX: number,
    textureY: number
  ): void {
    const tileKey = `${textureFileName}:${textureX}:${textureY}`;
    const tileEntry = this.tiles.get(tileKey);

    if (tileEntry) {
      tileEntry.refCount--;
      if (tileEntry.refCount <= 0) {
        URL.revokeObjectURL(tileEntry.objectUrl);
        this.tiles.delete(tileKey);
      }
    }

    const textureEntry = this.textures.get(textureFileName);
    if (textureEntry) {
      textureEntry.refCount--;
      if (textureEntry.refCount <= 0) {
        setTimeout(() => {
          const entry = this.textures.get(textureFileName);
          if (entry && entry.refCount <= 0) {
            URL.revokeObjectURL(entry.objectUrl);
            this.textures.delete(textureFileName);
          }
        }, 10000);
      }
    }
  }

  async clearAll(): Promise<void> {
    for (const entry of this.tiles.values()) {
      URL.revokeObjectURL(entry.objectUrl);
    }
    this.tiles.clear();

    for (const entry of this.textures.values()) {
      URL.revokeObjectURL(entry.objectUrl);
    }
    this.textures.clear();
    this.loadingTextures.clear();

    const cache = await this.getCacheApi();
    const keys = await cache.keys();
    await Promise.all(keys.map((key) => cache.delete(key)));
  }
}

export const textureCacheManager = new TextureCacheManager();
