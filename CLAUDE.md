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
  constants.rs     # Shared constants — TILE_SIZE, MAP_WIDTH, MAP_HEIGHT
  map/
    mod.rs         # Declares submodules, re-exports Map, TileData, TileType, MapRendererPlugin
    map.rs         # TileType, TileData, Map struct and constructor — no generation logic
    map_gen.rs     # Map generation logic
    map_renderer.rs # Spawns and manages bevy_ecs_tilemap entities from Map resource
  ai/
    mod.rs         # Declares submodules
    a_star.rs      # find_path(map, start, goal) — returns Option<Vec<(u32,u32)>>, 8-directional
    flow_fields.rs # FlowLayer enum, FlowField struct (per-layer BFS data), FlowFields resource (HashMap of layers)
  character/
    mod.rs         # Declares submodules, re-exports GridPosition, Path, Speed, CharacterPlugin
    characters.rs  # GridPosition, Path, Speed components; CharacterPlugin; spawn, movement, and click-to-move systems
```

### Assets

- `assets/PlaceHolder_tileset1.png` — spritesheet, three 16×16 tiles: floor (0), wall (1), door (2). `TILE_SIZE = 16.0` defined in `src/constants.rs` as a shared `pub const`, imported via `use crate::constants::TILE_SIZE` wherever tile sizing is needed

## Architecture Decisions

### Pathfinding

- **A\* implementation** lives in `ai/a_star.rs` — `find_path` takes a `&Map`, start, and goal as `(u32, u32)` grid coords, returns `Option<Vec<(u32, u32)>>`
- **8-directional movement** with Chebyshev heuristic (`max(dx, dy)`)
- **Passability** is derived from `TileType` via `is_passable()` — no separate field, so it's always in sync with tile state
- **Lazy deletion** pattern for the open set — duplicate nodes are allowed in the heap, skipped via `closed_set`; `g_scores` prevents `came_from` being overwritten by worse paths
- **Non-uniform movement cost** — cardinal moves cost 10, diagonal moves cost 14 (approximating √2 × 10); this naturally produces visually direct paths by making unnecessary diagonals more expensive. Heuristic is scaled by 10 to remain admissible.
- **Tie-breaking by `h`** — when two nodes share the same `f` score, the one closer to the goal (lower `h`) is preferred; keeps the search greedy in tie cases and reduces nodes explored on open maps
- **Flat `Vec` arrays** replace HashMaps for `g_scores`, `closed_set`, and `came_from` — indexed by `x + y * width`; avoids hashing overhead and improves cache locality
- **`came_from` stores packed `u32` indices** rather than `(u32, u32)` tuples — unpack with `index % width` for x and `index / width` for y; sentinel value `u32::MAX` means no parent set
- **`f` is precomputed** on `Node` construction and stored as a field — avoids recomputing `g + h` on every heap comparison; `h` is also bound to a local before each `push` so `heuristic` is never called twice for the same node
- **`find_path` validates inputs upfront** — returns `None` immediately if start or goal are out of bounds, or if the goal tile is impassable; the expensive search is never started in those cases

### Flow Fields

- **Purpose:** shares one BFS computation across all entities targeting the same goal — every reachable tile gets a direction pointing toward the goal, so N colonists pay O(map) not O(N × path)
- **`FlowLayer` enum** keys the `FlowFields` resource HashMap — one layer per goal type (e.g. `Colonist`); add variants as new goal types are needed
- **`FlowField` struct** holds `width`, `height`, and `directions: Vec<Option<(i8, i8)>>` — `None` means impassable or unreachable; `Some((0,0))` means the goal tile itself
- **`build_flow_fields(&mut self, map, goal)`** runs Dijkstra outward from the goal tile, filling each reachable tile's direction with the unit vector pointing back toward the goal — same 10/14 cardinal/diagonal cost model as A* for consistency
- **Direction convention:** direction at `(nx, ny)` = `(x - nx, y - ny)` cast to `i8`, where `(x, y)` is the parent tile (one step closer to goal); values are always in `{-1, 0, 1}`
- **Lazy deletion** — same pattern as A*: duplicate heap entries are allowed, stale ones skipped via `cost_so_far` comparison on pop
- **Flat `Vec` arrays** for `cost_so_far` and `directions`, indexed by `x + y * width`; `u32::MAX` is the sentinel for "not yet reached"
- **`OFFSETS` constant** lives in `constants.rs` and is shared with `a_star.rs` — both use the same 8-directional neighbourhood
- **`build_flow_fields` validates upfront** — returns early if goal is out of bounds or impassable, same guard pattern as `find_path`
- **`FlowFields` resource** holds a `HashMap<FlowLayer, FlowField>` — must be inserted into the app at startup; rebuild the relevant layer whenever the goal changes or a `TileChangedEvent` fires

### Characters

- **Components:** `GridPosition(u32, u32)` — authoritative grid position; `Path(VecDeque<(u32,u32)>)` — remaining waypoints; `Speed(f32)` — movement speed in tiles per second
- **Smooth movement:** `move_character` advances `Transform` toward the next waypoint each frame using `move_towards(target, speed * delta_secs)`; `GridPosition` is only updated when the character arrives at a waypoint (`distance_squared < 0.01`, avoiding a sqrt)
- **Click-to-move:** `move_to_click` converts cursor window position → world position via `camera.viewport_to_world_2d`, then applies the tilemap centering offset to get grid coordinates, bounds-checks both axes before casting to `u32` (negative cast saturates silently), then calls `find_path`
- **System ordering:** `move_to_click` is chained before `move_character` via `.chain()` — ensures a click is applied before movement processes that same frame
- **Tilemap offset:** the tilemap is centered on screen — tile world position = `tile_coord * 16 - map_size * 8`; all coordinate conversions must account for this
- **Loop-invariant hoisting:** map offset values (`width/height * 8.0`) are computed once before the character loop in `move_character`, not per-iteration

### Tile System

- **Hybrid approach:** map data lives in a `Resource` (flat array, indexed `x + y * width`), visuals are entities/tilemap, dynamic actors (colonists, enemies, buildings) are entities with grid position components
- **Tiles are destructible and buildable** — walls can be broken by players and enemies, floors can be built on
- **Tile changes:** mutate the map resource → fire a `TileChangedEvent { x, y }` → listener updates visuals and pathfinding
- **Grid coordinates are the source of truth** — `Transform` is derived for rendering only, never used for game logic
- **Parallel arrays for rare data:** primary array holds only hot data (tile type, passability); oxygen, temperature, etc. live in separate resources indexed the same way for cache efficiency; truly sparse properties (affects <~5% of tiles) use `HashMap<(u32, u32), T>` instead
- **Keep `TileData` lean** — start small, add parallel resources only when actually needed
- **Map expands infinitely** via procedural chunk-based generation; fog of war hides unexplored chunks. When the chunk system is built, fire a `MapResizedEvent` (or chunk-reveal event) so dependent systems (grid overlay, pathfinding) can react
- **Grid overlay deferred** — a `PrimitiveTopology::LineList` mesh is the right approach; build it once the chunk/expansion system exists so the mesh update hook has something to connect to
- **`MapOffset` is fragile** — currently hardcoded in `main.rs` with the map size baked in; will break when the map expands. Revisit when the chunk/expansion system is built — the offset should be derived from map state, not set once at startup

## Documentation Standards

- Use `///` doc comments on all `pub` structs, enums, and methods — these show in IDE tooltips and `cargo doc`
- Doc comments on the type itself explain what it represents; doc comments on methods explain what they do and return
- Fields should have `///` comments if their purpose isn't immediately obvious from the name
- Regular `//` comments are for non-obvious implementation details only — not for restating what the code does
- The developer wants help keeping code well documented for long-term maintainability — flag missing or inadequate doc comments when reviewing code

## Claude's Role

Claude should **never write code** with the exception of claude.md. Only explain concepts, approaches, and Bevy/Rust patterns so the developer writes the code themselves. This includes small inline snippets — no code at all unless the developer explicitly asks for an example.

- **Level:** Intermediate Rust, beginner Bevy — assume Rust is solid, but explain Bevy-specific concepts (ECS, systems, plugins, resources, events) thoroughly including the why behind them
- **Always explain why** — not just what to do, but the reasoning and tradeoffs behind it
- **Point out problems, never fix them** — flag bugs, issues, and inefficiencies; let the developer resolve them
- **Flag working-but-suboptimal code** — if something works but is inefficient or could be done more sensibly, say so
- **Wait for the developer to drive** — don't suggest next steps or features unprompted
- **Warn about bad designs early** — if a design direction will cause pain (especially Bevy ECS anti-patterns common in colony/sim games e.g. storing too much state in single entities, overusing Resources instead of Components), raise it before they build too far