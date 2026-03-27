const BASE = 'http://localhost:3000'

async function req<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  })
  if (res.status === 204) return undefined as T
  if (!res.ok) throw new Error(`${method} ${path} → ${res.status}`)
  return res.json() as Promise<T>
}

// ── Types ──────────────────────────────────────────────────────────────────

export type BotStatus =
  | 'connecting'
  | 'connected'
  | 'in_world'
  | 'two_factor_auth'
  | 'server_overloaded'

export interface BotSummary {
  id: number
  username: string
  status: BotStatus
  world: string
  pos_x: number
  pos_y: number
  gems: number
  ping_ms: number
}

export interface Player {
  net_id: number
  name: string
  pos_x: number
  pos_y: number
  country: string
}

export interface WorldObject {
  uid: number
  item_id: number
  x: number
  y: number
  count: number
}

export interface InventoryItem {
  item_id: number
  amount: number
  is_active: boolean
  action_type: number
}

export interface BotState {
  status: BotStatus
  world_name: string
  pos_x: number
  pos_y: number
  world_width: number
  world_height: number
  tiles: { fg_item_id: number; bg_item_id: number; flags: number; tile_type: { type: string } & Record<string, unknown> }[]
  players: Player[]
  objects: WorldObject[]
  inventory: InventoryItem[]
  gems: number
  console: string[]
  ping_ms: number
  delays: { place_ms: number; walk_ms: number }
  track_info: { level: number; grow_id: number; install_date: number; global_playtime: number; awesomeness: number } | null
}

export interface SpawnBotBody {
  username: string
  password: string
  proxy_host?: string
  proxy_port?: number
  proxy_username?: string
  proxy_password?: string
}

export type BotCmd =
  | { type: 'move'; x: number; y: number }
  | { type: 'walk_to'; x: number; y: number }
  | { type: 'run_script'; content: string }
  | { type: 'stop_script' }
  | { type: 'wear'; item_id: number }
  | { type: 'unwear'; item_id: number }
  | { type: 'drop'; item_id: number; count: number }
  | { type: 'trash'; item_id: number; count: number }
  | { type: 'set_delays'; place_ms: number; walk_ms: number }

export interface ItemRecord {
  id: number
  name: string
  flags: number
  action_type: number
  material: number
  texture_file_name: string
  texture_hash: number
  visual_effect: number
  collision_type: number
  rarity: number
  max_item: number
  grow_time: number
  base_color: number
  overlay_color: number
  clothing_type: number
}

export interface ItemsPage {
  items: ItemRecord[]
  total: number
  page: number
  page_size: number
}

// ── Endpoints ──────────────────────────────────────────────────────────────

export const api = {
  getBots: () => req<BotSummary[]>('GET', '/bots'),
  spawnBot: (body: SpawnBotBody) => req<{ id: number }>('POST', '/bots', body),
  deleteBot: (id: number) => req<void>('DELETE', `/bots/${id}`),
  getBotState: (id: number) => req<BotState>('GET', `/bots/${id}/state`),
  sendCmd: (id: number, cmd: BotCmd) => req<void>('POST', `/bots/${id}/cmd`, cmd),
  getItemNames: () => req<Record<string, string>>('GET', '/items/names'),
  getItems: (page = 1, q = '') =>
    req<ItemsPage>('GET', `/items?page=${page}${q ? `&q=${encodeURIComponent(q)}` : ''}`),
}
