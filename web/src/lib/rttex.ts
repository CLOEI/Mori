import * as pako from "pako";
import { ExtendBuffer, getLowestPowerOf2 } from "./utils";

export interface MipMap {
  width: number;
  height: number;
  bufferLength: number;
  count: number;
}

export interface RTPACK {
  type: string;
  version: number;
  reserved: number;
  compressedSize: number;
  decompressedSize: number;
  compressionType: number;
  reserved2: Int8Array;
}

export interface RTTXTR {
  type: string;
  version: number;
  reserved: number;
  width: number;
  height: number;
  format: number;
  originalWidth: number;
  originalHeight: number;
  isAlpha: number;
  isCompressed: number;
  reservedFlags: number;
  mipmap: MipMap;
  reserved2: Int32Array;
}

function arrayToString(arr: number[] | Uint8Array): string {
  return String.fromCharCode(...Array.from(arr));
}

function stringToArray(str: string): number[] {
  return str.split("").map((c) => c.charCodeAt(0));
}

export class RTTEX {
  public image: ExtendBuffer;
  public type: string | undefined;

  constructor(image: number[] | Uint8Array | ExtendBuffer) {
    const buffer =
      image instanceof ExtendBuffer
        ? image
        : new ExtendBuffer(Array.from(image));

    const header = arrayToString(buffer.data.slice(0, 6));

    if (header !== "RTPACK" && header !== "RTTXTR") {
      throw new Error("File header must be a RTPACK or RTTXTR");
    }

    this.image = buffer;
    this.type = header;
  }

  public parseRTPACK(): RTPACK {
    if (this.type !== "RTPACK") throw new TypeError("Invalid type of RTPACK");

    this.image.mempos = 0;
    const type = arrayToString(this.image.data.slice(0, 6));
    this.image.mempos = 6;

    const data: RTPACK = {
      type,
      version: this.image.readU8(),
      reserved: this.image.readU8(),
      compressedSize: this.image.readU32(),
      decompressedSize: this.image.readU32(),
      compressionType: this.image.readU8(),
      reserved2: new Int8Array(16),
    };

    for (let i = 0; i < 15; i++) {
      data.reserved2[i] = this.image.readI8();
    }

    return data;
  }

  public parseRTTXTR(): RTTXTR {
    let imgData = this.image.data;

    if (this.type === "RTPACK") {
      const compressed = imgData.slice(32);
      imgData = Array.from(pako.inflate(new Uint8Array(compressed)));
    }

    const img = new ExtendBuffer(imgData);
    const header = arrayToString(img.data.slice(0, 6));

    if (header !== "RTTXTR") {
      throw new TypeError("Invalid type of RTTXTR");
    }

    img.mempos = 6;
    const data: RTTXTR = {
      type: header,
      version: img.readU8(),
      reserved: img.readU8(),
      width: img.readI32(),
      height: img.readI32(),
      format: img.readI32(),
      originalWidth: img.readI32(),
      originalHeight: img.readI32(),
      isAlpha: img.readU8(),
      isCompressed: img.readU8(),
      reservedFlags: img.readU16(),
      mipmap: {
        count: 0,
        width: 0,
        height: 0,
        bufferLength: 0,
      },
      reserved2: new Int32Array(16),
    };

    data.mipmap.count = img.readI32();

    for (let i = 0; i < 16; i++) {
      data.reserved2[i] = img.readI32();
    }

    data.mipmap.width = img.readI32();
    data.mipmap.height = img.readI32();
    data.mipmap.bufferLength = img.readI32();

    return data;
  }

  public static async decode(
    rttexImg: number[] | Uint8Array,
  ): Promise<Uint8Array> {
    let data = Array.from(rttexImg);

    const header = arrayToString(data.slice(0, 6));

    if (header === "RTPACK") {
      data = Array.from(pako.inflate(new Uint8Array(data.slice(32))));
    }

    const finalHeader = arrayToString(data.slice(0, 6));

    if (finalHeader === "RTTXTR") {
      const buf = new ExtendBuffer(data);
      buf.mempos = 8;
      const width = buf.readI32();
      const height = buf.readI32();
      const pixelData = new Uint8Array(data.slice(124));

      return await flipVerticalAndEncodePNG(width, height, pixelData);
    } else {
      throw new Error("Invalid format type.");
    }
  }

  public static async encode(img: Uint8Array): Promise<Uint8Array> {
    const header = arrayToString(Array.from(img.slice(0, 6)));

    if (header === "RTPACK" || header === "RTTXTR") {
      throw new TypeError("Invalid format, must be a PNG");
    }

    const { width, height, pixelData } = await decodePNGAndFlipVertical(img);

    const rttex = new ExtendBuffer(124);
    rttex.mempos = 0;

    stringToArray("RTTXTR").forEach((c) => rttex.writeU8(c));
    rttex.writeU8(0); // version
    rttex.writeU8(0); // reserved

    rttex.writeI32(getLowestPowerOf2(height)); // width
    rttex.writeI32(getLowestPowerOf2(width)); // height
    rttex.writeI32(5121); // format
    rttex.writeI32(height); // originalWidth
    rttex.writeI32(width); // originalHeight

    rttex.writeU8(1); // isAlpha
    rttex.writeU8(0); // isCompressed
    rttex.writeU16(1); // reservedFlags
    rttex.writeI32(1); // mipmapCount

    // reserved (16)
    for (let i = 0; i < 16; i++) {
      rttex.writeI32(0);
    }

    rttex.writeI32(height); // mipmapHeight
    rttex.writeI32(width); // mipmapWidth
    rttex.writeI32(pixelData.length); // bufferLength

    const combined = [...rttex.data, ...Array.from(pixelData)];
    const compressed = pako.deflate(new Uint8Array(combined));

    const rtpack = new ExtendBuffer(32);
    rtpack.mempos = 0;

    stringToArray("RTPACK").forEach((c) => rtpack.writeU8(c));
    rtpack.writeU8(1); // version
    rtpack.writeU8(1); // reserved

    rtpack.writeU32(compressed.length); // compressedSize
    rtpack.writeU32(124 + pixelData.length); // decompressedSize
    rtpack.writeU8(1); // compressionType

    // reserved (15)
    for (let i = 0; i < 15; i++) {
      rtpack.writeU8(0);
    }

    return new Uint8Array([...rtpack.data, ...Array.from(compressed)]);
  }
}

async function flipVerticalAndEncodePNG(
  width: number,
  height: number,
  pixelData: Uint8Array,
): Promise<Uint8Array> {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");

  if (!ctx) throw new Error("Could not get canvas context");

  const imageData = ctx.createImageData(width, height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const srcIdx = ((height - 1 - y) * width + x) * 4;
      const dstIdx = (y * width + x) * 4;

      imageData.data[dstIdx] = pixelData[srcIdx];
      imageData.data[dstIdx + 1] = pixelData[srcIdx + 1];
      imageData.data[dstIdx + 2] = pixelData[srcIdx + 2];
      imageData.data[dstIdx + 3] = pixelData[srcIdx + 3];
    }
  }

  ctx.putImageData(imageData, 0, 0);

  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (!blob) {
        reject(new Error("Failed to encode PNG"));
        return;
      }
      blob
        .arrayBuffer()
        .then((buffer) => {
          resolve(new Uint8Array(buffer));
        })
        .catch(reject);
    }, "image/png");
  });
}

async function decodePNGAndFlipVertical(
  pngData: Uint8Array,
): Promise<{ width: number; height: number; pixelData: Uint8Array }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    // @ts-expect-error something is wrong with the types, but it works
    const blob = new Blob([pngData], { type: "image/png" });
    const url = URL.createObjectURL(blob);

    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext("2d");

      if (!ctx) {
        reject(new Error("Could not get canvas context"));
        return;
      }

      ctx.drawImage(img, 0, 0);
      const imageData = ctx.getImageData(0, 0, img.width, img.height);

      const flipped = new Uint8Array(imageData.data.length);
      for (let y = 0; y < img.height; y++) {
        for (let x = 0; x < img.width; x++) {
          const srcIdx = (y * img.width + x) * 4;
          const dstIdx = ((img.height - 1 - y) * img.width + x) * 4;

          flipped[dstIdx] = imageData.data[srcIdx];
          flipped[dstIdx + 1] = imageData.data[srcIdx + 1];
          flipped[dstIdx + 2] = imageData.data[srcIdx + 2];
          flipped[dstIdx + 3] = imageData.data[srcIdx + 3];
        }
      }

      URL.revokeObjectURL(url);
      resolve({
        width: img.width,
        height: img.height,
        pixelData: flipped,
      });
    };

    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error("Failed to decode PNG"));
    };

    img.src = url;
  });
}
