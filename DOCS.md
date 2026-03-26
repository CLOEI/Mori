# Mori API & WebSocket Documentation

**Version**: 2.0.0
**Base URL**: `http://localhost:3000`
**WebSocket**: `ws://localhost:3000/ws`

---

## HTTP API

All endpoints return `application/json`. No authentication is required.

---

### GET `/`

Serves the HTML dashboard.

---

### GET `/bots`

Returns a summary list of all running bots.

**Response**
```json
[
  {
    "id": 1,
    "username": "string",
    "status": "in_world",
    "world": "string",
    "pos_x": 0.0,
    "pos_y": 0.0,
    "gems": 0,
    "ping_ms": 0
  }
]
```

---

### POST `/bots`

Spawns a new bot.

**Request Body**
```json
{
  "username": "string",
  "password": "string",
  "proxy_host": "string",
  "proxy_port": 1080,
  "proxy_username": "string",
  "proxy_password": "string"
}
```

`proxy_host` and `proxy_port` are required together to enable SOCKS5 proxy. Username/password are optional.

**Response**
```json
{ "id": 1 }
```

---

### DELETE `/bots/{id}`

Stops and removes a bot.

| Status | Meaning |
|--------|---------|
| `204` | Stopped successfully |
| `404` | Bot not found |

---

### GET `/bots/{id}/state`

Returns the full state of a bot.

**Response**
```json
{
  "status": "in_world",
  "world_name": "string",
  "pos_x": 0.0,
  "pos_y": 0.0,
  "world_width": 100,
  "world_height": 60,
  "tiles": [
    { "fg_item_id": 0, "bg_item_id": 0 }
  ],
  "players": [
    {
      "net_id": 0,
      "name": "string",
      "pos_x": 0.0,
      "pos_y": 0.0,
      "country": "us"
    }
  ],
  "inventory": [
    {
      "item_id": 0,
      "amount": 0,
      "is_active": false,
      "action_type": 0
    }
  ],
  "gems": 0,
  "console": ["string"],
  "ping_ms": 0,
  "delays": {
    "place_ms": 500,
    "walk_ms": 500
  }
}
```

| Status | Meaning |
|--------|---------|
| `200` | OK |
| `404` | Bot not found |

---

### POST `/bots/{id}/cmd`

Sends a command to a bot.

| Status | Meaning |
|--------|---------|
| `204` | Command sent |
| `404` | Bot not found |

All commands use a tagged union with a `"type"` field.

#### `move`
Move the bot to a pixel position.
```json
{ "type": "move", "x": 0.0, "y": 0.0 }
```

#### `walk_to`
Pathfind to a tile position.
```json
{ "type": "walk_to", "x": 0, "y": 0 }
```

#### `run_script`
Run a Lua script on the bot.
```json
{ "type": "run_script", "content": "string" }
```

#### `stop_script`
Stop the currently running script.
```json
{ "type": "stop_script" }
```

#### `wear`
Equip an item.
```json
{ "type": "wear", "item_id": 0 }
```

#### `unwear`
Unequip an item.
```json
{ "type": "unwear", "item_id": 0 }
```

#### `drop`
Drop items into the world.
```json
{ "type": "drop", "item_id": 0, "count": 1 }
```

#### `trash`
Permanently delete items.
```json
{ "type": "trash", "item_id": 0, "count": 1 }
```

#### `set_delays`
Configure action delays in milliseconds.
```json
{ "type": "set_delays", "place_ms": 500, "walk_ms": 500 }
```

---

### GET `/items`

Paginated search through the item database.

**Query Parameters**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `page` | integer | `1` | Page number (1-indexed) |
| `q` | string | `""` | Search by item ID (exact) or name (substring, case-insensitive) |

**Response**
```json
{
  "items": [
    {
      "id": 0,
      "name": "string",
      "flags": 0,
      "action_type": 0,
      "material": 0,
      "texture_file_name": "string",
      "texture_hash": 0,
      "visual_effect": 0,
      "collision_type": 0,
      "rarity": 0,
      "max_item": 0,
      "grow_time": 0,
      "base_color": 0,
      "overlay_color": 0,
      "clothing_type": 0
    }
  ],
  "total": 0,
  "page": 1,
  "page_size": 50
}
```

50 items per page.

---

## WebSocket

Connect to `ws://localhost:3000/ws`.

All messages are JSON text frames with the format:
```json
{ "event": "EventName", "data": { ... } }
```

The server only sends messages; the client does not send any.

---

### Events

#### `BotAdded`
A new bot was spawned.
```json
{
  "event": "BotAdded",
  "data": {
    "bot_id": 1,
    "username": "string"
  }
}
```

#### `BotRemoved`
A bot was stopped.
```json
{
  "event": "BotRemoved",
  "data": { "bot_id": 1 }
}
```

#### `BotStatus`
Bot connection status changed.
```json
{
  "event": "BotStatus",
  "data": {
    "bot_id": 1,
    "status": "in_world"
  }
}
```

See [BotStatus values](#botstatus-values) below.

#### `BotWorld`
Bot entered or left a world. `world_name` is an empty string when leaving.
```json
{
  "event": "BotWorld",
  "data": {
    "bot_id": 1,
    "world_name": "string"
  }
}
```

#### `BotMove`
Bot position updated.
```json
{
  "event": "BotMove",
  "data": {
    "bot_id": 1,
    "x": 0.0,
    "y": 0.0
  }
}
```

#### `BotGems`
Bot gem balance changed.
```json
{
  "event": "BotGems",
  "data": {
    "bot_id": 1,
    "gems": 0
  }
}
```

#### `BotPing`
Bot ping updated. Fired when the value changes.
```json
{
  "event": "BotPing",
  "data": {
    "bot_id": 1,
    "ping_ms": 0
  }
}
```

#### `BotTrackInfo`
Account info received on login.
```json
{
  "event": "BotTrackInfo",
  "data": {
    "bot_id": 1,
    "level": 0,
    "grow_id": 0,
    "install_date": 0,
    "global_playtime": 0,
    "awesomeness": 0
  }
}
```

#### `PlayerSpawn`
A player appeared in the bot's world.
```json
{
  "event": "PlayerSpawn",
  "data": {
    "bot_id": 1,
    "net_id": 0,
    "name": "string",
    "country": "us",
    "x": 0.0,
    "y": 0.0
  }
}
```

#### `PlayerMove`
A player moved.
```json
{
  "event": "PlayerMove",
  "data": {
    "bot_id": 1,
    "net_id": 0,
    "x": 0.0,
    "y": 0.0
  }
}
```

#### `PlayerLeave`
A player left the bot's world.
```json
{
  "event": "PlayerLeave",
  "data": {
    "bot_id": 1,
    "net_id": 0
  }
}
```

#### `WorldLoaded`
Full world data sent once when the bot enters a world.
```json
{
  "event": "WorldLoaded",
  "data": {
    "bot_id": 1,
    "name": "string",
    "width": 100,
    "height": 60,
    "tiles": [
      [2, 8],
      [0, 0]
    ]
  }
}
```

`tiles` is a flat array of `[fg_item_id, bg_item_id]` pairs, row-major.

#### `TileUpdate`
A single tile was modified.
```json
{
  "event": "TileUpdate",
  "data": {
    "bot_id": 1,
    "x": 0,
    "y": 0,
    "fg": 0,
    "bg": 0
  }
}
```

#### `ObjectsUpdate`
Full set of dropped objects in the world.
```json
{
  "event": "ObjectsUpdate",
  "data": {
    "bot_id": 1,
    "objects": [
      {
        "uid": 0,
        "item_id": 0,
        "x": 0.0,
        "y": 0.0,
        "count": 1
      }
    ]
  }
}
```

#### `InventoryUpdate`
Bot inventory changed.
```json
{
  "event": "InventoryUpdate",
  "data": {
    "bot_id": 1,
    "gems": 0,
    "items": [
      {
        "item_id": 0,
        "amount": 0,
        "is_active": false,
        "action_type": 0
      }
    ]
  }
}
```

#### `Console`
A console message was received (game chat, script output, etc.).
```json
{
  "event": "Console",
  "data": {
    "bot_id": 1,
    "message": "string"
  }
}
```

---

## Reference

### BotStatus Values

| Value | Description |
|-------|-------------|
| `connecting` | Initial state, attempting to connect |
| `connected` | Connected to game server |
| `in_world` | Logged in and inside a world |
| `two_factor_auth` | Blocked by 2FA — retries after 120s |
| `server_overloaded` | Server overloaded — retries after 30s |

### Coordinates

All `x`/`y` values are in **tile coordinates** (pixels ÷ 32). The bot's position `(5.0, 10.0)` means tile column 5, row 10.

### Default Delays

| Delay | Default |
|-------|---------|
| `place_ms` | 500ms |
| `walk_ms` | 500ms |
