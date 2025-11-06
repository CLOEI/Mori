-- Example Lua script demonstrating world and inventory access

local bot = getBot()

-- Access world data
local world = bot:world()
print("World name: " .. world.name)
print("World size: " .. world.width .. "x" .. world.height)
print("Total tiles: " .. #world.tiles)

-- Iterate through dropped items
print("\nDropped items in world:")
for i, item in ipairs(world.floating) do
    print(string.format("  Item #%d: ID=%d, Position=(%.1f, %.1f), Count=%d, UID=%d",
        i, item.id, item.x, item.y, item.count, item.uid))
end

-- Check specific tiles
print("\nSample tiles:")
for i = 1, math.min(10, #world.tiles) do
    local tile = world.tiles[i]
    print(string.format("  Tile %d: Foreground=%d, Background=%d", i, tile.fg, tile.bg))
end

-- Access inventory data
local inv = bot:inventory()
print("\nInventory:")
print("  Size: " .. inv.size)
print("  Gems: " .. inv.gems)
print("  Items in inventory: " .. #inv.items)

-- Iterate through inventory items
print("\nInventory items:")
for i, item in ipairs(inv.items) do
    print(string.format("  Item #%d: ID=%d, Amount=%d", i, item.id, item.amount))
end

-- Example: Find a specific item in inventory
local function hasItem(itemId, minAmount)
    local inv = bot:inventory()
    for _, item in ipairs(inv.items) do
        if item.id == itemId and item.amount >= minAmount then
            return true
        end
    end
    return false
end

-- Example: Find dropped items near a position
local function findNearbyDroppedItems(x, y, radius)
    local world = bot:world()
    local nearby = {}

    for _, item in ipairs(world.floating) do
        local dx = item.x - x
        local dy = item.y - y
        local distance = math.sqrt(dx * dx + dy * dy)

        if distance <= radius then
            table.insert(nearby, item)
        end
    end

    return nearby
end

-- Example: Get tile at specific position
local function getTileAt(x, y)
    local world = bot:world()
    local index = y * world.width + x + 1  -- Lua uses 1-based indexing

    if index >= 1 and index <= #world.tiles then
        return world.tiles[index]
    end

    return nil
end

print("\nExample usage:")
if hasItem(2, 10) then
    print("  Bot has at least 10 of item ID 2")
else
    print("  Bot does not have enough of item ID 2")
end

local nearbyItems = findNearbyDroppedItems(bot.pos.x, bot.pos.y, 100)
print(string.format("  Found %d dropped items within 100 units", #nearbyItems))

local tile = getTileAt(10, 10)
if tile then
    print(string.format("  Tile at (10,10): fg=%d, bg=%d", tile.fg, tile.bg))
end
