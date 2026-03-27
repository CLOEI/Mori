import type { BotStatus, WorldObject, InventoryItem } from './api'

// ── Event payload types ────────────────────────────────────────────────────

export interface TileData {
  fg: number
  bg: number
  flags: number
  tile_type: { type: string } & Record<string, unknown>
}

export type WsEventMap = {
  BotAdded: { bot_id: number; username: string }
  BotRemoved: { bot_id: number }
  BotStatus: { bot_id: number; status: BotStatus }
  BotWorld: { bot_id: number; world_name: string }
  BotMove: { bot_id: number; x: number; y: number }
  BotGems: { bot_id: number; gems: number }
  BotPing: { bot_id: number; ping_ms: number }
  BotTrackInfo: {
    bot_id: number
    level: number
    grow_id: number
    install_date: number
    global_playtime: number
    awesomeness: number
  }
  PlayerSpawn: { bot_id: number; net_id: number; name: string; country: string; x: number; y: number }
  PlayerMove: { bot_id: number; net_id: number; x: number; y: number }
  PlayerLeave: { bot_id: number; net_id: number }
  WorldLoaded: { bot_id: number; name: string; width: number; height: number; tiles: TileData[] }
  TileUpdate: { bot_id: number; x: number; y: number; fg: number; bg: number }
  ObjectsUpdate: { bot_id: number; objects: WorldObject[] }
  InventoryUpdate: { bot_id: number; gems: number; items: InventoryItem[] }
  Console: { bot_id: number; message: string }
}

type Listener<K extends keyof WsEventMap> = (data: WsEventMap[K]) => void
type AnyListener = (event: string, data: unknown) => void

// ── EventBus ───────────────────────────────────────────────────────────────

class EventBus {
  private listeners = new Map<string, Set<Listener<never>>>()
  private anyListeners = new Set<AnyListener>()

  on<K extends keyof WsEventMap>(event: K, fn: Listener<K>) {
    if (!this.listeners.has(event)) this.listeners.set(event, new Set())
    this.listeners.get(event)!.add(fn as Listener<never>)
  }

  off<K extends keyof WsEventMap>(event: K, fn: Listener<K>) {
    this.listeners.get(event)?.delete(fn as Listener<never>)
  }

  onAny(fn: AnyListener) {
    this.anyListeners.add(fn)
  }

  offAny(fn: AnyListener) {
    this.anyListeners.delete(fn)
  }

  emit<K extends keyof WsEventMap>(event: K, data: WsEventMap[K]) {
    this.listeners.get(event)?.forEach((fn) => fn(data as never))
    this.anyListeners.forEach((fn) => fn(event, data))
  }
}

// ── MoriWebSocket singleton ────────────────────────────────────────────────

class MoriWebSocket {
  readonly bus = new EventBus()
  private ws: WebSocket | null = null
  private retryTimer: ReturnType<typeof setTimeout> | null = null
  private destroyed = false

  connect(url = `${window.location.protocol === 'https:' ? 'wss' : 'ws'}://${window.location.hostname}:3000/ws`) {
    if (this.ws) return
    this.ws = new WebSocket(url)

    this.ws.onmessage = (e) => {
      try {
        const { event, data } = JSON.parse(e.data) as { event: string; data: unknown }
        this.bus.emit(event as keyof WsEventMap, data as never)
      } catch {
        // ignore malformed frames
      }
    }

    this.ws.onclose = () => {
      this.ws = null
      if (!this.destroyed) {
        this.retryTimer = setTimeout(() => this.connect(url), 3000)
      }
    }

    this.ws.onerror = () => {
      this.ws?.close()
    }
  }

  disconnect() {
    this.destroyed = true
    if (this.retryTimer) clearTimeout(this.retryTimer)
    this.ws?.close()
    this.ws = null
  }
}

export const moriWs = new MoriWebSocket()
