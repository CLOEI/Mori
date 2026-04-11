# Mori Lua API Documentation

**Version**: 2.1.0

Scripts run in a sandboxed Lua 5.4 environment on a dedicated thread per bot. Each script has access to a `Bot` object (via `getBot()`) that communicates with the bot's run loop over a channel.

---

## Global Functions

### `getBot() -> Bot`
Returns the current bot object.

### `getLocal() -> Player`
Returns the local player. Shortcut for `getBot():getLocal()`.

### `getWorld() -> World | nil`
Returns the current world snapshot, or `nil` if not in a world.

### `getInventory() -> Inventory`
Returns the bot's current inventory.

### `getPlayer(key: number | string) -> Player | nil`
Returns a player by net ID or name. Returns `nil` if not found.

### `getPlayers() -> table<Player>`
Returns all players in the current world.

### `getTile(x: number, y: number) -> Tile | nil`
Returns the tile at the given position.

### `getTiles() -> table<Tile>`
Returns all tiles in the current world.

### `getObject(oid: number) -> NetObject | nil`
Returns the dropped object with the given object ID.

### `getObjects() -> table<NetObject>`
Returns all dropped objects in the current world.

### `getNPC(id: number) -> NPC | nil`
Returns the NPC with the given ID.

### `getNPCs() -> table<NPC>`
Returns all NPCs in the current world.

### `getInfo(id: number | string) -> ItemInfo | nil`
Returns item info by ID or name. Accepts numeric ID or item name string.

### `getInfos() -> table<ItemInfo>`
Returns all items from the loaded `items.dat`.

### `getUsername() -> string`
Returns the bot's GrowID (username).

### `sleep(ms: number)`
Pauses script execution for the given number of milliseconds. Respects script stop signals.

### `read(path: string) -> string`
Reads a file from disk and returns its contents as a string. Errors if file not found.

### `write(path: string, content: string)`
Writes a string to a file on disk, overwriting if it exists.

### `append(path: string, content: string)`
Appends a string to a file on disk, creating it if it does not exist.

### `removeColor(text: string) -> string`
Strips Growtopia color codes (`` `X ``) from a string.

### `clearConsole()`
Clears the bot's console log.

---

## Event System

Events are dispatched while inside `listenEvents()`. Register handlers before calling it.

### Constants

```lua
Event.variantlist  -- 1: VariantList packet received
Event.gameupdate   -- 2: GameUpdatePacket received
Event.gamemessage  -- 3: Raw game message string received
```

### `addEvent(etype: number, fn: function)`
Registers a handler for an event type.

- `Event.variantlist` — `fn(vl: VariantList, net_id: number)`
- `Event.gameupdate`  — `fn(pkt: GameUpdatePacket)`
- `Event.gamemessage` — `fn(text: string)`

### `removeEvent(etype: number)`
Removes the handler for the given event type.

### `removeEvents()`
Removes all registered event handlers.

### `listenEvents(secs?: number)`
Blocks and dispatches events. Runs indefinitely if `secs` is omitted. Returns when time expires, `unlistenEvents()` is called, or the script is stopped.

### `unlistenEvents()`
Signals `listenEvents()` to return on its next iteration.

#### Example

```lua
addEvent(Event.variantlist, function(vl, net_id)
    local name = vl:get(0):getString()
    if name == "OnConsoleMessage" then
        print(vl:get(1):getString())
    end
end)
listenEvents(30)
```

---

## Bot

The main bot object returned by `getBot()`.

#### Properties

| Property | Type | Access | Description |
|----------|------|--------|-------------|
| `name` | string | r | Bot's GrowID |
| `status` | string | r | Current connection status (see [Status Values](#status-values)) |
| `gem_count` | number | r | Current gem balance |
| `auto_collect` | boolean | r/w | Enable/disable automatic item collection |
| `ignore_gems` | boolean | r/w | Skip gems (item ID 112) during auto-collect |
| `ignore_essences` | boolean | r/w | Skip essences (IDs 5024/5026/5028/5030) during auto-collect |
| `auto_leave_on_mod` | boolean | r/w | Leave world automatically when a moderator spawns |
| `auto_ban` | boolean | r/w | Send `/ban <name>` when any non-local player spawns |
| `collect_interval` | number | r/w | Auto-collect tick interval in milliseconds (default: 500) |
| `collect_range` | number | r/w | Auto-collect radius in tiles, 1–5 (default: 3) |
| `collect_path_check` | boolean | r/w | Skip objects with no A* path during auto-collect |
| `reconnect_interval` | number | r/w | Delay in ms before reconnecting after disconnect (0 = immediate) |
| `place_delay` | number | r/w | Delay between place/punch actions in milliseconds |
| `walk_delay` | number | r/w | Delay between walk/pathfind steps in milliseconds |

#### Methods

**Getters**

* `getWorld() -> World | nil` — Returns a snapshot of the current world, or `nil` if not in one.
* `getInventory() -> Inventory` — Returns the bot's inventory.
* `getLogin() -> Login` — Returns the bot's login info.
* `getLocal() -> Player` — Returns the local player.
* `getPing() -> number` — Returns the current ping in milliseconds.
* `isInWorld(name?: string) -> boolean` — Returns `true` if the bot is in a world. If `name` is given, checks for that specific world.
* `isInTile(x: number, y: number) -> boolean` — Returns `true` if the bot is standing on the given tile.
* `getPath(x: number, y: number) -> table<{x, y}>` — Returns the A* path to the given tile as a list of `{x, y}` nodes.

**Network**

* `connect()` — Triggers a reconnect.
* `disconnect()` — Disconnects from the server.
* `sendRaw(pkt: GameUpdatePacket)` — Sends a raw `GameUpdatePacket` to the server.
* `sendPacket(type: number, text: string)` — Sends a raw text packet of the given type.

**World Actions**

* `warp(name: string, id?: string)` — Warps to a world. `id` defaults to `""` (main door).
* `say(text: string)` — Sends a chat message.
* `leaveWorld()` — Leaves the current world.
* `respawn()` — Respawns the bot.
* `active(x: number, y: number)` — Activates/toggles the tile at the given position.
* `enter(pass?: string)` — Enters a door or world entrance with an optional password.

**Tile Actions**

* `place(x: number, y: number, item: number)` — Places an item at the given tile.
* `hit(x: number, y: number)` — Punches the tile at the given position.
* `wrench(x: number, y: number)` — Wrenches the tile at the given position.
* `wrenchPlayer(net_id: number)` — Wrenches a player by their net ID.

**Inventory**

* `wear(item_id: number)` — Equips an item.
* `unwear(item_id: number)` — Unequips an item.
* `use(item_id: number)` — Alias for `wear`.
* `drop(item_id: number, count: number)` — Drops items into the world.
* `trash(item_id: number, count: number)` — Permanently deletes items.
* `fastDrop(item_id: number, count: number)` — Drops items without the normal delay.
* `fastTrash(item_id: number, count: number)` — Trashes items without the normal delay.

**Movement**

* `moveTo(dx: number, dy: number)` — Moves relative to current tile position.
* `moveTile(x: number, y: number)` — Moves to an absolute tile position.
* `moveLeft(range?: number)` — Moves left by `range` tiles (default: 1).
* `moveRight(range?: number)` — Moves right by `range` tiles (default: 1).
* `moveUp(range?: number)` — Moves up by `range` tiles (default: 1).
* `moveDown(range?: number)` — Moves down by `range` tiles (default: 1).
* `setDirection(facing_left: boolean)` — Sets the bot's facing direction.
* `findPath(x: number, y: number)` — Starts pathfinding to the given tile (non-blocking).

**Collection**

* `collect(range: number, interval_ms: number) -> number` — Collects nearby objects within `range` pixels over `interval_ms` ms. Returns the number of objects collected.
* `collectObject(oid: number, range: number)` — Collects a specific object by OID if within `range` pixels.

**Settings**

* `setMac(mac: string)` — Sets the bot's MAC address.
* `setAutoCollect(enabled: boolean)` — Enables or disables auto-collect.
* `setIgnoreGems(enabled: boolean)` — Enables or disables gem skipping.
* `setIgnoreEssences(enabled: boolean)` — Enables or disables essence skipping.
* `setAutoLeaveOnMod(enabled: boolean)` — Enables or disables auto-leave on mod detection.
* `setAutoBan(enabled: boolean)` — Enables or disables auto-ban on player spawn.
* `stopScript()` — Stops the current script.

---

## World

A snapshot of the world at the time `getWorld()` was called.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | string | World name |
| `x` | number | Width in tiles |
| `y` | number | Height in tiles |
| `tile_count` | number | Total number of tiles |
| `version` | number | World data version |
| `tiles` | table\<Tile\> | All tiles |
| `objects` | table\<NetObject\> | All dropped objects |
| `players` | table\<Player\> | All players |
| `npcs` | table\<NPC\> | All NPCs |

#### Methods

* `getTile(x: number, y: number) -> Tile | nil` — Returns the tile at the given position.
* `getTiles() -> table<Tile>` — Returns all tiles.
* `getObject(oid: number) -> NetObject | nil` — Returns the dropped object with the given OID.
* `getObjects() -> table<NetObject>` — Returns all dropped objects.
* `getPlayer(key: number | string) -> Player | nil` — Returns a player by net ID or name (case-insensitive). Returns `nil` if not found.
* `getPlayers() -> table<Player>` — Returns all players.
* `getLocal() -> Player` — Returns the local player.
* `getNPC(id: number) -> NPC | nil` — Returns the NPC with the given ID.
* `getNPCs() -> table<NPC>` — Returns all NPCs.
* `isValidPosition(x: number, y: number) -> boolean` — Returns `true` if the position is within world bounds.
* `getTileParent(tile: Tile) -> Tile | nil` — Returns the parent tile of a child tile.
* `hasAccess(x: number, y: number) -> boolean` — Returns `true` if the bot has world lock access.

---

## Player

Represents a player in the world. Returned by `getLocal()`, `getPlayer()`, etc.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | string | Player name |
| `country` | string | Country code |
| `netid` | number | Network ID |
| `userid` | number | User ID |
| `posx` | number | X position in tile coordinates |
| `posy` | number | Y position in tile coordinates |
| `avatarFlags` | number | Avatar state flags (`mstate`) |
| `roleicon` | string | Role/title icon |

---

## Tile

Represents a single tile in the world.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `fg` / `foreground` | number | Foreground item ID |
| `bg` / `background` | number | Background item ID |
| `x` | number | Tile X position |
| `y` | number | Tile Y position |
| `flags` | number | Raw tile flags bitmask |
| `parent` | number | Parent tile index (if `HAS_PARENT` flag is set) |

#### Methods

* `hasExtra() -> boolean` — Returns `true` if the tile has extra data.
* `getExtra() -> table | nil` — Returns a table of extra tile data. Returns `nil` if none. The `type` field identifies the variant (see below).
* `canHarvest() -> boolean` — Returns `true` if the tile is a ready-to-harvest seed.
* `hasFlag(flag: number) -> boolean` — Returns `true` if the given flag bit is set.

**`getExtra()` variants**

| `type` | Extra fields |
|--------|-------------|
| `"sign"` | `label: string` |
| `"door"` | `label: string`, `flags: number` |
| `"lock"` | `settings: number`, `owner_uid: number`, `access_count: number` |
| `"seed"` | `time_passed: number`, `item_on_tree: number` |
| `"mannequin"` | `label`, `hat`, `shirt`, `pants`, `boots`, `face`, `hand`, `back`, `hair`, `neck` |
| `"weather_machine"` | `settings: number` |
| `"dice"` | `symbol: number` |
| `"unknown"` | — |

**Tile flag bits**

| Bit | Value | Name |
|-----|-------|------|
| 0 | `0x0001` | `HAS_EXTRA_DATA` |
| 1 | `0x0002` | `HAS_PARENT` |
| 2 | `0x0004` | `WAS_SPLICED` |
| 5 | `0x0020` | `FLIPPED_X` |
| 6 | `0x0040` | `IS_ON` |
| 7 | `0x0080` | `IS_OPEN_TO_PUBLIC` |
| 9 | `0x0200` | `FG_ALT_MODE` |

---

## NetObject

A dropped item in the world.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | number | Item ID |
| `x` | number | X position in pixels |
| `y` | number | Y position in pixels |
| `count` | number | Item count |
| `flags` | number | Object flags |
| `oid` | number | Unique object ID |

---

## NPC

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `type` | number | NPC type |
| `id` | number | NPC ID |
| `x` | number | X position |
| `y` | number | Y position |
| `destx` | number | Destination X |
| `desty` | number | Destination Y |
| `var` | number | Variant value |
| `unk` | number | Unknown field |

---

## Inventory

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `itemcount` | number | Number of distinct items |
| `slotcount` | number | Total inventory slot count |
| `items` | table\<InventoryItem\> | All inventory items |

#### Methods

* `getItem(id: number) -> InventoryItem | nil` — Returns the inventory item with the given ID.
* `getItems() -> table<InventoryItem>` — Returns all inventory items.
* `findItem(id: number) -> number` — Returns the amount of the given item ID, or `0` if not found.
* `getItemCount(id: number) -> number` — Alias for `findItem`.
* `canCollect(id: number) -> boolean` — Returns `true` if the bot can pick up more of the given item (not at max stack).

---

## InventoryItem

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | number | Item ID |
| `count` | number | Amount held |
| `isActive` | boolean | Whether the item slot is flagged as active |

---

## ItemInfo

Returned by `getInfo()`. Contains data from `items.dat`.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | number | Item ID |
| `name` | string | Item name |
| `action_type` | number | Action type |
| `collision_type` | number | Collision type |
| `clothing_type` | number | Clothing slot type |
| `rarity` | number | Item rarity |
| `grow_time` | number | Grow time in seconds |
| `drop_chance` | number | Drop chance |
| `texture` | string | Texture filename |
| `texture_hash` | number | Texture file hash |
| `texture_x` | number | Sprite sheet X offset |
| `texture_y` | number | Sprite sheet Y offset |
| `seed_color` | number | Seed base color (BGRA) |
| `seed_overlay_color` | number | Seed overlay color (BGRA) |
| `null_Item` | boolean | `true` if the item name contains "null" |
| `strength` | number | Hits to break (`block_health / 6`) |

#### Example

```lua
local info = getInfo(7188)
print("Hits to break: " .. info.strength)
```

---

## Login

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `mac` | string | Bot's MAC address |

---

## GameUpdatePacket

Low-level packet object for sending custom packets.

#### Constructor

```lua
local pkt = GameUpdatePacket.new()
```

#### Properties (read/write)

| Property | Type | Description |
|----------|------|-------------|
| `type` | number | Packet type |
| `object_type` | number | Object type field |
| `count1` | number | Jump count field |
| `count2` | number | Animation type field |
| `netid` | number | Net ID |
| `item` | number | Target net ID / item field |
| `flags` | number | Packet flags bitmask |
| `float_var` | number | Float variable |
| `int_data` | number | Integer data value |
| `vec_x` / `pos_x` | number | X vector component |
| `vec_y` / `pos_y` | number | Y vector component |
| `vec2_x` / `pos2_x` | number | Secondary X vector |
| `vec2_y` / `pos2_y` | number | Secondary Y vector |
| `particle_rotation` | number | Particle rotation |
| `int_x` | number | Integer X |
| `int_y` | number | Integer Y |

---

## VariantList

Received in `Event.variantlist` handlers.

#### Methods

* `get(index: number) -> Variant | nil` — Returns the variant at the given index (0-based).
* `print() -> string` — Returns all variants joined by `", "`.

---

## Variant

A single value inside a `VariantList`.

#### Methods

* `getType() -> number` — Returns the variant type: `1`=float, `2`=string, `3`=vec2, `4`=vec3, `5`=uint, `9`=int.
* `getString() -> string` — Returns the value as a string.
* `getInt() -> number` — Returns the value as a signed integer.
* `getFloat() -> number` — Returns the value as a float.
* `getVector2() -> {x: number, y: number}` — Returns the value as a 2D vector table.
* `getVector3() -> {x: number, y: number, z: number}` — Returns the value as a 3D vector table.
* `print() -> string` — Alias for `getString()`.

---

## HttpClient

HTTP client for making outbound requests.

#### Constructor

```lua
local client = HttpClient.new()
```

#### Properties (read/write)

| Property | Type | Description |
|----------|------|-------------|
| `url` | string | Request URL |
| `method` | string | HTTP method (`GET`, `POST`, etc.) |
| `content` | string | Request body |
| `headers` | table | Key-value header table (mutate directly) |

#### Methods

* `setMethod(method: string)` — Sets the HTTP method.
* `setProxy(type: number, address: string)` — Sets the proxy. `address` is `"host:port"`. Use `Proxy` enum for type.
* `removeProxy()` — Clears the proxy setting.
* `request() -> HttpResult` — Executes the request. Times out after 10 seconds.

---

## HttpResult

Returned by `HttpClient:request()`.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `body` | string | Response body |
| `status` | number | HTTP status code (0 on connection error) |
| `error` | number | Error code (0 = success, 1 = error) |

#### Methods

* `getError() -> string` — Returns the error message if the request failed.

---

## Proxy

Enum used with `HttpClient:setProxy()`.

```lua
Proxy.http   -- 1
Proxy.socks4 -- 2
Proxy.socks5 -- 3
```

---

## Webhook

Discord webhook sender.

#### Constructor

```lua
local wh = Webhook.new(url?: string)
```

#### Properties (read/write)

| Property | Type | Description |
|----------|------|-------------|
| `url` | string | Webhook URL |
| `content` | string | Message text content |
| `username` | string | Override display name |
| `avatar_url` | string | Override avatar URL |
| `embed1` | Embed | First embed |
| `embed2` | Embed | Second embed |

#### Methods

* `makeContent() -> string` — Builds and returns the JSON payload without sending.
* `send()` — Sends the webhook message.
* `edit(message_id: number)` — Edits an existing message by ID (PATCH).

---

## Embed

Accessed via `webhook.embed1` or `webhook.embed2`.

#### Properties (read/write)

| Property | Type | Description |
|----------|------|-------------|
| `use` | boolean | Whether to include this embed |
| `color` | number | Embed color as integer |
| `title` | string | Embed title |
| `type` | string | Embed type (default: `"rich"`) |
| `description` | string | Embed description |
| `url` | string | Title hyperlink URL |
| `thumbnail` | string | Thumbnail image URL |
| `image` | string | Main image URL |
| `footer` | table | `{ text?: string, icon_url?: string }` |
| `author` | table | `{ name?: string, url?: string, icon_url?: string }` |

#### Methods

* `addField(name: string, value: string, inline: boolean)` — Appends a field to the embed.

#### Example

```lua
local wh = Webhook.new("https://discord.com/api/webhooks/...")
wh.content = "Hello from Mori!"
wh.embed1.use = true
wh.embed1.title = "Status"
wh.embed1.description = "Bot is running in " .. getWorld().name
wh.embed1.color = 0x00FF00
wh.embed1:addField("World", getWorld().name, true)
wh:send()
```

---

## Reference

### Status Values

| Value | Description |
|-------|-------------|
| `connecting` | Initial state, attempting to connect |
| `connected` | Connected to game server |
| `in_game` | In-game — world select screen or inside a world |
| `two_factor_auth` | Blocked by 2FA — retries after `twofa_secs` |
| `server_overloaded` | Server overloaded — retries after `server_overload_secs` |
| `too_many_logins` | Too many concurrent logins — retries after `too_many_logins_secs` |
| `update_required` | Client update required — bot stops permanently |
| `maintenance` | Server under maintenance — retries after `maintenance_secs` |
