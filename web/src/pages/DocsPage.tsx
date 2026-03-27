import { useState, useMemo } from 'react'
import { Search } from 'lucide-react'
import { Input } from '@/components/ui/input'

type Entry = { sig: string; desc: string }
type Section = { title: string; entries: Entry[] }

const SECTIONS: Section[] = [
  {
    title: 'Globals',
    entries: [
      { sig: 'bot', desc: 'The current bot (BotProxy). Scripts may only control their own bot.' },
      { sig: 'sleep(ms)', desc: 'Pause script execution for the given number of milliseconds.' },
      { sig: 'getInfo(id|name)', desc: 'Get ItemInfo by numeric ID or name string. → ItemInfo?' },
      { sig: 'getInfos()', desc: 'Get all items from items.dat. → ItemInfo[]' },
      { sig: 'getUsername()', desc: "Return the current bot's username. → string" },
      { sig: 'read(path)', desc: 'Read a file from disk and return its contents as a string.' },
      { sig: 'write(path, content)', desc: 'Write a string to a file (overwrite).' },
      { sig: 'append(path, content)', desc: 'Append a string to a file (creates if missing).' },
      { sig: 'removeColor(text)', desc: 'Strip Growtopia backtick-prefixed color codes from a string.' },
      { sig: 'clearConsole()', desc: "Clear the current bot's console output." },
      { sig: 'runThread(fn, ...)', desc: 'Call a function immediately with the given args (synchronous).' },
    ],
  },
  {
    title: 'Shortcut Globals',
    entries: [
      { sig: 'getWorld()', desc: 'Shortcut for bot:getWorld(). → World?' },
      { sig: 'getInventory()', desc: 'Shortcut for bot:getInventory(). → Inventory' },
      { sig: 'getLocal()', desc: 'Shortcut for bot:getLocal(). → Player' },
      { sig: 'getPlayer(key)', desc: 'Get a player by net_id or name from the current world. → Player?' },
      { sig: 'getPlayers()', desc: 'Get all players in the current world. → Player[]' },
      { sig: 'getTile(x, y)', desc: 'Get a tile at position. → Tile?' },
      { sig: 'getTiles()', desc: 'Get all tiles in the current world. → Tile[]' },
      { sig: 'getObject(oid)', desc: 'Get a dropped world object by UID. → NetObject?' },
      { sig: 'getObjects()', desc: 'Get all dropped objects in the current world. → NetObject[]' },
      { sig: 'hasAccess(x, y)', desc: 'Returns false (stub).' },
    ],
  },
  {
    title: 'Events',
    entries: [
      { sig: 'Event.variantlist', desc: '= 1. Handler receives (VariantList, net_id).' },
      { sig: 'Event.gameupdate', desc: '= 2. Handler receives (GameUpdatePacket).' },
      { sig: 'Event.gamemessage', desc: '= 3. Handler receives (text: string).' },
      { sig: 'addEvent(etype, fn)', desc: 'Register a callback for an event type.' },
      { sig: 'removeEvent(etype)', desc: 'Remove the callback for an event type.' },
      { sig: 'removeEvents()', desc: 'Clear all event callbacks.' },
      { sig: 'listenEvents([secs])', desc: 'Pump ENet and fire registered callbacks. Loops until unlistenEvents() if no duration given.' },
      { sig: 'unlistenEvents()', desc: 'Request the current listenEvents loop to exit early.' },
    ],
  },
  {
    title: 'Bot — Fields',
    entries: [
      { sig: 'bot.name', desc: 'Username string.' },
      { sig: 'bot.status', desc: 'Current status string.' },
      { sig: 'bot.gem_count', desc: 'Gem count from inventory.' },
      { sig: 'bot.auto_collect', desc: 'Auto-collect toggle (read/write).' },
    ],
  },
  {
    title: 'Bot — Info / State',
    entries: [
      { sig: 'bot:getWorld()', desc: 'Get the current world snapshot. → World?' },
      { sig: 'bot:getInventory()', desc: 'Get the current inventory snapshot. → Inventory' },
      { sig: 'bot:getLocal()', desc: 'Get a Player object representing this bot. → Player' },
      { sig: 'bot:getConsole()', desc: 'Get the console object. → Console' },
      { sig: 'bot:getLogin()', desc: 'Get login data (MAC address). → Login' },
      { sig: 'bot:getPing()', desc: 'Current round-trip ping in ms. → uint' },
      { sig: 'bot:isInWorld([name])', desc: 'True if in any world, or a specific world if name given. → bool' },
      { sig: 'bot:isInTile(x, y)', desc: 'True if the bot is currently standing on tile (x, y). → bool' },
    ],
  },
  {
    title: 'Bot — Movement',
    entries: [
      { sig: 'bot:moveTo(dx, dy)', desc: 'Move relative to current tile position by (dx, dy) tiles.' },
      { sig: 'bot:moveTile(x, y)', desc: 'Walk to absolute tile (x, y).' },
      { sig: 'bot:moveLeft([n])', desc: 'Move left n tiles (default 1).' },
      { sig: 'bot:moveRight([n])', desc: 'Move right n tiles (default 1).' },
      { sig: 'bot:moveUp([n])', desc: 'Move up n tiles (default 1).' },
      { sig: 'bot:moveDown([n])', desc: 'Move down n tiles (default 1).' },
      { sig: 'bot:setDirection(facing_left)', desc: 'Set bot facing direction.' },
      { sig: 'bot:findPath(x, y)', desc: 'Pathfind to tile (x, y) using A*.' },
      { sig: 'bot:getPath(x, y)', desc: 'Compute and return A* path without walking. → table[]' },
    ],
  },
  {
    title: 'Bot — World Actions',
    entries: [
      { sig: 'bot:warp(name, [id])', desc: 'Warp to a world by name, with optional door ID.' },
      { sig: 'bot:say(text)', desc: 'Send a chat message.' },
      { sig: 'bot:leaveWorld()', desc: 'Leave the current world.' },
      { sig: 'bot:respawn()', desc: 'Respawn the bot.' },
      { sig: 'bot:place(x, y, item)', desc: 'Place item ID at tile (x, y).' },
      { sig: 'bot:hit(x, y)', desc: 'Punch tile at (x, y).' },
      { sig: 'bot:wrench(x, y)', desc: 'Wrench tile at (x, y).' },
      { sig: 'bot:wrenchPlayer(net_id)', desc: 'Wrench a player by net ID.' },
      { sig: 'bot:active(x, y)', desc: 'Activate/enter tile at (x, y).' },
      { sig: 'bot:enter([pass])', desc: 'Activate the tile under the bot, optionally with a password.' },
      { sig: 'bot:collectObject(oid, range)', desc: 'Collect a specific dropped object by UID within range tiles.' },
      { sig: 'bot:collect(range, interval)', desc: 'Collect all nearby objects within range tiles. → int count' },
    ],
  },
  {
    title: 'Bot — Inventory Actions',
    entries: [
      { sig: 'bot:wear(item_id)', desc: 'Wear / equip an item.' },
      { sig: 'bot:unwear(item_id)', desc: 'Unequip an item.' },
      { sig: 'bot:use(item_id)', desc: 'Alias for wear.' },
      { sig: 'bot:consume(item_id)', desc: 'Use / drink an item.' },
      { sig: 'bot:drop(item_id, [count])', desc: 'Drop item by ID (default 1).' },
      { sig: 'bot:trash(item_id)', desc: 'Delete item from inventory.' },
      { sig: 'bot:send(item_id, [count], username)', desc: 'Send item to another player (default count 1).' },
      { sig: 'bot:store(item_id, [count])', desc: "Store item in the current world's storage (not portable)." },
    ],
  },
  {
    title: 'Remote Bot (RBProxy)',
    entries: [
      { sig: 'getRBot(name)', desc: 'Get another bot handle from the controller. → RBProxy?' },
      { sig: 'rb.name / .status / .gem_count', desc: "Read-only fields mirroring the remote bot's state." },
      { sig: 'rb:getWorld()', desc: "Snapshot of remote bot's world (via shared Arc). → World?" },
      { sig: 'rb:getInventory()', desc: "Snapshot of remote bot's inventory. → Inventory" },
      { sig: 'rb:say / warp / moveTo / findPath / place / hit / ...', desc: "Commands queued and executed on the remote bot's thread." },
    ],
  },
  {
    title: 'World',
    entries: [
      { sig: '.name', desc: 'World name string.' },
      { sig: '.x / .y', desc: 'Width / height in tiles.' },
      { sig: '.tile_count / .version / .public', desc: 'Tile count, version number, public flag.' },
      { sig: '.tiles / .objects / .players', desc: 'Arrays of Tile, NetObject, Player.' },
      { sig: ':getTile(x, y)', desc: 'Get Tile at position. → Tile?' },
      { sig: ':getTiles() / :getTilesSafe()', desc: 'Get all tiles as array. → Tile[]' },
      { sig: ':getObject(oid)', desc: 'Get dropped object by UID. → NetObject?' },
      { sig: ':getObjects()', desc: 'All dropped objects. → NetObject[]' },
      { sig: ':getPlayer(net_id|name)', desc: 'Find player by net ID or name. → Player?' },
      { sig: ':getPlayers()', desc: 'All players in world. → Player[]' },
      { sig: ':getLocal()', desc: "Bot's own Player representation. → Player" },
      { sig: ':isValidPosition(x, y)', desc: 'True if tile coords are in bounds. → bool' },
    ],
  },
  {
    title: 'Inventory',
    entries: [
      { sig: '.itemcount / .slotcount', desc: 'Item count and max slot count.' },
      { sig: '.items', desc: 'Array of InventoryItem.' },
      { sig: ':getItem(id)', desc: 'Get item by ID. → InventoryItem?' },
      { sig: ':getItems()', desc: 'All items in inventory. → InventoryItem[]' },
      { sig: ':findItem(id) / :getItemCount(id)', desc: 'Return count of item ID (0 if not present). → uint' },
    ],
  },
  {
    title: 'Player',
    entries: [
      { sig: '.name / .country', desc: 'Display name and country code.' },
      { sig: '.netid / .userid', desc: 'Network ID and user ID.' },
      { sig: '.posx / .posy', desc: 'Pixel-space position (divide by 32 for tile).' },
      { sig: '.avatarFlags', desc: 'Raw avatar state flags.' },
      { sig: '.roleicon', desc: 'Role icon string.' },
    ],
  },
  {
    title: 'Tile',
    entries: [
      { sig: '.fg / .foreground', desc: 'Foreground item ID.' },
      { sig: '.bg / .background', desc: 'Background item ID.' },
      { sig: '.x / .y', desc: 'Tile coordinates.' },
      { sig: '.flags', desc: 'Raw tile flags bitmask.' },
      { sig: ':hasExtra()', desc: 'True if tile has extra data. → bool' },
      { sig: ':getExtra()', desc: 'Returns extra data table. → table?' },
      { sig: ':canHarvest()', desc: 'True if tile is a ready-to-harvest seed. → bool' },
      { sig: ':hasFlag(flag)', desc: 'Test a specific flag bit. → bool' },
    ],
  },
  {
    title: 'NetObject (dropped item)',
    entries: [
      { sig: '.id', desc: 'Item ID.' },
      { sig: '.x / .y', desc: 'Pixel-space position.' },
      { sig: '.count', desc: 'Stack count.' },
      { sig: '.flags', desc: 'Object flags.' },
      { sig: '.oid', desc: 'Unique object UID used with collectObject.' },
    ],
  },
  {
    title: 'ItemInfo',
    entries: [
      { sig: '.id / .name', desc: 'Item ID and display name.' },
      { sig: '.action_type / .collision_type / .clothing_type', desc: 'Type enumerations.' },
      { sig: '.rarity / .grow_time / .drop_chance', desc: 'Economy stats.' },
      { sig: '.texture / .texture_hash / .texture_x / .texture_y', desc: 'Sprite sheet info.' },
      { sig: '.strength', desc: 'Block health / punch count.' },
    ],
  },
  {
    title: 'Console',
    entries: [
      { sig: '.contents', desc: 'Table of log lines (max 100). string[]' },
      { sig: ':append(text)', desc: 'Push a line to the console (visible in the dashboard).' },
    ],
  },
  {
    title: 'GameUpdatePacket',
    entries: [
      { sig: 'GameUpdatePacket.new()', desc: 'Create a new blank packet. → GameUpdatePacket' },
      { sig: '.type / .object_type / .count1 / .count2', desc: 'Packet type and counters (read/write).' },
      { sig: '.netid / .item / .flags', desc: 'Net ID, item target, flags bitmask (read/write).' },
      { sig: '.vec_x/.pos_x / .vec_y/.pos_y', desc: 'Primary position vector (read/write, aliased).' },
      { sig: '.vec2_x/.pos2_x / .vec2_y/.pos2_y', desc: 'Secondary position vector (read/write, aliased).' },
    ],
  },
  {
    title: 'Variant / VariantList',
    entries: [
      { sig: 'variant:getType()', desc: '1=float, 2=string, 3=vec2, 4=vec3, 5=uint, 9=int, 0=unknown. → uint' },
      { sig: 'variant:getString() / :getInt() / :getFloat()', desc: 'Extract typed value.' },
      { sig: 'variant:getVector2()', desc: 'Returns {x, y}.' },
      { sig: 'variantlist:get(idx)', desc: 'Get variant at 0-based index. → Variant?' },
      { sig: 'variantlist:print()', desc: 'Comma-separated string of all variants.' },
    ],
  },
  {
    title: 'Login',
    entries: [
      { sig: '.mac', desc: "The bot's MAC address string." },
    ],
  },
]

export function DocsPage() {
  const [q, setQ] = useState('')

  const filtered = useMemo(() => {
    const term = q.trim().toLowerCase()
    if (!term) return SECTIONS
    return SECTIONS
      .map((s) => ({
        ...s,
        entries: s.entries.filter(
          (e) =>
            e.sig.toLowerCase().includes(term) ||
            e.desc.toLowerCase().includes(term),
        ),
      }))
      .filter((s) => s.entries.length > 0 || s.title.toLowerCase().includes(term))
  }, [q])

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="shrink-0 flex items-center gap-3 px-6 py-3 border-b border-border bg-card">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <Input
            placeholder="Search API…"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            className="pl-8 h-8 text-xs"
          />
        </div>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="max-w-3xl mx-auto px-6 py-6 space-y-7">
          {filtered.map((section) => (
            <div key={section.title}>
              <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-3 pb-2 border-b border-border">
                {section.title}
              </p>
              <div>
                {section.entries.map((entry, i) => (
                  <div
                    key={i}
                    className="flex gap-3 items-baseline py-1.5 text-xs border-b border-border/50 last:border-b-0"
                  >
                    <span className="font-mono text-primary whitespace-nowrap shrink-0">
                      {entry.sig}
                    </span>
                    <span className="text-muted-foreground">{entry.desc}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
          {filtered.length === 0 && (
            <p className="text-center text-muted-foreground text-xs py-12">No results.</p>
          )}
        </div>
      </div>
    </div>
  )
}
