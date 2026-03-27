import { useCallback, useEffect, useRef, useState } from 'react'
import { Application, Graphics, Container } from 'pixi.js'
import { useAtomValue } from 'jotai'
import { itemNamesAtom, type LiveBot } from '@/lib/store'
import { api } from '@/lib/api'
import type { TileData } from '@/lib/ws'

const TILE_FLAGS: [number, string][] = [
  // 0x0001 extra_data intentionally skipped — implied by tile_type
  [0x0002, 'has_parent'], [0x0004, 'spliced'], [0x0008, 'spawns_seeds'],
  [0x0010, 'seedling'], [0x0020, 'flipped_x'], [0x0040, 'on'],
  [0x0080, 'public'], [0x0100, 'bg_on'], [0x0200, 'alt_mode'],
  [0x0400, 'wet'], [0x0800, 'glued'], [0x1000, 'on_fire'],
  [0x2000, 'painted_red'], [0x4000, 'painted_green'], [0x8000, 'painted_blue'],
]

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
const MIN_ZOOM = 0.5
const MAX_ZOOM = 4

function tileColor(fg: number): number | null {
  if (fg === 0) return null
  if (fg === 2) return 0x5c4a2a
  if (fg === 4) return 0x2d6e2d
  if (fg === 8) return 0x8b7355
  if (fg === 10) return 0x4a90d9
  if (fg === 1000) return 0x6b4226
  if (fg >= 6 && fg <= 7) return 0xc8a850
  return 0x4a4a5a
}

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
  const [tooltip, setTooltip] = useState<TooltipState | null>(null)
  const appRef = useRef<Application | null>(null)
  const layersRef = useRef<{
    tiles: Graphics
    objects: Graphics
    players: Graphics
    bot: Graphics
  } | null>(null)

  const botRef = useRef(bot)
  botRef.current = bot

  const zoom = useRef(3)
  const offset = useRef({ x: 0, y: 0 })
  const isDragging = useRef(false)
  const dragStart = useRef({ x: 0, y: 0, ox: 0, oy: 0 })

  // ── Helpers ────────────────────────────────────────────────────────────────

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

  function centerOnBot() {
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
  }

  // ── Init Pixi ──────────────────────────────────────────────────────────────

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
      const tileGfx = new Graphics()
      const objectGfx = new Graphics()
      const playerGfx = new Graphics()
      const botGfx = new Graphics()
      world.addChild(tileGfx, objectGfx, playerGfx, botGfx)
      app.stage.addChild(world)
      layersRef.current = { tiles: tileGfx, objects: objectGfx, players: playerGfx, bot: botGfx }

      // Draw whatever is already loaded (state was fetched before Pixi was ready)
      const b = botRef.current

      if (b.tiles.length > 0) {
        for (let i = 0; i < b.tiles.length; i++) {
          const tile = b.tiles[i]
          const color = tileColor(tile.fg)
          if (color === null) continue
          const tx = (i % b.world_width) * TILE_PX
          const ty = Math.floor(i / b.world_width) * TILE_PX
          tileGfx.rect(tx, ty, TILE_PX - 1, TILE_PX - 1).fill(color)
        }
      }

      for (const obj of b.objects) {
        objectGfx.circle(obj.x * TILE_PX + TILE_PX / 2, obj.y * TILE_PX + TILE_PX / 2, 2).fill(0xf5c518)
      }

      for (const p of b.players.values()) {
        playerGfx.rect(p.pos_x * TILE_PX, p.pos_y * TILE_PX, TILE_PX - 1, TILE_PX - 1).fill(0x60a5fa)
      }

      botGfx.rect(b.pos_x * TILE_PX, b.pos_y * TILE_PX, TILE_PX - 1, TILE_PX - 1).fill(0xef4444)

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
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ── Draw tiles ─────────────────────────────────────────────────────────────

  useEffect(() => {
    const layers = layersRef.current
    if (!layers || bot.tiles.length === 0) return
    const g = layers.tiles
    g.clear()
    for (let i = 0; i < bot.tiles.length; i++) {
      const tile = bot.tiles[i]
      const color = tileColor(tile.fg)
      if (color === null) continue
      const tx = (i % bot.world_width) * TILE_PX
      const ty = Math.floor(i / bot.world_width) * TILE_PX
      g.rect(tx, ty, TILE_PX - 1, TILE_PX - 1).fill(color)
    }
    centerOnBot()
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bot.tiles, bot.world_width])

  // ── Draw objects ───────────────────────────────────────────────────────────

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

  // ── Draw players ───────────────────────────────────────────────────────────

  useEffect(() => {
    const layers = layersRef.current
    if (!layers) return
    const g = layers.players
    g.clear()
    for (const p of bot.players.values()) {
      g.rect(p.pos_x * TILE_PX, p.pos_y * TILE_PX, TILE_PX - 1, TILE_PX - 1).fill(0x60a5fa)
    }
  }, [bot.players])

  // ── Draw bot square + follow camera ───────────────────────────────────────

  useEffect(() => {
    const layers = layersRef.current
    if (layers) {
      layers.bot.clear()
      layers.bot.rect(bot.pos_x * TILE_PX, bot.pos_y * TILE_PX, TILE_PX - 1, TILE_PX - 1).fill(0xef4444)
    }
    centerOnBot()
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bot.pos_x, bot.pos_y])

  // ── Hover: tile tooltip ────────────────────────────────────────────────────

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

  // ── Click to walk ──────────────────────────────────────────────────────────

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

  // ── Drag to pan (clamped) ──────────────────────────────────────────────────

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

  const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    e.preventDefault()
    const rect = containerRef.current!.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    const oldZoom = zoom.current
    zoom.current = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom.current * (e.deltaY < 0 ? 1.15 : 0.87)))
    const ratio = zoom.current / oldZoom

    const raw = {
      x: mx - (mx - offset.current.x) * ratio,
      y: my - (my - offset.current.y) * ratio,
    }
    const dims = getWorldScreenSize()
    if (!dims) return
    offset.current = clampOffset(raw.x, raw.y, dims.worldW, dims.worldH, dims.screenW, dims.screenH)
    applyTransform()
  }

  return (
    <div className="relative w-full aspect-video">
      <div
        ref={containerRef}
        className="w-full h-full rounded border border-border bg-[#080c10] cursor-crosshair overflow-hidden"
        onClick={handleClick}
        onMouseDown={handleMouseDown}
        onWheel={handleWheel}
        onMouseMove={handleMouseMove}
        onMouseLeave={() => setTooltip(null)}
      />

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
