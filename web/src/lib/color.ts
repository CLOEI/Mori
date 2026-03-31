export class Color {
  private m_color: [number, number, number, number] = [0xFF, 0xFF, 0xFF, 0xFF];

  constructor(r: number, g: number, b: number, a?: number);
  constructor(col: number);
  constructor(rOrCol: number, g?: number, b?: number, a: number = 255) {
    if (g !== undefined && b !== undefined) {
      this.m_color[0] = b;
      this.m_color[1] = g;
      this.m_color[2] = rOrCol;
      this.m_color[3] = a;
    } else {
      const col = rOrCol;
      this.m_color[0] = (col >> 24) & 0xFF;
      this.m_color[1] = (col >> 16) & 0xFF;
      this.m_color[2] = (col >> 8) & 0xFF;
      this.m_color[3] = col & 0xFF;
    }
  }

  getUint(): number {
    let result = 0;
    for (let index = 0; index < 4; index++) {
      result = (result << 8) + this.m_color[index];
    }
    return result >>> 0;
  }

  setRed(col: number): void { this.m_color[2] = col; }
  getRed(): number { return this.m_color[2]; }
  setGreen(col: number): void { this.m_color[1] = col; }
  getGreen(): number { return this.m_color[1]; }
  setBlue(col: number): void { this.m_color[0] = col; }
  getBlue(): number { return this.m_color[0]; }
  setAlpha(col: number): void { this.m_color[3] = col; }
  getAlpha(): number { return this.m_color[3]; }

  toRGBAString(): string {
    return `rgba(${this.getRed()}, ${this.getGreen()}, ${this.getBlue()}, ${this.getAlpha() / 255})`;
  }

  toHex(): string {
    const r = this.getRed().toString(16).padStart(2, '0');
    const g = this.getGreen().toString(16).padStart(2, '0');
    const b = this.getBlue().toString(16).padStart(2, '0');
    const a = this.getAlpha().toString(16).padStart(2, '0');
    return `#${r}${g}${b}${a}`;
  }
}
