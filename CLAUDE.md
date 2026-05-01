# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # compile
cargo run            # build and run
cargo test           # run tests
cargo clippy         # lint
cargo fmt            # format
```

## Project

**The Last Vestige** — a top-down colony builder/defender in the style of RimWorld.

- Engine: Bevy 0.18.1
- Camera: orthographic top-down
- Platform: desktop only
- Entry point: `src/main.rs`

## Code Organisation

Files have specific, focused responsibilities — keep things clean and tidy. One file should not do many unrelated jobs. Prefer one plugin per feature area.

## Module Structure

```
src/
  main.rs          # App entry point, startup systems
  map/
    mod.rs         # Declares submodules, re-exports Map, TileData, TileType, MapRendererPlugin
    map.rs         # TileType, TileData, Map struct and constructor — no generation logic
    map_gen.rs     # Map generation logic
    map_renderer.rs # Spawns and manages bevy_ecs_tilemap entities from Map resource
```

### Assets

- `assets/PlaceHolder_tileset1.png` — spritesheet, three 16×16 tiles: floor (0), wall (1), door (2). `TILE_SIZE = 16.0` in `map_renderer.rs`

## Architecture Decisions

### Tile System

- **Hybrid approach:** map data lives in a `Resource` (flat array, indexed `x + y * width`), visuals are entities/tilemap, dynamic actors (colonists, enemies, buildings) are entities with grid position components
- **Tiles are destructible and buildable** — walls can be broken by players and enemies, floors can be built on
- **Tile changes:** mutate the map resource → fire a `TileChangedEvent { x, y }` → listener updates visuals and pathfinding
- **Grid coordinates are the source of truth** — `Transform` is derived for rendering only, never used for game logic
- **Parallel arrays for rare data:** primary array holds only hot data (tile type, passability); oxygen, temperature, etc. live in separate resources indexed the same way for cache efficiency; truly sparse properties (affects <~5% of tiles) use `HashMap<(u32, u32), T>` instead
- **Keep `TileData` lean** — start small, add parallel resources only when actually needed

## Documentation Standards

- Use `///` doc comments on all `pub` structs, enums, and methods — these show in IDE tooltips and `cargo doc`
- Doc comments on the type itself explain what it represents; doc comments on methods explain what they do and return
- Fields should have `///` comments if their purpose isn't immediately obvious from the name
- Regular `//` comments are for non-obvious implementation details only — not for restating what the code does
- The developer wants help keeping code well documented for long-term maintainability — flag missing or inadequate doc comments when reviewing code

## Claude's Role

Claude should **never write code** with the exception of claude.md. Only explain concepts, approaches, and Bevy/Rust patterns so the developer writes the code themselves.

- **Level:** Intermediate Rust, beginner Bevy — assume Rust is solid, but explain Bevy-specific concepts (ECS, systems, plugins, resources, events) thoroughly including the why behind them
- **Always explain why** — not just what to do, but the reasoning and tradeoffs behind it
- **Point out problems, never fix them** — flag bugs, issues, and inefficiencies; let the developer resolve them
- **Flag working-but-suboptimal code** — if something works but is inefficient or could be done more sensibly, say so
- **Wait for the developer to drive** — don't suggest next steps or features unprompted
- **Warn about bad designs early** — if a design direction will cause pain (especially Bevy ECS anti-patterns common in colony/sim games e.g. storing too much state in single entities, overusing Resources instead of Components), raise it before they build too far