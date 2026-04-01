import { atom, useAtom, useAtomValue, useSetAtom } from 'jotai'
import { useEffect } from 'react'
import { moriWs, type TileData } from './ws'
import type { BotStatus, BotSummary, InventoryItem, Player, WorldObject, ItemRecord } from './api'
import { api } from './api'

export interface TrackInfo {
  level: number
  grow_id: number
  install_date: number
  global_playtime: number
  awesomeness: number
}

export interface LiveBot {
  id: number
  username: string
  status: BotStatus
  world_name: string
  pos_x: number
  pos_y: number
  gems: number
  ping_ms: number
  world_width: number
  world_height: number
  tiles: TileData[]
  players: Map<number, Player>
  objects: WorldObject[]
  inventory: InventoryItem[]
  inventory_slots: number
  console: string[]
  delays: { place_ms: number; walk_ms: number; twofa_secs: number; server_overload_secs: number; too_many_logins_secs: number }
  track_info: TrackInfo | null
  auto_collect: boolean
  collect_radius_tiles: number
  collect_blacklist: number[]
  auto_reconnect: boolean
}

export function makeBot(id: number, username: string): LiveBot {
  return {
    id, username,
    status: 'connecting',
    world_name: '', pos_x: 0, pos_y: 0,
    gems: 0, ping_ms: 0,
    world_width: 100, world_height: 60,
    tiles: [], players: new Map(),
    objects: [], inventory: [], inventory_slots: 0, console: [],
    delays: { place_ms: 500, walk_ms: 500, twofa_secs: 120, server_overload_secs: 30, too_many_logins_secs: 5 },
    track_info: null,
    auto_collect: true,
    collect_radius_tiles: 3,
    collect_blacklist: [],
    auto_reconnect: true,
  }
}

export const botsAtom = atom<Map<number, LiveBot>>(new Map())
export const selectedBotIdAtom = atom<number | null>(null)
export const itemNamesAtom = atom<Record<string, string>>({})
export const itemColorsAtom = atom<Record<string, number>>({})
export const itemsMapAtom = atom<Map<number, ItemRecord>>(new Map())

export const selectedBotAtom = atom((get) => {
  const id = get(selectedBotIdAtom)
  return id !== null ? (get(botsAtom).get(id) ?? null) : null
})

function patchBot(
  map: Map<number, LiveBot>,
  id: number,
  patch: Partial<LiveBot>,
): Map<number, LiveBot> {
  const bot = map.get(id)
  if (!bot) return map
  return new Map(map).set(id, { ...bot, ...patch })
}

export function useMoriStore() {
  const setBots = useSetAtom(botsAtom)
  const setItemNames = useSetAtom(itemNamesAtom)
  const setItemColors = useSetAtom(itemColorsAtom)

  useEffect(() => {
    moriWs.connect()

    api.getBots().then((list: BotSummary[]) => {
      const map = new Map<number, LiveBot>()
      for (const b of list) {
        map.set(b.id, {
          ...makeBot(b.id, b.username),
          status: b.status,
          world_name: b.world,
          pos_x: b.pos_x, pos_y: b.pos_y,
          gems: b.gems, ping_ms: b.ping_ms,
        })
      }
      setBots(map)
    }).catch(() => {})

    api.getItemNames().then(setItemNames).catch(() => {})
    api.getItemColors().then(setItemColors).catch(() => {})

    const handlers: Array<[string, (d: never) => void]> = [
      ['BotAdded', (d: { bot_id: number; username: string }) =>
        setBots((m) => new Map(m).set(d.bot_id, makeBot(d.bot_id, d.username)))],

      ['BotRemoved', (d: { bot_id: number }) =>
        setBots((m) => { const n = new Map(m); n.delete(d.bot_id); return n })],

      ['BotStatus', (d: { bot_id: number; status: BotStatus }) =>
        setBots((m) => patchBot(m, d.bot_id, { status: d.status }))],

      ['BotWorld', (d: { bot_id: number; world_name: string }) =>
        setBots((m) => patchBot(m, d.bot_id, {
          world_name: d.world_name, tiles: [], players: new Map(), objects: [],
        }))],

      ['BotMove', (d: { bot_id: number; x: number; y: number }) =>
        setBots((m) => patchBot(m, d.bot_id, { pos_x: d.x, pos_y: d.y }))],

      ['BotGems', (d: { bot_id: number; gems: number }) =>
        setBots((m) => patchBot(m, d.bot_id, { gems: d.gems }))],

      ['BotPing', (d: { bot_id: number; ping_ms: number }) =>
        setBots((m) => patchBot(m, d.bot_id, { ping_ms: d.ping_ms }))],

      ['BotTrackInfo', (d: { bot_id: number; level: number; grow_id: number; install_date: number; global_playtime: number; awesomeness: number }) =>
        setBots((m) => patchBot(m, d.bot_id, {
          track_info: {
            level: d.level, grow_id: d.grow_id,
            install_date: d.install_date, global_playtime: d.global_playtime,
            awesomeness: d.awesomeness,
          },
        }))],

      ['PlayerSpawn', (d: { bot_id: number; net_id: number; name: string; country: string; x: number; y: number }) =>
        setBots((m) => {
          const bot = m.get(d.bot_id)
          if (!bot) return m
          const players = new Map(bot.players)
          players.set(d.net_id, { net_id: d.net_id, name: d.name, country: d.country, pos_x: d.x, pos_y: d.y })
          return patchBot(m, d.bot_id, { players })
        })],

      ['PlayerMove', (d: { bot_id: number; net_id: number; x: number; y: number }) =>
        setBots((m) => {
          const bot = m.get(d.bot_id)
          if (!bot) return m
          const existing = bot.players.get(d.net_id)
          if (!existing) return m
          const players = new Map(bot.players)
          players.set(d.net_id, { ...existing, pos_x: d.x, pos_y: d.y })
          return patchBot(m, d.bot_id, { players })
        })],

      ['PlayerLeave', (d: { bot_id: number; net_id: number }) =>
        setBots((m) => {
          const bot = m.get(d.bot_id)
          if (!bot) return m
          const players = new Map(bot.players)
          players.delete(d.net_id)
          return patchBot(m, d.bot_id, { players })
        })],

      ['WorldLoaded', (d: { bot_id: number; name: string; width: number; height: number; tiles: TileData[] }) =>
        setBots((m) => patchBot(m, d.bot_id, {
          world_name: d.name,
          world_width: d.width, world_height: d.height,
          tiles: d.tiles, players: new Map(), objects: [],
        }))],

      ['TileUpdate', (d: { bot_id: number; x: number; y: number; fg: number; bg: number }) =>
        setBots((m) => {
          const bot = m.get(d.bot_id)
          if (!bot) return m
          const tiles = [...bot.tiles]
          const idx = d.y * bot.world_width + d.x
          if (tiles[idx]) tiles[idx] = { ...tiles[idx], fg: d.fg, bg: d.bg }
          return patchBot(m, d.bot_id, { tiles })
        })],

      ['ObjectsUpdate', (d: { bot_id: number; objects: WorldObject[] }) =>
        setBots((m) => patchBot(m, d.bot_id, { objects: d.objects }))],

      ['InventoryUpdate', (d: { bot_id: number; gems: number; inventory_size: number; items: InventoryItem[] }) =>
        setBots((m) => patchBot(m, d.bot_id, { gems: d.gems, inventory: d.items, inventory_slots: d.inventory_size }))],

      ['BotUsername', (d: { bot_id: number; username: string }) =>
        setBots((m) => patchBot(m, d.bot_id, { username: d.username }))],

      ['Console', (d: { bot_id: number; message: string }) =>
        setBots((m) => {
          const bot = m.get(d.bot_id)
          if (!bot) return m
          const console_ = [...bot.console, d.message].slice(-500)
          return patchBot(m, d.bot_id, { console: console_ })
        })],
    ]

    for (const [event, handler] of handlers) {
      moriWs.bus.on(event as never, handler as never)
    }
    return () => {
      for (const [event, handler] of handlers) {
        moriWs.bus.off(event as never, handler as never)
      }
    }
  }, [setBots, setItemNames])
}

export { useAtom, useAtomValue, useSetAtom }
