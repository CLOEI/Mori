# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Mori is a cross-platform Growtopia companion tool written in Rust. It features both a GUI (using Bevy + egui) and aims to provide a web-based interface for managing multiple game bots. The project uses a hybrid architecture with a core library (`gt-core`) that handles the game logic and networking, while the main application provides the user interface.

## Architecture

### Core Components

- **Main Application** (`src/main.rs`): Bevy-based GUI application with egui for UI rendering
- **Core Library** (`core/`): Contains all game logic, networking, and bot functionality
  - `Bot` struct: Main bot implementation with networking, automation, and scripting capabilities
  - Packet handling system for game protocol communication
  - A* pathfinding for bot movement
  - Lua scripting integration for automation
  - Inventory and world state management

### Key Dependencies

- **Bevy**: Game engine for GUI rendering and application framework
- **egui**: Immediate mode GUI for user interface
- **rusty_enet**: ENet networking library for game server communication
- **mlua**: Lua scripting integration for bot automation
- **headless_chrome**: Browser automation for token fetching with stealth mode
- **serde_json**: JSON parsing for token extraction
- **gtitem-r**: Item database parsing (external git dependency)
- **gtworld-r**: World data parsing (external git dependency)

## Development Commands

### Build and Run
```bash
cargo run                    # Run the GUI application
cargo build                  # Build the project
cargo build --release        # Build optimized release version
```

### Core Library Development
```bash
cargo build -p gt-core       # Build only the core library
cargo test -p gt-core        # Run core library tests
```

### Performance Profile Configuration
The project uses optimized debug builds:
- Dev profile: opt-level 1 for main crate
- Dependencies: opt-level 3 for all external dependencies

## Project Structure

### Main Application (`src/`)
- `main.rs`: Bevy application setup and UI systems with login method implementations
- `token.rs`: Enhanced token fetching using headless Chrome with stealth mode and JSON parsing

### Core Library (`core/src/`)
- `lib.rs`: Main Bot struct and public API with updated TokenFetcher signature
- `types/`: Type definitions for game protocol and bot state
- `packet_handler.rs`/`variant_handler.rs`: Network protocol handling
- `login.rs`: Authentication and login logic
- `server.rs`: Server communication and data retrieval
- `inventory.rs`: Inventory management
- `astar.rs`: A* pathfinding implementation
- `lua.rs`: Lua scripting engine integration
- `utils/`: Utility modules for protocol handling

## Login System Implementation

The application supports four login methods with distinct UI flows:

### Google/Apple Authentication
- Uses token fetcher with headless Chrome automation
- Stealth mode enabled to avoid detection
- Automatic token extraction from validation response

### LTOKEN Authentication
- Direct token input (4 colon-separated values)
- No token fetcher required
- Immediate bot creation upon validation

### Legacy Authentication
- Username/password credential input
- Uses internal token generation
- Fallback method for traditional login

### Token Fetcher Architecture
- Type signature: `Box<dyn Fn(String, String) -> String + Send + Sync>`
- Parameters: bot_name (placeholder) and URL
- Returns extracted token from browser automation
- JSON response parsing with error handling

## Key Features Implementation

- **Multi-bot Management**: Each bot runs in its own thread with Arc<Bot> for safe sharing
- **Real-time World State**: World data synchronized with game server
- **Automation System**: Configurable delays and automated actions
- **Scripting**: Embedded Lua for custom automation scripts
- **Item Database**: External item.dat file parsing for game items
- **Path Finding**: A* algorithm for intelligent bot movement
- **Enhanced Token Security**: Stealth mode browser automation with JSON parsing

## Important Notes

- Bot connections use ENet protocol with custom packet handling
- The project requires `items.dat` file for item database functionality
- Token fetching uses headless Chrome with stealth mode to avoid detection
- Browser automation extracts tokens from JSON responses in validation pages
- All bot operations are thread-safe using Mutex/RwLock patterns
- Network packets follow Growtopia's custom protocol implementation
- Each login method has distinct initialization flows and token requirements

## External Dependencies

The core library depends on custom Rust implementations of Growtopia protocols:
- `rusty_enet`: Custom ENet implementation
- `gtitem-r`: Item database parser
- `gtworld-r`: World data parser

These are maintained as separate Git repositories and may need updates when the game protocol changes.