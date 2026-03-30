import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getLowestPowerOf2(n: number) {
  let lowest = 1;
  while (lowest < n) lowest <<= 1;
  return lowest;
}

export function protonSDKHash(chunk: number[]): number {
  let hash = 0x55555555;
  chunk.forEach((x) => (hash = (hash >>> 27) + (hash << 5) + x));
  return hash;
}

export class ExtendBuffer {
  public data: number[];
  public mempos = 0;

  constructor(data: number[] | number) {
    this.data = Array.isArray(data) ? data : new Array(data).fill(0);
  }

  private read(size: number): number {
    let value = 0;
    for (let i = 0; i < size; i++) {
      value |= this.data[this.mempos + i] << (i * 8);
    }
    this.mempos += size;
    return value >>> 0;
  }

  private readSigned(size: number): number {
    return this.read(size) << 0;
  }

  private write(value: number, size: number): void {
    for (let i = 0; i < size; i++) {
      this.data[this.mempos + i] = (value >> (i * 8)) & 0xff;
    }
    this.mempos += size;
  }

  public readU8 = () => this.read(1);
  public readU16 = (be = false) => (be ? this.readBE(2) : this.read(2));
  public readU32 = (be = false) => (be ? this.readBE(4) : this.read(4));

  public readI8 = () => this.readSigned(1);
  public readI16 = (be = false) =>
    be ? this.readSignedBE(2) : this.readSigned(2);
  public readI32 = (be = false) =>
    be ? this.readSignedBE(4) : this.readSigned(4);

  private readBE(size: number): number {
    let value = 0;
    for (let i = 0; i < size; i++) {
      value = (value << 8) | this.data[this.mempos + i];
    }
    this.mempos += size;
    return value >>> 0;
  }

  private readSignedBE(size: number): number {
    return this.readBE(size) << 0;
  }

  public writeU8 = (value: number) => this.write(value, 1);
  public writeU16 = (value: number, be = false) =>
    be ? this.writeBE(value, 2) : this.write(value, 2);
  public writeU32 = (value: number, be = false) =>
    be ? this.writeBE(value, 4) : this.write(value, 4);

  public writeI8 = (value: number) => this.write(value, 1);
  public writeI16 = (value: number, be = false) =>
    be ? this.writeBE(value, 2) : this.write(value, 2);
  public writeI32 = (value: number, be = false) =>
    be ? this.writeBE(value, 4) : this.write(value, 4);

  private writeBE(value: number, size: number): void {
    for (let i = 0; i < size; i++) {
      this.data[this.mempos + i] = (value >> ((size - 1 - i) * 8)) & 0xff;
    }
    this.mempos += size;
  }

  public writeU = (size: number, value: number, be = false) => {
    const methods = { 1: this.writeU8, 2: this.writeU16, 4: this.writeU32 };
    methods[size as 1 | 2 | 4](value, be);
  };

  public writeI = (size: number, value: number, be = false) => {
    const methods = { 1: this.writeI8, 2: this.writeI16, 4: this.writeI32 };
    methods[size as 1 | 2 | 4](value, be);
  };

  public async readString(be = false) {
    const len = be ? this.readBE(2) : this.read(2);
    const chars = this.data.slice(this.mempos, this.mempos + len);
    this.mempos += len;
    return String.fromCharCode(...chars);
  }

  public async writeString(str: string, be = false) {
    const bytes = str.split("").map((char) => char.charCodeAt(0));
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    be ? this.writeBE(str.length, 2) : this.write(str.length, 2);
    for (const byte of bytes) {
      this.data[this.mempos++] = byte;
    }
  }
}

const TEXTURE_ATLAS_SIZE = 1024;
const TILE_SIZE = 32;
const TILES_PER_ROW = TEXTURE_ATLAS_SIZE / TILE_SIZE; // 32 tiles

/**
 * Crops a specific tile from the texture atlas and returns it as a data URL
 * @param imageSrc - The source image (can be a URL or data URL)
 * @param tileX - X coordinate in the tile grid (0-31)
 * @param tileY - Y coordinate in the tile grid (0-31)
 * @returns Promise<string> - Data URL of the cropped tile
 */
export async function cropTileFromAtlas(
  imageSrc: string,
  tileX: number,
  tileY: number,
): Promise<string> {
  if (tileX < 0 || tileX >= TILES_PER_ROW || tileY < 0 || tileY >= TILES_PER_ROW) {
    throw new Error(
      `Tile coordinates out of bounds: (${tileX}, ${tileY}). Valid range: 0-${TILES_PER_ROW - 1}`,
    );
  }

  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = "anonymous";

    img.onload = () => {
      try {
        const canvas = document.createElement("canvas");
        canvas.width = TILE_SIZE;
        canvas.height = TILE_SIZE;
        const ctx = canvas.getContext("2d");

        if (!ctx) {
          reject(new Error("Could not get canvas context"));
          return;
        }

        // Calculate pixel coordinates
        const sourceX = tileX * TILE_SIZE;
        const sourceY = tileY * TILE_SIZE;

        // Draw the specific tile onto the canvas
        ctx.drawImage(
          img,
          sourceX,
          sourceY,
          TILE_SIZE,
          TILE_SIZE,
          0,
          0,
          TILE_SIZE,
          TILE_SIZE,
        );

        // Convert to data URL
        const dataUrl = canvas.toDataURL("image/png");
        resolve(dataUrl);
      } catch (error) {
        reject(error);
      }
    };

    img.onerror = () => {
      reject(new Error("Failed to load image"));
    };

    img.src = imageSrc;
  });
}

// The 47 fundamental visual states. 
// These are the core textures where all diagonals legally connect to their neighbors.
const CORE_TEXTURE_STATES: Record<number, number> = {
    0: 12, 2: 11, 8: 29, 10: 43, 14: 7, 32: 10, 34: 9, 40: 45, 
    42: 33, 46: 32, 56: 5, 58: 31, 62: 3, 128: 30, 130: 44, 131: 8, 
    136: 28, 138: 42, 139: 41, 142: 40, 143: 2, 160: 28, 162: 36, 163: 35, 
    168: 39, 170: 27, 171: 23, 174: 24, 175: 18, 184: 37, 186: 26, 187: 22, 
    190: 19, 191: 15, 224: 28, 226: 34, 227: 4, 232: 38, 234: 25, 235: 20, 
    238: 21, 239: 16, 248: 1, 250: 17, 251: 14, 254: 13, 255: 0
};

export function generate8BitLUT() {
    const lut = new Uint8Array(256);

    for (let i = 0; i < 256; i++) {
        let bit = i;

        // THE DIAGONAL FILTER
        // Bits: TL=1, T=2, TR=4, R=8, BR=16, B=32, BL=64, L=128
        // If a diagonal exists, but the adjacent flat sides do not, we strip the diagonal bit out.
        
        if (!(bit & 2) || !(bit & 128)) bit &= ~1;   // Top-Left (1) requires Top (2) & Left (128)
        if (!(bit & 2) || !(bit & 8))   bit &= ~4;   // Top-Right (4) requires Top (2) & Right (8)
        if (!(bit & 32) || !(bit & 8))  bit &= ~16;  // Bot-Right (16) requires Bot (32) & Right (8)
        if (!(bit & 32) || !(bit & 128)) bit &= ~64; // Bot-Left (64) requires Bot (32) & Left (128)

        // Map the filtered mask to the core state (Fallback to 12 / isolated block for safety)
        lut[i] = CORE_TEXTURE_STATES[bit] !== undefined ? CORE_TEXTURE_STATES[bit] : 12;
    }

    return lut;
}