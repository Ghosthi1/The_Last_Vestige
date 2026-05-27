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
  constants.rs     # Shared constants — TILE_SIZE, MAP_WIDTH, MAP_HEIGHT, OFFSETS, ENEMY_SPEED, ENEMY_STOP_RADIUS, ENEMY_SEPARATION_STRENGTH
  map/
    mod.rs         # Declares submodules, re-exports Map, TileData, TileType, MapRendererPlugin
    map.rs         # TileType, TileData, Map struct and constructor — no generation logic
    map_gen.rs     # Map generation logic
    map_renderer.rs # Spawns and manages bevy_ecs_tilemap entities from Map resource
  ai/
    mod.rs         # Declares submodules
    a_star.rs      # find_path(map, start, goal) — returns Option<Vec<(u32,u32)>>, 8-directional
    flow_fields.rs # FlowLayer enum, FlowField struct (per-layer BFS data), FlowFields resource (named fields per layer)
    ai_plugins.rs  # AiPlugin; rebuild_colonist_flow_field system
  character/
    mod.rs         # Declares submodules, re-exports GridPosition, Path, Speed, Colonist, CharacterPlugin
    characters.rs  # GridPosition, Path, Speed, Colonist components; CharacterPlugin; spawn, movement, and click-to-move systems
  enemys/
    mod.rs         # Declares submodules, re-exports Enemy and EnemyPlugin
    enemy.rs       # Enemy marker component; EnemyPlugin; spawn and flow-field-driven movement systems
  camera/
    mod.rs         # Declares submodules, re-exports CameraPlugin
    camera.rs      # CameraPlugin; zoom_camera (scroll wheel, multiplicative scale on OrthographicProjection); pan_camera (middle mouse drag, delta scaled by ortho.scale)
```

### Assets

- `assets/PlaceHolder_tileset.png` — spritesheet, three 32×32 tiles: floor (0), wall (1), door (2). `TILE_SIZE = 32.0` defined in `src/constants.rs` as a shared `pub const`, imported via `use crate::constants::TILE_SIZE` wherever tile sizing is needed
- `assets/enemeys/Spiders/Grunt.png` — sprite for the Grunt enemy; loaded via `AssetServer` in `spawn_enemy` and set on the `Sprite` `image` field; `custom_size` is `Vec2::splat(TILE_SIZE)` but the grunt is intentionally drawn smaller than the canvas for visual style — hitbox size will be defined independently when collision is added

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
- **`FlowLayer` enum** names the layer types — one variant per goal type (e.g. `Colonists`); add variants as new goal types are needed
- **`FlowField` struct** holds `width`, `height`, `directions: Vec<Option<(i8, i8)>>`, `cost_so_far: Vec<u32>`, `valid_goals: Vec<(u32, u32)>`, and `open_set: BinaryHeap<(Reverse<u32>, u32, u32)>` — `directions` is `None` for impassable/unreachable tiles, `Some((0,0))` for the goal tile itself; `cost_so_far`, `valid_goals`, and `open_set` are all reusable buffers stored as fields and cleared at the start of each rebuild to avoid per-call heap allocation
- **`build_flow_fields(&mut self, map, goals)`** takes a slice of goal positions — seeds all goals into the heap at cost 0 so the Dijkstra expands from all simultaneously; each tile's direction points toward whichever goal is cheapest to reach. Same 10/14 cardinal/diagonal cost model as A* for consistency
- **Min-heap via `std::cmp::Reverse`** — cost is wrapped in `Reverse` on push and unwrapped on pop, giving correct Dijkstra ordering (cheapest node first) from Rust's max-heap `BinaryHeap`
- **Multiple goals:** seeding multiple positions at cost 0 before the loop is all that's needed — the BFS naturally produces a "nearest goal" field for free
- **Direction convention:** direction at `(nx, ny)` = `(x - nx, y - ny)` cast to `i8`, where `(x, y)` is the parent tile (one step closer to goal); values are always in `{-1, 0, 1}`
- **Lazy deletion** — same pattern as A*: duplicate heap entries are allowed, stale ones skipped via `cost_so_far` comparison on pop
- **Flat `Vec` arrays** for `cost_so_far` and `directions`, indexed by `x + y * width`; `u32::MAX` is the sentinel for "not yet reached"
- **`OFFSETS` constant** lives in `constants.rs` and is shared with `a_star.rs` — both use the same 8-directional neighbourhood
- **`build_flow_fields` validates goals upfront** — filters goals into a `valid_goals` vec before seeding; skips invalid or impassable goals so they are never seeded; returns early if no valid goals remain
- **`FlowFields` resource** has named fields (`colonists`, `structures`, `walls`) — one `FlowField` per layer, accessed directly without hashing; implements `Default` using `MAP_WIDTH`/`MAP_HEIGHT` so new layers only require adding a field and a line to the `Default` impl; inserted in `main.rs` as `FlowFields::default()`
- **`AiPlugin`** in `ai/ai_plugins.rs` owns the rebuild system — `pub fn rebuild_colonist_flow_field` runs every `Update` with two queries: both filtered `With<Colonist>` so enemies with `GridPosition` are never included as goals; one also filtered on `Changed<GridPosition>` as a cheap early-return gate; uses `Local<Vec<(u32,u32)>>` for the positions buffer so it is allocated once and reused each frame; rebuilds the `colonists` field directly
- **Rebuild trigger:** `GridPosition` is written in `move_character` (`grid_pos.0 = *next`) when a colonist arrives at a waypoint — this marks the component changed and fires the rebuild system that frame
- **Layer design:** `FlowLayer` variants represent targets things navigate *toward* — `Colonists` means "goal is colonist positions, used by enemies"; colonists themselves use A* for player-directed movement

### Characters

- **Components:** `GridPosition((u32, u32))` — authoritative grid position (inner tuple); `Path(VecDeque<(u32,u32)>)` — remaining waypoints; `Speed(f32)` — movement speed in tiles per second; `Colonist` — zero-sized marker, filters colonist-only queries so enemies are never accidentally included
- **Smooth movement:** `move_character` advances `Transform` toward the next waypoint each frame using `move_towards(target, speed * delta_secs)`; `GridPosition` is only updated when the character arrives at a waypoint (`distance_squared < 0.01`, avoiding a sqrt)
- **Click-to-move:** `move_to_click` converts cursor window position → world position via `camera.viewport_to_world_2d`, then applies the tilemap centering offset to get grid coordinates, bounds-checks both axes before casting to `u32` (negative cast saturates silently), then calls `find_path`
- **System ordering:** `move_to_click` is chained before `move_character` via `.chain()` — ensures a click is applied before movement processes that same frame
- **Tilemap offset:** the tilemap is centered on screen — tile world position = `tile_coord * TILE_SIZE + TILE_SIZE/2 - map_size * TILE_SIZE/2`; this places entities at the **center** of each tile; all coordinate conversions must account for this
- **Loop-invariant hoisting:** map offset values (`width/height * TILE_SIZE/2`) are computed once before the character loop in `move_character`, not per-iteration

### Enemies

- **`Enemy` marker component** — zero-sized, lives in `enemys/enemy.rs`; used to filter enemy-only queries and distinguish enemies from colonists who share `GridPosition` and `Speed`
- **Continuous movement** — enemies move in world space, not tile-to-tile; `Transform` is authoritative, `GridPosition` is derived from it each frame by `(translation + offset) / TILE_SIZE`, floored to `u32`; this allows more than 8 enemies to surround a single colonist
- **Flow-field movement** — each frame `move_enemy` looks up the flow field direction for the enemy's current `GridPosition`, converts the `(i8, i8)` to a normalised `Vec2`, scales by `speed * delta_secs`, and applies via axis-separated wall collision (see below); normalisation ensures diagonal movement is not faster than cardinal
- **Colonist proximity stop** — before applying velocity, `move_enemy` collects all colonist positions into a `Vec<Vec2>` snapshot once before the enemy loop, then checks if any colonist is within `TILE_SIZE * ENEMY_STOP_RADIUS` (distance squared); if so, the enemy stops moving that frame; `ENEMY_STOP_RADIUS = 0.7` lives in `constants.rs`
- **Separation steering** — `separate_enemies` runs `.before(move_enemy)` each frame; snapshots all enemy positions into a `Vec<Vec2>`, then iterates the upper triangle of pairs (`j > i`) to compute repulsion once per pair and accumulate into a `forces` vec — `forces[i] += force`, `forces[j] -= force`; force uses a smooth linear falloff `(1.0 - dist / TILE_SIZE) * diff / dist`, reusing the single `length()` call; a second pass applies `forces[i]` to each transform via axis-separated wall collision; `ENEMY_SEPARATION_STRENGTH = 10.0` lives in `constants.rs`
- **Axis-separated wall collision** — both `move_enemy` and `separate_enemies` apply movement one axis at a time; before adding a delta to `transform.translation.x`, a test position `(current_pos + Vec2::new(delta_x, 0.0))` is passed to `tile_at`; if it returns `None` (wall or out of bounds) the x movement is skipped; same check independently for y; this prevents enemies being pushed into walls by separation forces while still allowing sliding along wall faces — the `tile_at` helper in `enemy.rs` converts a world `Vec2` to a tile coordinate, returning `None` if out of bounds or impassable
- **Query disjointness** — `move_enemy` accesses `&mut Transform` for enemies and `&Transform` for colonists; Bevy requires explicit `Without<Colonist>` on the enemy query and `Without<Enemy>` on the colonist query to prove they never overlap, otherwise it panics with `B0001` at startup
- **System ordering:** `separate_enemies.before(move_enemy)`, `move_enemy.after(rebuild_colonist_flow_field)` — separation is applied before flow-field movement each frame; flow field is always current before enemies read it
- **Spawn:** `spawn_enemy` takes `AssetServer` as a parameter; the texture handle must be `.clone()`d for every spawn call since `Handle<Image>` is moved on first use; `GridPosition` and `Transform` must be initialised from the same grid coordinates

### Rendering

- **Texture sampler** — `DefaultPlugins.set(ImagePlugin::default_nearest())` in `main.rs` sets nearest-neighbor sampling globally; this is required for pixel art to remain crisp at all zoom levels — Bevy's default bilinear sampler blurs upscaled sprites; since the entire game is pixel art the global setting is correct and no per-asset override is needed
- **Tilemap transform offset** — `bevy_ecs_tilemap` centers each tile's sprite at its grid position in local space, so tile `(0,0)` is centered at the tilemap entity's `Transform` origin. To align with the character/gizmo coordinate system (where tile centers sit at `tx * TILE_SIZE + TILE_SIZE/2 - MAP_WIDTH * TILE_SIZE/2`), the tilemap transform must be `-(map.width * TILE_SIZE)/2 + TILE_SIZE/2` in x and the same in y — without the `+ TILE_SIZE/2` the tiles appear shifted half a tile left/down relative to all other coordinate systems

### Camera

- **Zoom** — `zoom_camera` reads `Res<AccumulatedMouseScroll>` and applies a multiplicative scale change to `OrthographicProjection.scale` each frame: `scale = (scale * (1.0 - delta.y * sensitivity)).clamp(0.3, 3.0)`; sensitivity is `0.1`; multiplicative scaling feels consistent at all zoom levels; lower clamp bound is a UX decision — nearest-neighbor keeps pixels crisp but extreme zoom-in shows very large blocky pixels, so the clamp prevents unintentionally rough-looking close-up views
- **Pan** — `pan_camera` checks `ButtonInput<MouseButton>::pressed(Middle)` and reads `Res<AccumulatedMouseMotion>`; translates the camera by the mouse delta multiplied by `ortho.scale` so panning speed stays consistent regardless of zoom level; x is negated (drag right = pan left), y is added as-is (screen and world y felt correct without negation)
- **Projection access** — `OrthographicProjection` is not a standalone component in Bevy 0.18; access it via `Query<(&mut Transform, &Projection)>` and match `Projection::Orthographic(ref mut ortho)` to read or write `ortho.scale`
- **Input resources** — Bevy 0.18 provides `AccumulatedMouseScroll` and `AccumulatedMouseMotion` as frame-accumulated resources; prefer these over `EventReader<MouseWheel>`/`EventReader<MouseMotion>` for per-frame input reading

### Tile System

- **Hybrid approach:** map data lives in a `Resource` (flat array, indexed `x + y * width`), visuals are entities/tilemap, dynamic actors (colonists, enemies, buildings) are entities with grid position components
- **Tiles are destructible and buildable** — walls can be broken by players and enemies, floors can be built on
- **Tile changes:** mutate the map resource → fire a `TileChangedEvent { x, y }` → listener updates visuals and pathfinding
- **Grid coordinates are the source of truth for colonists** — colonist `Transform` is derived from `GridPosition`; for enemies the relationship is reversed: `Transform` is authoritative and `GridPosition` is derived from it each frame to support continuous swarming movement
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