import { useCallback, useEffect, useRef, useState } from 'react'
import { Application, Graphics, Container, Sprite, Texture } from 'pixi.js'
import { useAtomValue, useSetAtom } from 'jotai'
import { itemNamesAtom, itemColorsAtom, itemsMapAtom, type LiveBot } from '@/lib/store'
import { api, type ItemRecord } from '@/lib/api'
import type { TileData } from '@/lib/ws'
import { TileManager, type WorldData, type TileData as TileManagerData } from '@/lib/tile-manager'
import { textureCacheManager } from '@/lib/texture-cache'
import { Progress } from '@/components/ui/progress'
import { Color } from '@/lib/color'

const TILE_FLAGS: [number, string][] = [
  [0x0002, 'has_parent'], [0x0004, 'spliced'], [0x0008, 'spawns_seeds'],
  [0x0010, 'seedling'], [0x0020, 'flipped_x'], [0x0040, 'on'],
  [0x0080, 'public'], [0x0100, 'bg_on'], [0x0200, 'alt_mode'],
  [0x0400, 'wet'], [0x0800, 'glued'], [0x1000, 'on_fire'],
  [0x2000, 'painted_red'], [0x4000, 'painted_green'], [0x8000, 'painted_blue'],
]

async function createCompositeTreeTexture(
  baseUrl: string,
  overlayUrl: string,
  baseColor?: Color,
  overlayColor?: Color
): Promise<HTMLCanvasElement> {
  return new Promise((resolve, reject) => {
    const canvas = document.createElement('canvas');
    canvas.width = 32;
    canvas.height = 32;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });

    if (!ctx) {
      reject(new Error('Could not get canvas context'));
      return;
    }

    const baseImg = new Image();
    const overlayImg = new Image();

    let baseLoaded = false;
    let overlayLoaded = false;

    const tryRender = () => {
      if (!baseLoaded || !overlayLoaded) return;

      ctx.clearRect(0, 0, 32, 32);

      const tempCanvas = document.createElement('canvas');
      tempCanvas.width = 32;
      tempCanvas.height = 32;
      const tempCtx = tempCanvas.getContext('2d', { willReadFrequently: true });

      if (!tempCtx) {
        reject(new Error('Could not get temp canvas context'));
        return;
      }

      tempCtx.drawImage(baseImg, 0, 0, 32, 32);
      if (baseColor) {
        const imageData = tempCtx.getImageData(0, 0, 32, 32);
        const data = imageData.data;
        const r = baseColor.getRed() / 255;
        const g = baseColor.getGreen() / 255;
        const b = baseColor.getBlue() / 255;

        for (let i = 0; i < data.length; i += 4) {
          if (data[i + 3] > 0) {
            data[i] = Math.floor(data[i] * r);
            data[i + 1] = Math.floor(data[i + 1] * g);
            data[i + 2] = Math.floor(data[i + 2] * b);
          }
        }
        tempCtx.putImageData(imageData, 0, 0);
      }
      ctx.drawImage(tempCanvas, 0, 0);

      // Apply overlay with overlay color
      tempCtx.clearRect(0, 0, 32, 32);
      tempCtx.drawImage(overlayImg, 0, 0, 32, 32);
      
      if (overlayColor && overlayColor.getUint() !== 0xFFFFFFFF) {
        const overlayData = tempCtx.getImageData(0, 0, 32, 32);
        const data = overlayData.data;
        const r = overlayColor.getRed() / 255;
        const g = overlayColor.getGreen() / 255;
        const b = overlayColor.getBlue() / 255;

        for (let i = 0; i < data.length; i += 4) {
          if (data[i + 3] > 0) {
            data[i] = Math.floor(data[i] * r);
            data[i + 1] = Math.floor(data[i + 1] * g);
            data[i + 2] = Math.floor(data[i + 2] * b);
          }
        }
        tempCtx.putImageData(overlayData, 0, 0);
      }
      
      ctx.drawImage(tempCanvas, 0, 0);

      resolve(canvas);
    };

    baseImg.onload = () => {
      baseLoaded = true;
      tryRender();
    };
    baseImg.onerror = () => reject(new Error('Failed to load base image'));

    overlayImg.onload = () => {
      overlayLoaded = true;
      tryRender();
    };
    overlayImg.onerror = () => reject(new Error('Failed to load overlay image'));

    baseImg.src = baseUrl;
    overlayImg.src = overlayUrl;
  });
}


function activeFlags(flags: number): string[] {
  return TILE_FLAGS.filter(([bit]) => flags & bit).map(([, name]) => name)
}

type TileTypeData = { type: string } & Record<string, unknown>

function tileTypeExtra(tt: TileTypeData, itemNames: Record<string, string>): string | null {
  const name = (id: unknown) => itemNames[String(id)] ?? `ID:${id}`
  switch (tt.type) {
    case 'Sign':     return `Sign: "${tt.text}"`
    case 'Door':     return `Door: "${tt.text}"${tt.owner_uid ? ` owner:${tt.owner_uid}` : ''}`
    case 'Lock':     return `Lock owner:${tt.owner_uid} access:${tt.access_count}${tt.minimum_level ? ` minlvl:${tt.minimum_level}` : ''}`
    case 'Seed':     return `Seed: ${Math.floor((tt.time_passed as number) / 60)}m grown`
    case 'VendingMachine': return `Vend: ${name(tt.item_id)} price:${tt.price}`
    case 'DisplayBlock':   return `Displays: ${name(tt.item_id)}`
    case 'Mannequin': return `Mannequin: "${tt.text}"`
    case 'HearthMonitor': return `Hearth: ${tt.player_name}`
    case 'SilkWorm': return `Silkworm: ${tt.name} age:${tt.age}`
    case 'CountryFlag': return `Flag: ${tt.country}`
    case 'AudioRack': return `Audio: ${tt.note} vol:${tt.volume}`
    case 'TesseractManipulator': return `Tesseract: ${name(tt.item_id)} gems:${tt.gems} ${tt.enabled ? 'on' : 'off'}`
    case 'Dice': return `Dice: ${tt.symbol}`
    case 'Forge': return `Forge: ${tt.temperature}°`
    case 'CookingOven': return `Oven: ${tt.temperature_level}°`
    case 'StorageBlock': return `Storage: ${(tt.items as unknown[])?.length ?? 0} types`
    case 'WeatherMachine': return `Weather settings:${tt.settings}`
    case 'Mailbox': case 'Bulletin': case 'DonationBox': case 'BigLock':
      return tt.s1 ? `"${tt.s1}"` : tt.type
    case 'Basic': return null
    default: return `Type: ${tt.type}`
  }
}

interface TooltipState {
  screenX: number
  screenY: number
  tileX: number
  tileY: number
  tile: TileData
}

const TILE_PX = 4
const CHAR_PX = 2
const MIN_ZOOM = 0.5
const MAX_ZOOM = 20


function clampOffset(
  ox: number,
  oy: number,
  worldW: number,
  worldH: number,
  screenW: number,
  screenH: number,
): { x: number; y: number } {
  // If the world is smaller than the screen, centre it; otherwise clamp
  const x = worldW <= screenW
    ? (screenW - worldW) / 2
    : Math.min(0, Math.max(screenW - worldW, ox))
  const y = worldH <= screenH
    ? (screenH - worldH) / 2
    : Math.min(0, Math.max(screenH - worldH, oy))
  return { x, y }
}

export function Minimap({ bot }: { bot: LiveBot }) {
  const containerRef = useRef<HTMLDivElement>(null)
  const itemNames = useAtomValue(itemNamesAtom)
  const itemColors = useAtomValue(itemColorsAtom)
  const itemsMap = useAtomValue(itemsMapAtom)
  const setItemsMap = useSetAtom(itemsMapAtom)
  const [tooltip, setTooltip] = useState<TooltipState | null>(null)
  const [items, setItems] = useState<ItemRecord[]>([])
  const [useTileManager, setUseTileManager] = useState(false)
  const [loadingProgress, setLoadingProgress] = useState<{ current: number; total: number; stage: 'loading' | 'decoding' } | null>(null)
  const [currentZoom, setCurrentZoom] = useState(3)
  const appRef = useRef<Application | null>(null)
  const layersRef = useRef<{
    tiles: Container
    objects: Graphics
    players: Graphics
    bot: Graphics
  } | null>(null)
  const tileManagerRef = useRef<TileManager | null>(null)

  const botRef = useRef(bot)
  const itemColorsRef = useRef(itemColors)
  const itemsRef = useRef(items)

  // Update refs when props change
  useEffect(() => {
    botRef.current = bot
    itemColorsRef.current = itemColors
    itemsRef.current = items
  }, [bot, itemColors, items])

  // Load items data - only fetch items used in current world
  useEffect(() => {
    if (!bot || bot.tiles.length === 0) {
      setItems([])
      return
    }

    async function loadWorldItems() {
      try {
        // Collect unique item IDs from world tiles
        const uniqueItemIds = new Set<number>()
        for (const tile of bot.tiles) {
          if (tile.fg !== 0) {
            uniqueItemIds.add(tile.fg)
            if (tile.fg % 2 === 1) {
              uniqueItemIds.add(tile.fg + 1)
            }
          }
          if (tile.bg !== 0) uniqueItemIds.add(tile.bg)
        }
        
        const missingIds: number[] = []
        const cachedItems: ItemRecord[] = []
        
        for (const id of uniqueItemIds) {
          const cached = itemsMap.get(id)
          if (cached) {
            cachedItems.push(cached)
          } else {
            missingIds.push(id)
          }
        }
        
        const newItems: ItemRecord[] = []
        if (missingIds.length > 0) {
          try {
            const items = await api.getItemsByIds(missingIds)
            newItems.push(...items)
          } catch (err) {
          }
          
          // Update cache with new items
          if (newItems.length > 0) {
            setItemsMap(prev => {
              const updated = new Map(prev)
              for (const item of newItems) {
                updated.set(item.id, item)
              }
              return updated
            })
          }
        }
        
        const allItems = [...cachedItems, ...newItems]
        setItems(allItems)
        
      } catch (err) {
        setItems([])
      }
    }
    
    loadWorldItems()
  }, [bot, itemsMap, setItemsMap])

  const zoom = useRef(3)
  const offset = useRef({ x: 0, y: 0 })
  const isDragging = useRef(false)
  const dragStart = useRef({ x: 0, y: 0, ox: 0, oy: 0 })

  function renderTilesInitial(container: Container, b: LiveBot, colors: Record<number, number>) {
    const gfx = new Graphics()
    for (let i = 0; i < b.tiles.length; i++) {
      const tile = b.tiles[i]
      if (tile.fg === 0) continue
      const color = colors[tile.fg] ?? 0x4a4a5a
      const tx = (i % b.world_width) * TILE_PX
      const ty = Math.floor(i / b.world_width) * TILE_PX
      gfx.rect(tx, ty, TILE_PX, TILE_PX).fill(color)
    }
    container.addChild(gfx)
  }

  async function renderTilesWithTileManager(container: Container, b: LiveBot, items: ItemRecord[]) {
    
    container.removeChildren()
    
    if (items.length === 0 || b.tiles.length === 0) {
      return
    }

    // Convert to WorldData format
    const tiles: TileManagerData[] = b.tiles.map((tile, i) => ({
      fgItemId: tile.fg,
      bgItemId: tile.bg,
      flags: tile.flags,
      x: i % b.world_width,
      y: Math.floor(i / b.world_width)
    }))

    const worldData: WorldData = {
      width: b.world_width,
      height: b.world_height,
      tiles
    }

    if (!tileManagerRef.current) {
      tileManagerRef.current = new TileManager(worldData, items)
    } else {
      tileManagerRef.current.updateWorldData(worldData)
      tileManagerRef.current.updateItems(items)
    }

    const tileManager = tileManagerRef.current
    const itemMap = new Map(items.map(item => [item.id, item]))

    // Collect all texture requests needed
    const textureRequests: Array<{
      textureFileName: string
      textureX: number
      textureY: number
      x: number
      y: number
      isBg: boolean
      isTree?: boolean
      treeOverlayY?: number
      treeOverlayX?: number
      baseColor?: number
      overlayColor?: number
      item?: ItemRecord
    }> = []

    for (let y = 0; y < b.world_height; y++) {
      for (let x = 0; x < b.world_width; x++) {
        const tile = tileManager.getTile(x, y)
        if (!tile) continue

        // Background
        if (tile.bgItemId !== 0) {
          const item = itemMap.get(tile.bgItemId)
          if (item) {
            const coords = tileManager.getSpriteCoords(tile, true)
            textureRequests.push({
              textureFileName: item.texture_file_name,
              textureX: coords.x,
              textureY: coords.y,
              x,
              y,
              isBg: true
            })
          }
        }

        // Foreground
        if (tile.fgItemId !== 0) {
          const item = itemMap.get(tile.fgItemId)
          if (item) {
            const isSeed = tile.fgItemId % 2 === 1;
            
            if (isSeed) {
              const treeItemId = tile.fgItemId + 1;
              const treeItem = itemMap.get(treeItemId);
              
              if (treeItem) {
                // Convert seed's base_color and overlay_color to Color objects
                const baseColor = item.base_color ? new Color(item.base_color) : new Color(0xFFFFFFFF);
                const overlayColor = item.overlay_color ? new Color(item.overlay_color) : new Color(0xFFFFFFFF);
                
                // Debug: Log complete seed item data (tree sprites are in the seed item, not tree item)
                
                textureRequests.push({
                  textureFileName: "tiles_page1.rttex",
                  textureX: item.tree_base_sprite,
                  textureY: 19,  // Row 19 is the trunk (bottom part)
                  x,
                  y,
                  isBg: false,
                  isTree: true,
                  treeOverlayY: 18,  // Row 18 is the leaves (top part)
                  treeOverlayX: item.tree_overlay_sprite,
                  baseColor: baseColor.getUint(),
                  overlayColor: overlayColor.getUint(),
                  item: treeItem
                })
              } else {
                const coords = tileManager.getSpriteCoords(tile, false)
                textureRequests.push({
                  textureFileName: item.texture_file_name,
                  textureX: coords.x,
                  textureY: coords.y,
                  x,
                  y,
                  isBg: false
                })
              }
            } else {
              const coords = tileManager.getSpriteCoords(tile, false)
              textureRequests.push({
                textureFileName: item.texture_file_name,
                textureX: coords.x,
                textureY: coords.y,
                x,
                y,
                isBg: false
              })
            }
          }
        }
      }
    }

    const CHUNK_SIZE = 200
    const bgContainer = new Container()
    const fgContainer = new Container()
    const textureCache = new Map<string, Texture>()
    
    for (let chunkStart = 0; chunkStart < textureRequests.length; chunkStart += CHUNK_SIZE) {
      const chunk = textureRequests.slice(chunkStart, chunkStart + CHUNK_SIZE)
      
      // Batch load this chunk
      const chunkTextureReqs = chunk.map(req => ({
        textureFileName: req.textureFileName,
        textureX: req.textureX,
        textureY: req.textureY
      }))
      
      const urls = await textureCacheManager.batchLoadTiles(chunkTextureReqs, (loaded) => {
        const overallProgress = chunkStart + loaded
        setLoadingProgress({ 
          current: overallProgress, 
          total: textureRequests.length, 
          stage: 'loading' 
        })
      })
      
      // Convert loaded images to textures and create sprites
      setLoadingProgress({ 
        current: chunkStart, 
        total: textureRequests.length, 
        stage: 'decoding' 
      })
      
      for (let i = 0; i < chunk.length; i++) {
        const req = chunk[i]
        const url = urls[i]
        if (!url) continue
        
        if (req.isTree && req.treeOverlayY !== undefined && req.treeOverlayX !== undefined) {
          try {
            const overlayUrl = await textureCacheManager.getCroppedTile(
              "tiles_page1.rttex",
              req.treeOverlayX,
              req.treeOverlayY
            );

            const baseColor = req.baseColor && req.baseColor !== 0 ? new Color(req.baseColor) : undefined;
            const overlayColor = req.overlayColor && req.overlayColor !== 0 ? new Color(req.overlayColor) : undefined;
            const compositeCanvas = await createCompositeTreeTexture(url, overlayUrl, baseColor, overlayColor);
            
            const texture = Texture.from(compositeCanvas);
            const sprite = new Sprite(texture);
            sprite.x = req.x * TILE_PX;
            sprite.y = req.y * TILE_PX;
            sprite.width = TILE_PX;
            sprite.height = TILE_PX;
            fgContainer.addChild(sprite);
          } catch (e) {
          }
          continue;
        }
        
        const key = `${req.textureFileName}_${req.textureX}_${req.textureY}`
        
        let texture = textureCache.get(key)
        if (!texture) {
          try {
            const img = new Image()
            img.src = url
            await new Promise<void>((resolve, reject) => {
              img.onload = () => resolve()
              img.onerror = reject
              setTimeout(() => reject(new Error('Image load timeout')), 5000)
            })
            
            const canvas = document.createElement('canvas')
            canvas.width = img.width
            canvas.height = img.height
            const ctx = canvas.getContext('2d')
            if (ctx) {
              ctx.drawImage(img, 0, 0)
              texture = Texture.from(canvas)
              textureCache.set(key, texture)
            }
          } catch (e) {
            continue
          }
        }
        
        if (texture) {
          const sprite = new Sprite(texture)
          sprite.x = req.x * TILE_PX
          sprite.y = req.y * TILE_PX
          sprite.width = TILE_PX
          sprite.height = TILE_PX
          
          if (req.isBg) {
            bgContainer.addChild(sprite)
          } else {
            fgContainer.addChild(sprite)
          }
        }
      }
      
      // Yield to main thread between chunks
      if (chunkStart + CHUNK_SIZE < textureRequests.length) {
        await new Promise(resolve => setTimeout(resolve, 0))
      }
    }

    container.addChild(bgContainer)
    container.addChild(fgContainer)
    
    setLoadingProgress(null)
  }

  function getWorldScreenSize() {
    const app = appRef.current
    const b = botRef.current
    if (!app) return null
    return {
      worldW: b.world_width  * TILE_PX * zoom.current,
      worldH: b.world_height * TILE_PX * zoom.current,
      screenW: app.screen.width,
      screenH: app.screen.height,
    }
  }

  function applyTransform() {
    const world = appRef.current?.stage.children[0] as Container | undefined
    if (!world) return
    world.scale.set(zoom.current)
    world.x = offset.current.x
    world.y = offset.current.y
  }

  const centerOnBot = useCallback(() => {
    const app = appRef.current
    const b = botRef.current
    if (!app) return
    const dims = getWorldScreenSize()!
    const raw = {
      x: app.screen.width  / 2 - b.pos_x * TILE_PX * zoom.current,
      y: app.screen.height / 2 - b.pos_y * TILE_PX * zoom.current,
    }
    offset.current = clampOffset(raw.x, raw.y, dims.worldW, dims.worldH, dims.screenW, dims.screenH)
    applyTransform()
  }, [])

  useEffect(() => {
    const el = containerRef.current
    if (!el) return

    let mounted = true
    const app = new Application()

    app.init({
      resizeTo: el,
      backgroundColor: 0x080c10,
      antialias: false,
    }).then(() => {
      if (!mounted) { app.destroy(true, { children: true }); return }

      el.appendChild(app.canvas)
      appRef.current = app

      const world = new Container()
      const tileContainer = new Container()
      const objectGfx = new Graphics()
      const playerGfx = new Graphics()
      const botGfx = new Graphics()
      world.addChild(tileContainer, objectGfx, playerGfx, botGfx)
      app.stage.addChild(world)
      layersRef.current = { tiles: tileContainer, objects: objectGfx, players: playerGfx, bot: botGfx }

      // Draw whatever is already loaded (state was fetched before Pixi was ready)
      const b = botRef.current

      if (b.tiles.length > 0) {
        renderTilesInitial(tileContainer, b, itemColorsRef.current)
      }

      for (const obj of b.objects) {
        objectGfx.circle(obj.x * TILE_PX + TILE_PX / 2, obj.y * TILE_PX + TILE_PX / 2, 2).fill(0xf5c518)
      }

      for (const p of b.players.values()) {
        playerGfx.rect(p.pos_x * TILE_PX + (TILE_PX - CHAR_PX) / 2, (p.pos_y + 1) * TILE_PX - CHAR_PX, CHAR_PX, CHAR_PX).fill(0x60a5fa)
      }

      botGfx.rect(b.pos_x * TILE_PX + (TILE_PX - CHAR_PX) / 2, (b.pos_y + 1) * TILE_PX - CHAR_PX, CHAR_PX, CHAR_PX).fill(0xef4444)

      centerOnBot()
    })

    return () => {
      mounted = false
      if (appRef.current) {
        appRef.current.destroy(true, { children: true })
        appRef.current = null
      }
      layersRef.current = null
    }
  }, [centerOnBot])

  useEffect(() => {
    const layers = layersRef.current
    if (!layers || bot.tiles.length === 0) return
    
    if (useTileManager && items.length > 0) {
      renderTilesWithTileManager(layers.tiles, bot, items)
        .then(() => {
          centerOnBot()
        })
        .catch(() => {
          layers.tiles.removeChildren()
          const g = new Graphics()
          for (let i = 0; i < bot.tiles.length; i++) {
            const tile = bot.tiles[i]
            if (tile.fg === 0) continue
            const color = itemColors[tile.fg] ?? 0x4a4a5a
            const tx = (i % bot.world_width) * TILE_PX
            const ty = Math.floor(i / bot.world_width) * TILE_PX
            g.rect(tx, ty, TILE_PX, TILE_PX).fill(color)
          }
          layers.tiles.addChild(g)
        })
    } else {
      layers.tiles.removeChildren()
      const g = new Graphics()
      for (let i = 0; i < bot.tiles.length; i++) {
        const tile = bot.tiles[i]
        if (tile.fg === 0) continue
        const color = itemColors[tile.fg] ?? 0x4a4a5a
        const tx = (i % bot.world_width) * TILE_PX
        const ty = Math.floor(i / bot.world_width) * TILE_PX
        g.rect(tx, ty, TILE_PX, TILE_PX).fill(color)
      }
      layers.tiles.addChild(g)
      centerOnBot()
    }
  }, [bot.tiles, bot.world_width, itemColors, useTileManager, items, centerOnBot])

  useEffect(() => {
    const layers = layersRef.current
    if (!layers) return
    const g = layers.objects
    g.clear()
    for (const obj of bot.objects) {
      const px = obj.x * TILE_PX
      const py = obj.y * TILE_PX
      g.circle(px + TILE_PX / 2, py + TILE_PX / 2, 2).fill(0xf5c518)
    }
  }, [bot.objects])

  useEffect(() => {
    const layers = layersRef.current
    if (!layers) return
    const g = layers.players
    g.clear()
    for (const p of bot.players.values()) {
      g.rect(p.pos_x * TILE_PX, p.pos_y * TILE_PX, TILE_PX, TILE_PX).fill(0x60a5fa)
    }
  }, [bot.players])

  useEffect(() => {
    const layers = layersRef.current
    if (layers) {
      layers.bot.clear()
      layers.bot.rect(bot.pos_x * TILE_PX + (TILE_PX - CHAR_PX) / 2, (bot.pos_y + 1) * TILE_PX - CHAR_PX, CHAR_PX, CHAR_PX).fill(0xef4444)
    }
    centerOnBot()
  }, [bot.pos_x, bot.pos_y, centerOnBot])

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = containerRef.current!.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    const tileX = Math.floor((mx - offset.current.x) / (zoom.current * TILE_PX))
    const tileY = Math.floor((my - offset.current.y) / (zoom.current * TILE_PX))
    if (tileX < 0 || tileY < 0 || tileX >= bot.world_width || tileY >= bot.world_height) {
      setTooltip(null)
      return
    }
    const tile = bot.tiles[tileY * bot.world_width + tileX]
    if (!tile) { setTooltip(null); return }
    setTooltip({ screenX: e.clientX, screenY: e.clientY, tileX, tileY, tile })
  }

  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (isDragging.current) return
      const rect = containerRef.current!.getBoundingClientRect()
      const mx = e.clientX - rect.left
      const my = e.clientY - rect.top
      const worldX = (mx - offset.current.x) / (zoom.current * TILE_PX)
      const worldY = (my - offset.current.y) / (zoom.current * TILE_PX)
      api.sendCmd(bot.id, { type: 'walk_to', x: Math.floor(worldX), y: Math.floor(worldY) }).catch(() => {})
    },
    [bot.id],
  )

  const handleMouseDown = (e: React.MouseEvent) => {
    isDragging.current = false
    dragStart.current = { x: e.clientX, y: e.clientY, ox: offset.current.x, oy: offset.current.y }

    const onMove = (ev: MouseEvent) => {
      const dx = ev.clientX - dragStart.current.x
      const dy = ev.clientY - dragStart.current.y
      if (Math.abs(dx) > 4 || Math.abs(dy) > 4) isDragging.current = true
      const dims = getWorldScreenSize()
      if (!dims) return
      offset.current = clampOffset(
        dragStart.current.ox + dx,
        dragStart.current.oy + dy,
        dims.worldW, dims.worldH, dims.screenW, dims.screenH,
      )
      applyTransform()
    }
    const onUp = () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  // ── Scroll to zoom (clamped) ───────────────────────────────────────────────

  const handleWheel = useCallback((e: WheelEvent) => {
    e.preventDefault()
    e.stopPropagation()
    const rect = containerRef.current!.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    const oldZoom = zoom.current
    zoom.current = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom.current * (e.deltaY < 0 ? 1.15 : 0.87)))
    setCurrentZoom(Math.round(zoom.current * 100) / 100)
    const ratio = zoom.current / oldZoom

    const raw = {
      x: mx - (mx - offset.current.x) * ratio,
      y: my - (my - offset.current.y) * ratio,
    }
    const dims = getWorldScreenSize()
    if (!dims) return
    offset.current = clampOffset(raw.x, raw.y, dims.worldW, dims.worldH, dims.screenW, dims.screenH)
    applyTransform()
  }, [])

  const handleZoomChange = useCallback((delta: number) => {
    const app = appRef.current
    if (!app) return

    const oldZoom = zoom.current
    zoom.current = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom.current + delta))
    setCurrentZoom(Math.round(zoom.current * 100) / 100)
    const ratio = zoom.current / oldZoom

    const centerX = app.screen.width / 2
    const centerY = app.screen.height / 2

    const raw = {
      x: centerX - (centerX - offset.current.x) * ratio,
      y: centerY - (centerY - offset.current.y) * ratio,
    }
    const dims = getWorldScreenSize()
    if (!dims) return
    offset.current = clampOffset(raw.x, raw.y, dims.worldW, dims.worldH, dims.screenW, dims.screenH)
    applyTransform()
  }, [])

  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    container.addEventListener('wheel', handleWheel, { passive: false })
    return () => {
      container.removeEventListener('wheel', handleWheel)
    }
  }, [handleWheel])

  return (
    <div className="relative w-full aspect-video">
      <div
        ref={containerRef}
        className="w-full h-full rounded border border-border bg-[#080c10] cursor-crosshair overflow-hidden"
        onClick={handleClick}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseLeave={() => setTooltip(null)}
      />

      {loadingProgress && (
        <div className="absolute inset-0 bg-black/50 rounded flex flex-col items-center justify-center gap-3 pointer-events-none">
          <div className="w-48">
            <Progress 
              value={(loadingProgress.current / loadingProgress.total) * 100}
              className="h-2"
            />
          </div>
          <div className="text-sm text-foreground font-medium">
            {loadingProgress.stage === 'loading' 
              ? `Loading textures: ${loadingProgress.current} / ${loadingProgress.total}`
              : `Decoding textures: ${loadingProgress.current} / ${loadingProgress.total}`
            }
          </div>
        </div>
      )}

      <button
        onClick={() => {
          setUseTileManager(!useTileManager)
        }}
        className="absolute top-2 right-2 px-2 py-1 text-xs bg-background border border-border rounded hover:bg-accent"
        title="Toggle accurate tile rendering"
      >
        {useTileManager ? '🎨 Sprites' : '🎨 Colors'}
      </button>

      <div className="absolute bottom-2 right-2 flex items-center gap-1 bg-background border border-border rounded p-1">
        <button
          onClick={() => handleZoomChange(-0.5)}
          disabled={currentZoom <= MIN_ZOOM}
          className="w-6 h-6 flex items-center justify-center text-xs hover:bg-accent rounded disabled:opacity-50 disabled:cursor-not-allowed"
          title="Zoom out"
        >
          −
        </button>
        <span className="px-2 text-xs font-mono min-w-[3.5rem] text-center">
          {Math.round(currentZoom * 100)}%
        </span>
        <button
          onClick={() => handleZoomChange(0.5)}
          disabled={currentZoom >= MAX_ZOOM}
          className="w-6 h-6 flex items-center justify-center text-xs hover:bg-accent rounded disabled:opacity-50 disabled:cursor-not-allowed"
          title="Zoom in"
        >
          +
        </button>
      </div>

      {tooltip && (
        <div
          className="fixed z-50 pointer-events-none bg-popover border border-border rounded px-2 py-1.5 text-[11px] shadow-md space-y-0.5"
          style={{ left: tooltip.screenX + 14, top: tooltip.screenY + 10 }}
        >
          <div className="text-muted-foreground font-mono">
            ({tooltip.tileX}, {tooltip.tileY})
          </div>
          <div>
            <span className="text-muted-foreground">FG </span>
            <span className="font-medium">
              {itemNames[tooltip.tile.fg] ?? `#${tooltip.tile.fg}`}
            </span>
          </div>
          <div>
            <span className="text-muted-foreground">BG </span>
            <span className="font-medium">
              {itemNames[tooltip.tile.bg] ?? `#${tooltip.tile.bg}`}
            </span>
          </div>
          {activeFlags(tooltip.tile.flags).length > 0 && (
            <div className="text-muted-foreground">
              {activeFlags(tooltip.tile.flags).join(', ')}
            </div>
          )}
          {(() => {
            const extra = tileTypeExtra(tooltip.tile.tile_type as TileTypeData, itemNames)
            return extra ? <div className="text-primary">{extra}</div> : null
          })()}
        </div>
      )}
    </div>
  )
}
