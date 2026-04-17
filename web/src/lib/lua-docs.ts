export type DocEntry = {
  name: string
  signature?: string
  description: string
  details?: string[]
}

export type DocTable = {
  title: string
  columns: string[]
  rows: string[][]
}

export type DocSection = {
  id: string
  title: string
  summary: string
  entries?: DocEntry[]
  tables?: DocTable[]
  example?: string
}

export const LUA_DOC_VERSION = '2.1.0'

export const LUA_DOC_SECTIONS: DocSection[] = [
  {
    id: 'global-functions',
    title: 'Global Functions',
    summary:
      'Core helpers exposed to every sandboxed Lua 5.4 script running on a dedicated bot thread.',
    entries: [
      { name: 'getBot', signature: 'getBot() -> Bot', description: 'Returns the current bot object.' },
      { name: 'getLocal', signature: 'getLocal() -> Player', description: 'Returns the local player. Shortcut for getBot():getLocal().' },
      { name: 'getWorld', signature: 'getWorld() -> World | nil', description: 'Returns a snapshot handle for the current world, or nil if not in a world.' },
      { name: 'getInventory', signature: 'getInventory() -> Inventory', description: "Returns the bot's current inventory." },
      { name: 'buy', signature: 'buy(pack_id: string)', description: 'Shortcut for getBot():buy(pack_id).' },
      { name: 'getPlayer', signature: 'getPlayer(key: number | string) -> Player | nil', description: 'Returns a player by net ID or name.' },
      { name: 'getPlayers', signature: 'getPlayers() -> table<Player>', description: 'Returns all players in the current world.' },
      { name: 'getTile', signature: 'getTile(x: number, y: number) -> Tile | nil', description: 'Returns the tile at the given position.' },
      { name: 'getTiles', signature: 'getTiles() -> table<Tile>', description: 'Returns all tiles in the current world.' },
      { name: 'getObject', signature: 'getObject(oid: number) -> NetObject | nil', description: 'Returns the dropped object with the given object ID.' },
      { name: 'getObjects', signature: 'getObjects() -> table<NetObject>', description: 'Returns all dropped objects in the current world.' },
      { name: 'getNPC', signature: 'getNPC(id: number) -> NPC | nil', description: 'Returns the NPC with the given ID.' },
      { name: 'getNPCs', signature: 'getNPCs() -> table<NPC>', description: 'Returns all NPCs in the current world.' },
      { name: 'getInfo', signature: 'getInfo(id: number | string) -> ItemInfo | nil', description: 'Returns item info by numeric ID or item name string.' },
      { name: 'getInfos', signature: 'getInfos() -> table<ItemInfo>', description: 'Returns all items from the loaded items.dat.' },
      { name: 'getUsername', signature: 'getUsername() -> string', description: "Returns the bot's GrowID." },
      { name: 'sleep', signature: 'sleep(ms: number)', description: 'Pauses script execution for the given number of milliseconds.', details: ['Respects script stop signals.'] },
      { name: 'read', signature: 'read(path: string) -> string', description: 'Reads a file from disk and returns its contents as a string.' },
      { name: 'write', signature: 'write(path: string, content: string)', description: 'Writes a string to a file on disk, overwriting if it exists.' },
      { name: 'append', signature: 'append(path: string, content: string)', description: 'Appends a string to a file on disk, creating it if it does not exist.' },
      { name: 'removeColor', signature: 'removeColor(text: string) -> string', description: 'Strips Growtopia color codes from a string.' },
      { name: 'clearConsole', signature: 'clearConsole()', description: "Clears the bot's console log." },
      { name: 'packs', signature: 'packs: string', description: 'Raw store-pack constant in Name:PackID:Gems:Items format.' },
      { name: 'PackDB', signature: 'PackDB: table<string, table>', description: 'Parsed store-pack lookup keyed by pack_id.' },
      { name: 'PackNames', signature: 'PackNames: table<string, table>', description: 'Parsed store-pack lookup keyed by display name.' },
    ],
  },
  {
    id: 'event-system',
    title: 'Event System',
    summary:
      'Handlers are dispatched only while inside listenEvents(). Register callbacks before entering the loop.',
    entries: [
      { name: 'addEvent', signature: 'addEvent(etype: number, fn: function)', description: 'Registers a handler for an event type.', details: ['Event.variantlist -> fn(vl: VariantList, net_id: number)', 'Event.gameupdate -> fn(pkt: GameUpdatePacket)', 'Event.gamemessage -> fn(text: string)'] },
      { name: 'removeEvent', signature: 'removeEvent(etype: number)', description: 'Removes the handler for the given event type.' },
      { name: 'removeEvents', signature: 'removeEvents()', description: 'Removes all registered event handlers.' },
      { name: 'listenEvents', signature: 'listenEvents(secs?: number)', description: 'Blocks and dispatches events.', details: ['Runs indefinitely if secs is omitted.', 'Returns when time expires, unlistenEvents() is called, or the script is stopped.'] },
      { name: 'unlistenEvents', signature: 'unlistenEvents()', description: 'Signals listenEvents() to return on its next iteration.' },
    ],
    tables: [
      {
        title: 'Constants',
        columns: ['Name', 'Value', 'Description'],
        rows: [
          ['Event.variantlist', '1', 'VariantList packet received'],
          ['Event.gameupdate', '2', 'GameUpdatePacket received'],
          ['Event.gamemessage', '3', 'Raw game message string received'],
        ],
      },
    ],
    example: `addEvent(Event.variantlist, function(vl, net_id)
    local name = vl:get(0):getString()
    if name == "OnConsoleMessage" then
        print(vl:get(1):getString())
    end
end)
listenEvents(30)`,
  },
  {
    id: 'bot',
    title: 'Bot',
    summary:
      'The main bot object returned by getBot(), covering world state, networking, movement, collection, and runtime settings.',
    entries: [
      { name: 'getWorld', signature: 'getWorld() -> World | nil', description: 'Returns a snapshot handle for the current world, or nil if not in one.' },
      { name: 'getInventory', signature: 'getInventory() -> Inventory', description: "Returns the bot's inventory." },
      { name: 'getLogin', signature: 'getLogin() -> Login', description: "Returns the bot's login info." },
      { name: 'getLocal', signature: 'getLocal() -> Player', description: 'Returns the local player.' },
      { name: 'getPing', signature: 'getPing() -> number', description: 'Returns the current ping in milliseconds.' },
      { name: 'isInWorld', signature: 'isInWorld(name?: string) -> boolean', description: 'Returns true if the bot is in a world.' },
      { name: 'isInTile', signature: 'isInTile(x: number, y: number) -> boolean', description: 'Returns true if the bot is standing on the given tile.' },
      { name: 'getPath', signature: 'getPath(x: number, y: number) -> table<{x, y}>', description: 'Returns the A* path to the given tile as a list of nodes.' },
      { name: 'connect', signature: 'connect()', description: 'Triggers a reconnect.' },
      { name: 'disconnect', signature: 'disconnect()', description: 'Disconnects from the server.' },
      { name: 'sendRaw', signature: 'sendRaw(pkt: GameUpdatePacket)', description: 'Sends a raw GameUpdatePacket to the server.' },
      { name: 'sendPacket', signature: 'sendPacket(type: number, text: string)', description: 'Sends a raw text packet of the given type.' },
      { name: 'warp', signature: 'warp(name: string, id?: string)', description: 'Warps to a world.', details: ['id defaults to "" for the main door.'] },
      { name: 'say', signature: 'say(text: string)', description: 'Sends a chat message.' },
      { name: 'leaveWorld', signature: 'leaveWorld()', description: 'Leaves the current world.' },
      { name: 'respawn', signature: 'respawn()', description: 'Respawns the bot.' },
      { name: 'active', signature: 'active(x: number, y: number)', description: 'Activates or toggles the tile at the given position.' },
      { name: 'enter', signature: 'enter(pass?: string)', description: 'Enters a door or world entrance with an optional password.' },
      { name: 'place', signature: 'place(x: number, y: number, item: number)', description: 'Places an item at the given tile.' },
      { name: 'hit', signature: 'hit(x: number, y: number)', description: 'Punches the tile at the given position.' },
      { name: 'wrench', signature: 'wrench(x: number, y: number)', description: 'Wrenches the tile at the given position.' },
      { name: 'wrenchPlayer', signature: 'wrenchPlayer(net_id: number)', description: 'Wrenches a player by their net ID.' },
      { name: 'wear', signature: 'wear(item_id: number)', description: 'Equips an item.' },
      { name: 'unwear', signature: 'unwear(item_id: number)', description: 'Unequips an item.' },
      { name: 'use', signature: 'use(item_id: number)', description: 'Alias for wear.' },
      { name: 'drop', signature: 'drop(item_id: number, count: number)', description: 'Drops items into the world.' },
      { name: 'trash', signature: 'trash(item_id: number, count: number)', description: 'Permanently deletes items.' },
      { name: 'fastDrop', signature: 'fastDrop(item_id: number, count: number)', description: 'Drops items without the normal delay.' },
      { name: 'fastTrash', signature: 'fastTrash(item_id: number, count: number)', description: 'Trashes items without the normal delay.' },
      { name: 'buy', signature: 'buy(pack_id: string)', description: 'Buys a store item by its store pack ID.', details: ['Example: small_lock'] },
      { name: 'moveTo', signature: 'moveTo(dx: number, dy: number)', description: 'Moves relative to the current tile position.' },
      { name: 'moveTile', signature: 'moveTile(x: number, y: number)', description: 'Moves to an absolute tile position.' },
      { name: 'moveLeft', signature: 'moveLeft(range?: number)', description: 'Moves left by range tiles.', details: ['Default: 1'] },
      { name: 'moveRight', signature: 'moveRight(range?: number)', description: 'Moves right by range tiles.', details: ['Default: 1'] },
      { name: 'moveUp', signature: 'moveUp(range?: number)', description: 'Moves up by range tiles.', details: ['Default: 1'] },
      { name: 'moveDown', signature: 'moveDown(range?: number)', description: 'Moves down by range tiles.', details: ['Default: 1'] },
      { name: 'setDirection', signature: 'setDirection(facing_left: boolean)', description: "Sets the bot's facing direction." },
      { name: 'findPath', signature: 'findPath(x: number, y: number)', description: 'Starts pathfinding to the given tile.', details: ['Non-blocking.'] },
      { name: 'collect', signature: 'collect(range: number, interval_ms: number) -> number', description: 'Collects nearby objects within range pixels over interval_ms milliseconds.', details: ['Returns the number of objects collected.'] },
      { name: 'collectObject', signature: 'collectObject(oid: number, range: number)', description: 'Collects a specific object by OID if it is within range pixels.' },
      { name: 'setMac', signature: 'setMac(mac: string)', description: "Sets the bot's MAC address." },
      { name: 'setAutoCollect', signature: 'setAutoCollect(enabled: boolean)', description: 'Enables or disables auto-collect.' },
      { name: 'setIgnoreGems', signature: 'setIgnoreGems(enabled: boolean)', description: 'Enables or disables gem skipping.' },
      { name: 'setIgnoreEssences', signature: 'setIgnoreEssences(enabled: boolean)', description: 'Enables or disables essence skipping.' },
      { name: 'setAutoLeaveOnMod', signature: 'setAutoLeaveOnMod(enabled: boolean)', description: 'Enables or disables auto-leave on moderator detection.' },
      { name: 'setAutoBan', signature: 'setAutoBan(enabled: boolean)', description: 'Enables or disables auto-ban on player spawn.' },
      { name: 'stopScript', signature: 'stopScript()', description: 'Stops the current script.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Access', 'Description'],
        rows: [
          ['name', 'string', 'r', "Bot's GrowID"],
          ['status', 'string', 'r', 'Current connection status'],
          ['gem_count', 'number', 'r', 'Current gem balance'],
          ['auto_collect', 'boolean', 'r/w', 'Enable or disable automatic item collection'],
          ['ignore_gems', 'boolean', 'r/w', 'Skip gems (item ID 112) during auto-collect'],
          ['ignore_essences', 'boolean', 'r/w', 'Skip essences (IDs 5024/5026/5028/5030) during auto-collect'],
          ['auto_leave_on_mod', 'boolean', 'r/w', 'Leave world automatically when a moderator spawns'],
          ['auto_ban', 'boolean', 'r/w', 'Send /ban <name> when any non-local player spawns'],
          ['collect_interval', 'number', 'r/w', 'Auto-collect tick interval in milliseconds (default: 500)'],
          ['collect_range', 'number', 'r/w', 'Auto-collect radius in tiles, 1-5 (default: 3)'],
          ['collect_path_check', 'boolean', 'r/w', 'Skip objects with no A* path during auto-collect'],
          ['reconnect_interval', 'number', 'r/w', 'Delay in ms before reconnecting after disconnect (0 = immediate)'],
          ['place_delay', 'number', 'r/w', 'Delay between place and punch actions in milliseconds'],
          ['walk_delay', 'number', 'r/w', 'Delay between walk and pathfind steps in milliseconds'],
        ],
      },
    ],
  },
  {
    id: 'world',
    title: 'World',
    summary: 'A snapshot handle of the world at the time getWorld() was called.',
    entries: [
      { name: 'getTile', signature: 'getTile(x: number, y: number) -> Tile | nil', description: 'Returns the tile at the given position.' },
      { name: 'getTiles', signature: 'getTiles() -> table<Tile>', description: 'Returns all tiles. Bulk access is heavier than getTile(x, y).' },
      { name: 'getObject', signature: 'getObject(oid: number) -> NetObject | nil', description: 'Returns the dropped object with the given OID.' },
      { name: 'getObjects', signature: 'getObjects() -> table<NetObject>', description: 'Returns all dropped objects. Bulk access is heavier than getObject(oid).' },
      { name: 'getPlayer', signature: 'getPlayer(key: number | string) -> Player | nil', description: 'Returns a player by net ID or case-insensitive name.' },
      { name: 'getPlayers', signature: 'getPlayers() -> table<Player>', description: 'Returns all players.' },
      { name: 'getLocal', signature: 'getLocal() -> Player', description: 'Returns the local player.' },
      { name: 'getNPC', signature: 'getNPC(id: number) -> NPC | nil', description: 'Returns the NPC with the given ID.' },
      { name: 'getNPCs', signature: 'getNPCs() -> table<NPC>', description: 'Returns all NPCs.' },
      { name: 'isValidPosition', signature: 'isValidPosition(x: number, y: number) -> boolean', description: 'Returns true if the position is within world bounds.' },
      { name: 'getTileParent', signature: 'getTileParent(tile: Tile) -> Tile | nil', description: 'Returns the parent tile of a child tile.' },
      { name: 'hasAccess', signature: 'hasAccess(x: number, y: number) -> boolean', description: 'Returns true if the bot has world lock access.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['name', 'string', 'World name'],
          ['x', 'number', 'Width in tiles'],
          ['y', 'number', 'Height in tiles'],
          ['tile_count', 'number', 'Total number of tiles'],
          ['version', 'number', 'World data version'],
          ['tiles', 'table<Tile>', 'All tiles.'],
          ['objects', 'table<NetObject>', 'All dropped objects.'],
          ['players', 'table<Player>', 'All players.'],
          ['npcs', 'table<NPC>', 'All NPCs.'],
        ],
      },
    ],
  },
  {
    id: 'player',
    title: 'Player',
    summary: 'Represents a player in the world. Returned by getLocal(), getPlayer(), and related helpers.',
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['name', 'string', 'Player name'],
          ['country', 'string', 'Country code'],
          ['netid', 'number', 'Network ID'],
          ['userid', 'number', 'User ID'],
          ['posx', 'number', 'X position in tile coordinates'],
          ['posy', 'number', 'Y position in tile coordinates'],
          ['avatarFlags', 'number', 'Avatar state flags (mstate)'],
          ['roleicon', 'string', 'Role or title icon'],
        ],
      },
    ],
  },
  {
    id: 'tile',
    title: 'Tile',
    summary: 'Represents a single tile in the world, including extra data and flag helpers.',
    entries: [
      { name: 'hasExtra', signature: 'hasExtra() -> boolean', description: 'Returns true if the tile has extra data.' },
      { name: 'getExtra', signature: 'getExtra() -> table | nil', description: 'Returns a table of extra tile data, or nil if none.', details: ['The type field identifies the variant.'] },
      { name: 'canHarvest', signature: 'canHarvest() -> boolean', description: 'Returns true if the tile is a ready-to-harvest seed.' },
      { name: 'hasFlag', signature: 'hasFlag(flag: number) -> boolean', description: 'Returns true if the given flag bit is set.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['fg / foreground', 'number', 'Foreground item ID'],
          ['bg / background', 'number', 'Background item ID'],
          ['x', 'number', 'Tile X position'],
          ['y', 'number', 'Tile Y position'],
          ['flags', 'number', 'Raw tile flags bitmask'],
          ['parent', 'number', 'Parent tile index when HAS_PARENT is set'],
        ],
      },
      {
        title: 'getExtra() Variants',
        columns: ['type', 'Extra fields'],
        rows: [
          ['sign', 'label: string'],
          ['door', 'label: string, flags: number'],
          ['lock', 'settings: number, owner_uid: number, access_count: number'],
          ['seed', 'time_passed: number, item_on_tree: number'],
          ['mannequin', 'label, hat, shirt, pants, boots, face, hand, back, hair, neck'],
          ['weather_machine', 'settings: number'],
          ['dice', 'symbol: number'],
          ['unknown', 'No documented extra fields'],
        ],
      },
      {
        title: 'Tile Flag Bits',
        columns: ['Bit', 'Value', 'Name'],
        rows: [
          ['0', '0x0001', 'HAS_EXTRA_DATA'],
          ['1', '0x0002', 'HAS_PARENT'],
          ['2', '0x0004', 'WAS_SPLICED'],
          ['5', '0x0020', 'FLIPPED_X'],
          ['6', '0x0040', 'IS_ON'],
          ['7', '0x0080', 'IS_OPEN_TO_PUBLIC'],
          ['9', '0x0200', 'FG_ALT_MODE'],
        ],
      },
    ],
  },
  {
    id: 'netobject',
    title: 'NetObject',
    summary: 'Represents a dropped item in the world.',
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['id', 'number', 'Item ID'],
          ['x', 'number', 'X position in pixels'],
          ['y', 'number', 'Y position in pixels'],
          ['count', 'number', 'Item count'],
          ['flags', 'number', 'Object flags'],
          ['oid', 'number', 'Unique object ID'],
        ],
      },
    ],
  },
  {
    id: 'npc',
    title: 'NPC',
    summary: 'Represents a non-player character in the current world.',
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['type', 'number', 'NPC type'],
          ['id', 'number', 'NPC ID'],
          ['x', 'number', 'X position'],
          ['y', 'number', 'Y position'],
          ['destx', 'number', 'Destination X'],
          ['desty', 'number', 'Destination Y'],
          ['var', 'number', 'Variant value'],
          ['unk', 'number', 'Unknown field'],
        ],
      },
    ],
  },
  {
    id: 'inventory',
    title: 'Inventory',
    summary: 'The bot inventory snapshot, including lookup and collection helpers.',
    entries: [
      { name: 'getItem', signature: 'getItem(id: number) -> InventoryItem | nil', description: 'Returns the inventory item with the given ID.' },
      { name: 'getItems', signature: 'getItems() -> table<InventoryItem>', description: 'Returns all inventory items.' },
      { name: 'findItem', signature: 'findItem(id: number) -> number', description: 'Returns the amount of the given item ID, or 0 if not found.' },
      { name: 'getItemCount', signature: 'getItemCount(id: number) -> number', description: 'Alias for findItem.' },
      { name: 'canCollect', signature: 'canCollect(id: number) -> boolean', description: 'Returns true if the bot can pick up more of the given item.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['itemcount', 'number', 'Number of distinct items'],
          ['slotcount', 'number', 'Total inventory slot count'],
          ['items', 'table<InventoryItem>', 'All inventory items'],
        ],
      },
    ],
  },
  {
    id: 'inventory-item',
    title: 'InventoryItem',
    summary: 'Represents a single item stack in inventory.',
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['id', 'number', 'Item ID'],
          ['count', 'number', 'Amount held'],
          ['isActive', 'boolean', 'Whether the item slot is flagged as active'],
        ],
      },
    ],
  },
  {
    id: 'item-info',
    title: 'ItemInfo',
    summary: 'Returned by getInfo() and backed by data from items.dat.',
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['id', 'number', 'Item ID'],
          ['name', 'string', 'Item name'],
          ['action_type', 'number', 'Action type'],
          ['collision_type', 'number', 'Collision type'],
          ['clothing_type', 'number', 'Clothing slot type'],
          ['rarity', 'number', 'Item rarity'],
          ['grow_time', 'number', 'Grow time in seconds'],
          ['drop_chance', 'number', 'Drop chance'],
          ['texture', 'string', 'Texture filename'],
          ['texture_hash', 'number', 'Texture file hash'],
          ['texture_x', 'number', 'Sprite sheet X offset'],
          ['texture_y', 'number', 'Sprite sheet Y offset'],
          ['seed_color', 'number', 'Seed base color (BGRA)'],
          ['seed_overlay_color', 'number', 'Seed overlay color (BGRA)'],
          ['null_Item', 'boolean', 'true if the item name contains "null"'],
          ['strength', 'number', 'Hits to break (block_health / 6)'],
        ],
      },
    ],
    example: `local info = getInfo(7188)
print("Hits to break: " .. info.strength)`,
  },
  {
    id: 'login',
    title: 'Login',
    summary: "Contains the bot's login metadata.",
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [['mac', 'string', "Bot's MAC address"]],
      },
    ],
  },
  {
    id: 'game-update-packet',
    title: 'GameUpdatePacket',
    summary: 'Low-level packet object for sending custom packets.',
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['type', 'number', 'Packet type'],
          ['object_type', 'number', 'Object type field'],
          ['count1', 'number', 'Jump count field'],
          ['count2', 'number', 'Animation type field'],
          ['netid', 'number', 'Net ID'],
          ['item', 'number', 'Target net ID or item field'],
          ['flags', 'number', 'Packet flags bitmask'],
          ['float_var', 'number', 'Float variable'],
          ['int_data', 'number', 'Integer data value'],
          ['vec_x / pos_x', 'number', 'X vector component'],
          ['vec_y / pos_y', 'number', 'Y vector component'],
          ['vec2_x / pos2_x', 'number', 'Secondary X vector'],
          ['vec2_y / pos2_y', 'number', 'Secondary Y vector'],
          ['particle_rotation', 'number', 'Particle rotation'],
          ['int_x', 'number', 'Integer X'],
          ['int_y', 'number', 'Integer Y'],
        ],
      },
    ],
    example: `local pkt = GameUpdatePacket.new()`,
  },
  {
    id: 'variant-list',
    title: 'VariantList',
    summary: 'Object received in Event.variantlist handlers.',
    entries: [
      { name: 'get', signature: 'get(index: number) -> Variant | nil', description: 'Returns the variant at the given 0-based index.' },
      { name: 'print', signature: 'print() -> string', description: 'Returns all variants joined by ", ".' },
    ],
  },
  {
    id: 'variant',
    title: 'Variant',
    summary: 'A single value inside a VariantList.',
    entries: [
      { name: 'getType', signature: 'getType() -> number', description: 'Returns the variant type.', details: ['1 = float', '2 = string', '3 = vec2', '4 = vec3', '5 = uint', '9 = int'] },
      { name: 'getString', signature: 'getString() -> string', description: 'Returns the value as a string.' },
      { name: 'getInt', signature: 'getInt() -> number', description: 'Returns the value as a signed integer.' },
      { name: 'getFloat', signature: 'getFloat() -> number', description: 'Returns the value as a float.' },
      { name: 'getVector2', signature: 'getVector2() -> {x: number, y: number}', description: 'Returns the value as a 2D vector table.' },
      { name: 'getVector3', signature: 'getVector3() -> {x: number, y: number, z: number}', description: 'Returns the value as a 3D vector table.' },
      { name: 'print', signature: 'print() -> string', description: 'Alias for getString().' },
    ],
  },
  {
    id: 'http-client',
    title: 'HttpClient',
    summary: 'HTTP client for making outbound requests from Lua scripts.',
    entries: [
      { name: 'setMethod', signature: 'setMethod(method: string)', description: 'Sets the HTTP method.' },
      { name: 'setProxy', signature: 'setProxy(type: number, address: string)', description: 'Sets the proxy.', details: ['address is "host:port".', 'Use the Proxy enum for type.'] },
      { name: 'removeProxy', signature: 'removeProxy()', description: 'Clears the proxy setting.' },
      { name: 'request', signature: 'request() -> HttpResult', description: 'Executes the request.', details: ['Times out after 10 seconds.'] },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['url', 'string', 'Request URL'],
          ['method', 'string', 'HTTP method such as GET or POST'],
          ['content', 'string', 'Request body'],
          ['headers', 'table', 'Key-value header table; mutate directly'],
        ],
      },
    ],
    example: `local client = HttpClient.new()`,
  },
  {
    id: 'http-result',
    title: 'HttpResult',
    summary: 'Returned by HttpClient:request().',
    entries: [
      { name: 'getError', signature: 'getError() -> string', description: 'Returns the error message if the request failed.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['body', 'string', 'Response body'],
          ['status', 'number', 'HTTP status code (0 on connection error)'],
          ['error', 'number', 'Error code (0 = success, 1 = error)'],
        ],
      },
    ],
  },
  {
    id: 'proxy-enum',
    title: 'Proxy',
    summary: 'Enum used with HttpClient:setProxy().',
    tables: [
      {
        title: 'Values',
        columns: ['Name', 'Value'],
        rows: [
          ['Proxy.http', '1'],
          ['Proxy.socks4', '2'],
          ['Proxy.socks5', '3'],
        ],
      },
    ],
  },
  {
    id: 'webhook',
    title: 'Webhook',
    summary: 'Discord webhook sender with content and embed support.',
    entries: [
      { name: 'makeContent', signature: 'makeContent() -> string', description: 'Builds and returns the JSON payload without sending.' },
      { name: 'send', signature: 'send()', description: 'Sends the webhook message.' },
      { name: 'edit', signature: 'edit(message_id: number)', description: 'Edits an existing message by ID using PATCH.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['url', 'string', 'Webhook URL'],
          ['content', 'string', 'Message text content'],
          ['username', 'string', 'Override display name'],
          ['avatar_url', 'string', 'Override avatar URL'],
          ['embed1', 'Embed', 'First embed'],
          ['embed2', 'Embed', 'Second embed'],
        ],
      },
    ],
    example: `local wh = Webhook.new("https://discord.com/api/webhooks/...")
wh.content = "Hello from Mori!"
wh.embed1.use = true
wh.embed1.title = "Status"
wh.embed1.description = "Bot is running in " .. getWorld().name
wh.embed1.color = 0x00FF00
wh.embed1:addField("World", getWorld().name, true)
wh:send()`,
  },
  {
    id: 'embed',
    title: 'Embed',
    summary: 'Accessed via webhook.embed1 or webhook.embed2.',
    entries: [
      { name: 'addField', signature: 'addField(name: string, value: string, inline: boolean)', description: 'Appends a field to the embed.' },
    ],
    tables: [
      {
        title: 'Properties',
        columns: ['Property', 'Type', 'Description'],
        rows: [
          ['use', 'boolean', 'Whether to include this embed'],
          ['color', 'number', 'Embed color as integer'],
          ['title', 'string', 'Embed title'],
          ['type', 'string', 'Embed type (default: "rich")'],
          ['description', 'string', 'Embed description'],
          ['url', 'string', 'Title hyperlink URL'],
          ['thumbnail', 'string', 'Thumbnail image URL'],
          ['image', 'string', 'Main image URL'],
          ['footer', 'table', '{ text?: string, icon_url?: string }'],
          ['author', 'table', '{ name?: string, url?: string, icon_url?: string }'],
        ],
      },
    ],
  },
  {
    id: 'reference',
    title: 'Reference',
    summary: 'Status values and other quick lookups used throughout the API.',
    tables: [
      {
        title: 'Status Values',
        columns: ['Value', 'Description'],
        rows: [
          ['connecting', 'Initial state, attempting to connect'],
          ['connected', 'Connected to game server'],
          ['in_game', 'In-game, at world select or inside a world'],
          ['two_factor_auth', 'Blocked by 2FA; retries after twofa_secs'],
          ['server_overloaded', 'Server overloaded; retries after server_overload_secs'],
          ['too_many_logins', 'Too many concurrent logins; retries after too_many_logins_secs'],
          ['update_required', 'Client update required; bot stops permanently'],
          ['maintenance', 'Server under maintenance; retries after maintenance_secs'],
        ],
      },
    ],
  },
]
