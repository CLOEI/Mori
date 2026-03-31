import { authStore } from './auth'

const BASE = window.location.origin;

async function req<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const token = authStore.getToken()
  const headers: Record<string, string> = {}
  if (body) headers['Content-Type'] = 'application/json'
  if (token) headers['Authorization'] = `Bearer ${token}`
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });
  if (res.status === 401) {
    authStore.clearToken()
    window.dispatchEvent(new CustomEvent('mori:unauthorized'))
    throw new Error('Unauthorized')
  }
  if (res.status === 204) return undefined as T;
  if (!res.ok) throw new Error(`${method} ${path} → ${res.status}`);
  return res.json() as Promise<T>;
}
async function external_req<T>(
  url: string,
  method: string,
  path: string,
  headers?: unknown,
  body?: unknown,
): Promise<T> {
  const res = await fetch(`${url}${path}`, {
    method,
    headers: headers ? { ...headers } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${method} ${path} → ${res.status}: ${text}`);
  }
  if (res.headers.get("Content-Type")?.includes("application/json")) {
    return res.json() as Promise<T>;
  } else {
    return res.blob() as Promise<T>;
  }
}

export type BotStatus =
  | "connecting"
  | "connected"
  | "in_game"
  | "two_factor_auth"
  | "server_overloaded"
  | "too_many_logins"
  | "update_required";

export interface BotSummary {
  id: number;
  username: string;
  status: BotStatus;
  world: string;
  pos_x: number;
  pos_y: number;
  gems: number;
  ping_ms: number;
}

export interface Player {
  net_id: number;
  name: string;
  pos_x: number;
  pos_y: number;
  country: string;
}

export interface WorldObject {
  uid: number;
  item_id: number;
  x: number;
  y: number;
  count: number;
}

export interface InventoryItem {
  item_id: number;
  amount: number;
  is_active: boolean;
  action_type: number;
}

export interface BotState {
  status: BotStatus;
  world_name: string;
  pos_x: number;
  pos_y: number;
  world_width: number;
  world_height: number;
  tiles: {
    fg_item_id: number;
    bg_item_id: number;
    flags: number;
    tile_type: { type: string } & Record<string, unknown>;
  }[];
  players: Player[];
  objects: WorldObject[];
  inventory: InventoryItem[];
  inventory_slots: number;
  gems: number;
  console: string[];
  ping_ms: number;
  delays: {
    place_ms: number;
    walk_ms: number;
    twofa_secs: number;
    server_overload_secs: number;
    too_many_logins_secs: number;
  };
  track_info: {
    level: number;
    grow_id: number;
    install_date: number;
    global_playtime: number;
    awesomeness: number;
  } | null;
  auto_collect: boolean;
  collect_radius_tiles: number;
  collect_blacklist: number[];
}

export interface SpawnBotBody {
  username: string;
  password: string;
  proxy_host?: string;
  proxy_port?: number;
  proxy_username?: string;
  proxy_password?: string;
}

export interface SpawnLtokenBody {
  ltoken: string;
  proxy_host?: string;
  proxy_port?: number;
  proxy_username?: string;
  proxy_password?: string;
}

export type BotCmd =
  | { type: "move"; x: number; y: number }
  | { type: "walk_to"; x: number; y: number }
  | { type: "run_script"; content: string }
  | { type: "stop_script" }
  | { type: "wear"; item_id: number }
  | { type: "unwear"; item_id: number }
  | { type: "drop"; item_id: number; count: number }
  | { type: "trash"; item_id: number; count: number }
  | {
      type: "set_delays";
      place_ms: number;
      walk_ms: number;
      twofa_secs: number;
      server_overload_secs: number;
      too_many_logins_secs: number;
    }
  | { type: "set_auto_collect"; enabled: boolean }
  | { type: "set_collect_config"; radius_tiles: number; blacklist: number[] };

export interface ProxyTestRequest {
  proxy_host: string;
  proxy_port: number;
  proxy_username?: string;
  proxy_password?: string;
}

export interface CheckResult {
  ok: boolean;
  error?: string;
  detail?: string;
}

export interface ProxyTestResult {
  socks5: CheckResult;
  server_data: CheckResult;
  enet: CheckResult;
}

export interface ItemRecord {
  id: number;
  flags: number;
  action_type: number;
  material: number;
  name: string;
  texture_file_name: string;
  texture_hash: number;
  visual_effect: number;
  cooking_ingredient: number;
  texture_x: number;
  texture_y: number;
  render_type: number;
  is_stripey_wallpaper: number;
  collision_type: number;
  block_health: number;
  drop_chance: number;
  clothing_type: number;
  rarity: number;
  max_item: number;
  file_name: string;
  file_hash: number;
  audio_volume: number;
  pet_name: string;
  pet_prefix: string;
  pet_suffix: string;
  pet_ability: string;
  seed_base_sprite: number;
  seed_overlay_sprite: number;
  tree_base_sprite: number;
  tree_overlay_sprite: number;
  base_color: number;
  overlay_color: number;
  ingredient: number;
  grow_time: number;
  is_rayman: number;
  extra_options: string;
  texture_path_2: string;
  extra_option2: string;
  punch_option: string;
  description: string;
}

export interface ItemsPage {
  items: ItemRecord[];
  total: number;
  page: number;
  page_size: number;
}

export const api = {
  // Auth
  authStatus: () => fetch(`${BASE}/auth/status`).then(r => r.json()) as Promise<{ registered: boolean }>,
  authSetup: (password: string) => fetch(`${BASE}/auth/setup`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ password }) }),
  authLogin: (password: string) => fetch(`${BASE}/auth/login`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ password }) }),
  authLogout: () => req<void>('POST', '/auth/logout'),

  // Bots
  getBots: () => req<BotSummary[]>("GET", "/bots"),
  spawnBot: (body: SpawnBotBody) => req<{ id: number }>("POST", "/bots", body),
  spawnLtokenBot: (body: SpawnLtokenBody) =>
    req<{ id: number }>("POST", "/bots/ltoken", body),
  deleteBot: (id: number) => req<void>("DELETE", `/bots/${id}`),
  getBotState: (id: number) => req<BotState>("GET", `/bots/${id}/state`),
  sendCmd: (id: number, cmd: BotCmd) =>
    req<void>("POST", `/bots/${id}/cmd`, cmd),
  getItemNames: () => req<Record<string, string>>("GET", "/items/names"),
  getItemColors: () => req<Record<string, number>>("GET", "/items/colors"),
  getItems: (page = 1, q = "") =>
    req<ItemsPage>(
      "GET",
      `/items?page=${page}${q ? `&q=${encodeURIComponent(q)}` : ""}`,
    ),
  getItemsByIds: (ids: number[]) =>
    req<ItemRecord[]>(
      "GET",
      `/items?get-items=${ids.join(",")}`,
    ),
  testProxy: (body: ProxyTestRequest) =>
    req<ProxyTestResult>("POST", "/proxy/test", body),
};

export const external_api = {
  stiledevs: {
    getGameTextureAssets: (path: string) =>
      external_req<Blob>(
        window.location.origin,
        "GET",
        `/growtopia-cdn/growtopia/game/${path}`,
      ),
  },
};
