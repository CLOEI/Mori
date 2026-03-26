![Alt text](stuff/hero.png "Mori")

<br/>
<div align="center">
<h3 align="center">Mori - V2</h3>
<p align="center">
Your Cross-Platform Growtopia Companion
</p>
</div>

[Discord link for communication](https://discord.gg/a6FqT4G3dR)

## About The Project

There are many Growtopia companion tools available, but almost all of them are Windows-only, and I'm here to change that. Instead of being a GUI-focused app, I want to do something creative. This will be compiled as a CLI program, and it will open a port that you can use to preview the bots in your favorite browser, including their location in the world, manipulating location, and more. This is programmed in Rust, ensuring high performance and safety.

Star this project if you're interested in its journey!
<br/>
Any contribution would help alot.

## Features

- [x] Web GUI
- [ ] Auto update following the game client (version & items.dat)
- [x] Adding multi bot
- [x] Item database
- [x] Inventory
- [x] World map preview
- [ ] Growscan
- [x] Bot movement + findpath
- [x] Drop, trash item
- [x] Warp
- [x] Punch, place
- [x] Auto collect item
- [x] Auto Reconnect
- [ ] Google login ( Currently using API [growtopia-token](https://github.com/CLOEI/growtopia-token))
- [x] Session refresh
- [x] Legacy login
- [ ] Apple login
- [ ] Configureable delay
- [ ] Embedded scripting
- [ ] Bot terminal view
- [ ] Better item database with item image preview
- [ ] Better world map preview with texture
- [x] Socks5 support

## Running the tests

### All .dat tests at once

```sh
cargo test -- --nocapture
```

### Individual files

```sh
# items.dat
cargo test --test-thread=1 parse_items_dat -- --nocapture

# world.dat
cargo test --test-thread=1 parse_world_dat -- --nocapture

# save.dat (all save.dat tests)
cargo test save_dat -- --nocapture
```

---

## items.dat — `src/items.rs`

| Test | What it checks |
|---|---|
| `parse_items_dat` | Parses the full file, prints version, item count, and a few known items (IDs 1, 2, 32, 100, 242). Asserts item list is non-empty and last item has a non-empty name. |

**Expected output (example):**
```
version    : 24
item count : 10871
item[0]    : id=0 name="Blank" material=0 action=0 flags=0
item[  1]  : name="Dirt" texture="tiles_page1.rttex" rarity=1 grow_time=60
...
```

> `world.dat` is **not** required for `parse_items_dat`.

---

## world.dat — `src/world.rs`

| Test | What it checks |
|---|---|
| `parse_world_dat` | Parses the full world blob. Prints version, flags, world name, dimensions, tile count, object count, and weather. Asserts version == `0x19` and tile count == width × height. Also spot-checks the first 4 tiles for the expected fg item IDs. |

**Expected output (example):**
```
version      : 0x19
world_flags  : 0x0
world_name   : "START"
width        : 100
height       : 60
tiles        : 6000
objects      : 3
base_weather : 0
cur_weather  : 0
```

> If `world.dat` is missing the test prints `"world.dat not found — skipping"` and passes silently.

---

## save.dat — `src/save_dat.rs`

| Test | What it checks |
|---|---|
| `parse_save_dat` | Parses the file and prints every key/value pair. Asserts entry list is non-empty. |
| `roundtrip_save_dat` | Parses → serializes → re-parses and asserts all keys/values are identical. |
| `serialize_set` | Builds a `SaveDat` in memory (no file needed), round-trips through serialize/parse, checks `Token`, `Client`, and `player_age`. |
| `meta_xor_roundtrip` | Verifies the XOR-90210 codec on the `meta` field (no file needed). |
| `meta_from_save_dat` | Reads `save.dat`, decodes the `meta` field, and prints it. |
| `seed_diary_roundtrip` | Encodes/decodes a hand-crafted `SeedDiary` (no file needed). |
| `seed_diary_from_save_dat` | Reads `save.dat`, parses `seed_diary_data`, and prints every item ID with its grown flag. |

**Expected output for `parse_save_dat` (example):**
```
entries: 12
  "Token" = String("eyJ...")
  "LoginAccountType" = Int(1)
  "tankid_name" = String("MyName")
  ...
```



## Special thanks
[Badewen](https://github.com/badewen) - Help alot with debugging and reversing.


## Note

This is for educational purposes only. I am not responsible for any misuse of this tool. You also not allowed to sell or re-upload this tool as your own without my permission. use it at your own risk.

<p xmlns:cc="http://creativecommons.org/ns#" xmlns:dct="http://purl.org/dc/terms/"><a property="dct:title" rel="cc:attributionURL" href="https://github.com/CLOEI/Mori">Mori</a> by <a rel="cc:attributionURL dct:creator" property="cc:attributionName" href="https://github.com/CLOEI">Cendy</a> is licensed under <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/?ref=chooser-v1" target="_blank" rel="license noopener noreferrer" style="display:inline-block;">CC BY-NC-SA 4.0<img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/cc.svg?ref=chooser-v1" alt=""><img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/by.svg?ref=chooser-v1" alt=""><img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/nc.svg?ref=chooser-v1" alt=""><img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/sa.svg?ref=chooser-v1" alt=""></a></p>
