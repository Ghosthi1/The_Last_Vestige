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
  constants.rs     # Shared constants — TILE_SIZE, MAP_WIDTH, MAP_HEIGHT, OFFSETS, ENEMY_SPEED, ENEMY_STOP_RADIUS, ENEMY_SEPARATION_STRENGTH, ENEMY_HEALTH, COLONIST_HEALTH, COLONIST_SPEED
  map/
    mod.rs         # Declares submodules, re-exports Map, TileData, TileType, MapRendererPlugin
    map.rs         # TileType, TileData, Map struct and constructor — no generation logic; cursor_to_grid(camera, camera_transform, cursor_pos, map) shared utility
    map_gen.rs     # Map generation logic
    map_renderer.rs # Spawns and manages bevy_ecs_tilemap entities from Map resource
  ai/
    mod.rs         # Declares submodules
    a_star.rs      # find_path(map, start, goal) — returns Option<Vec<(u32,u32)>>, 8-directional
    flow_fields.rs # FlowLayer enum, FlowField struct (per-layer BFS data), FlowFields resource (named fields per layer)
    ai_plugins.rs  # AiPlugin; rebuild_colonist_flow_field system
  components/
    mod.rs         # Declares submodules, re-exports all shared components
    movement.rs    # GridPosition, Path, Speed — shared movement components used by both colonists and enemies
    combat.rs      # Health component — private current/max f32 fields; new(max), change_health(delta), is_dead(); used by both colonists and enemies
  character/
    mod.rs         # Declares submodules, re-exports Colonist, CharacterPlugin
    characters.rs  # Colonist marker component; CharacterPlugin; separate_colonists, move_character, move_to_click systems; tile_at helper
  enemys/
    mod.rs         # Declares submodules, re-exports Enemy, EnemyPlugin, EnemySpawnerPlugin
    enemy.rs       # Enemy marker component; EnemyPlugin; flow-field-driven movement systems (move_enemy, separate_enemies)
    enemy_spawner.rs # EnemySpawnerPlugin; spawn_enemy Startup system — spawns enemies with Enemy, GridPosition, Health, Speed, Sprite, Transform
  buildings/
    mod.rs         # Declares submodules, re-exports BuildingPlugin, TileChangedEvent
    buildings.rs   # BuildingPlugin; TileChangedEvent; place_wall_on_click, on_tile_change, on_tile_passability_change systems
  systems/
    mod.rs         # Declares camera and sound submodules, re-exports both
    camera.rs      # CameraPlugin; setup (Startup — spawns Camera2d); zoom_camera (scroll wheel, multiplicative scale on OrthographicProjection); pan_camera (middle mouse drag, delta scaled by ortho.scale)
    sound.rs       # AmbientPlugin; startup system that loads and spawns the looping ambient audio entity
```

### Assets

- `assets/PlaceHolder_tileset.png` — spritesheet, three 32×32 tiles: floor (0), wall (1), door (2). `TILE_SIZE = 32.0` defined in `src/constants.rs` as a shared `pub const`, imported via `use crate::constants::TILE_SIZE` wherever tile sizing is needed
- `assets/enemeys/Spiders/Grunt.png` — sprite for the Grunt enemy; loaded via `AssetServer` in `spawn_enemy` and set on the `Sprite` `image` field; `custom_size` is `Vec2::splat(TILE_SIZE)` but the grunt is intentionally drawn smaller than the canvas for visual style — hitbox size will be defined independently when collision is added
- `assets/Sound/Background/ambient_spaceship.ogg` — looping ambient soundtrack; loaded and spawned as an audio entity in `Systems/ambient.rs` via `AmbientPlugin`

## Architecture Decisions

### Pathfinding

- **A\* implementation** lives in `ai/a_star.rs` — `find_path` takes a `&Map`, start, and goal as `(u32, u32)` grid coords, returns `Option<Vec<(u32, u32)>>` where `None` means no path exists; the vec includes both start and goal; two private helpers: `idx(x, y, width)` converts 2D grid coords to a flat array index, `reconstruct_path(came_from, goal, width)` walks the `came_from` array backwards from goal to start and reverses the result
- **8-directional movement** with Chebyshev heuristic (`max(dx, dy)`)
- **Passability** is derived from `TileType` via `is_passable()` — no separate field, so it's always in sync with tile state
- **Lazy deletion** pattern for the open set — duplicate nodes are allowed in the heap, skipped via `closed_set`; `g_scores` prevents `came_from` being overwritten by worse paths
- **Non-uniform movement cost** — cardinal moves cost 10, diagonal moves cost 14 (approximating √2 × 10); this naturally produces visually direct paths by making unnecessary diagonals more expensive. Heuristic is scaled by 10 to remain admissible.
- **Tie-breaking by `h`** — when two nodes share the same `f` score, the one closer to the goal (lower `h`) is preferred; keeps the search greedy in tie cases and reduces nodes explored on open maps
- **Flat `Vec` arrays** replace HashMaps for `g_scores`, `closed_set`, and `came_from` — indexed by `x + y * width`; avoids hashing overhead and improves cache locality
- **`came_from` stores packed `u32` indices** rather than `(u32, u32)` tuples — unpack with `index % width` for x and `index / width` for y; sentinel value `u32::MAX` means no parent set
- **`f` is precomputed** on `Node` construction and stored as a field — avoids recomputing `g + h` on every heap comparison; `h` is also bound to a local before each `push` so `heuristic` is never called twice for the same node
- **`find_path` validates inputs upfront** — returns `None` immediately if start or goal are out of bounds, or if the goal tile is impassable; the expensive search is never started in those cases
- **No diagonal corner-cutting** — when expanding a diagonal neighbour `(nx, ny)` from current node `(x, y)`, both bordering cardinal tiles must also be passable: `(nx, node.pos.1)` and `(node.pos.0, ny)`; if either is a wall the diagonal is skipped; prevents paths that squeeze through the gap between two diagonally adjacent walls

### Flow Fields

- **Purpose:** shares one BFS computation across all entities targeting the same goal — every reachable tile gets a direction pointing toward the goal, so N colonists pay O(map) not O(N × path)
- **`FlowLayer` enum** — defined but not yet wired up; intended for future dynamic layer selection (e.g. passing a layer type to a system rather than accessing fields directly by name)
- **`FlowField` struct** holds `width`, `height`, `directions: Vec<Option<(i8, i8)>>`, `cost_so_far: Vec<u32>`, `valid_goals: Vec<(u32, u32)>`, and `open_set: BinaryHeap<(Reverse<u32>, u32, u32)>` — `directions` is `None` for impassable/unreachable tiles, `Some((0,0))` for the goal tile itself; `cost_so_far`, `valid_goals`, and `open_set` are all reusable buffers stored as fields and cleared at the start of each rebuild to avoid per-call heap allocation
- **`build_flow_fields(&mut self, map, goals)`** takes a slice of goal positions — seeds all goals into the heap at cost 0 so the Dijkstra expands from all simultaneously; each tile's direction points toward whichever goal is cheapest to reach. Same 10/14 cardinal/diagonal cost model as A* for consistency
- **Min-heap via `std::cmp::Reverse`** — cost is wrapped in `Reverse` on push and unwrapped on pop, giving correct Dijkstra ordering (cheapest node first) from Rust's max-heap `BinaryHeap`
- **Multiple goals:** seeding multiple positions at cost 0 before the loop is all that's needed — the BFS naturally produces a "nearest goal" field for free
- **Direction convention:** direction at `(nx, ny)` = `(x - nx, y - ny)` cast to `i8`, where `(x, y)` is the parent tile (one step closer to goal); values are always in `{-1, 0, 1}`
- **Lazy deletion** — same pattern as A*: duplicate heap entries are allowed, stale ones skipped via `cost_so_far` comparison on pop
- **Flat `Vec` arrays** for `cost_so_far` and `directions`, indexed by `x + y * width`; `u32::MAX` is the sentinel for "not yet reached"
- **`OFFSETS` constant** lives in `constants.rs` and is shared with `a_star.rs` — both use the same 8-directional neighbourhood
- **`build_flow_fields` validates goals upfront** — filters goals into a `valid_goals` vec before seeding; skips invalid or impassable goals so they are never seeded; returns early if no valid goals remain
- **No diagonal corner-cutting** — when expanding a diagonal neighbour `(nx, ny)` from current tile `(x, y)`, both bordering cardinal tiles must be passable: `(cx, y)` and `(x, cy)` where `cx = x + dx` and `cy = y + dy`; if either is a wall the diagonal is skipped; same rule as A* for consistency
- **`FlowFields` resource** has named fields (`colonists`, `structures`, `walls`) — one `FlowField` per layer, accessed directly without hashing; implements `Default` using `MAP_WIDTH`/`MAP_HEIGHT` so new layers only require adding a field and a line to the `Default` impl; inserted in `main.rs` as `FlowFields::default()`
- **`AiPlugin`** in `ai/ai_plugins.rs` owns the rebuild system — `pub fn rebuild_colonist_flow_field` runs every `Update` with two queries: both filtered `With<Colonist>` so enemies with `GridPosition` are never included as goals; one also filtered on `Changed<GridPosition>` as a cheap early-return gate; uses `Local<Vec<(u32,u32)>>` for the positions buffer so it is allocated once and reused each frame; rebuilds the `colonists` field directly
- **Rebuild trigger:** `GridPosition` is written in `move_character` (`grid_pos.0 = *next`) when a colonist arrives at a waypoint — this marks the component changed and fires the rebuild system that frame
- **Layer design:** `FlowFields` fields represent targets things navigate *toward* — `colonists` means "goal is colonist positions, used by enemies"; colonists themselves use A* for player-directed movement; layers are accessed directly by field name (`flow_fields.colonists`) rather than via `FlowLayer` dispatch

### Combat

- **`Health` component** — lives in `components/combat.rs`; private `current: f32` and `max: f32` fields; constructed via `Health::new(max)` which sets `current = max` and `debug_assert!`s `max > 0.0`; fields are private so all access goes through methods
- **`change_health(delta: f32)`** — adds `delta` to `current` (negative for damage, positive for healing); clamps `current` to `[0.0, max]`; when `current` hits `0.0` the entity is considered dead — a TODO marks where a `Dead` marker component should be inserted
- **`is_dead() -> bool`** — returns `self.current <= 0.0`; intended to be called by an external system that queries for dead entities and handles despawning, tile events, and animations — `Health` itself cannot interact with the world
- **Death handling pattern** — `Health` is pure data; death detection belongs in a system that queries entities with `Health`, calls `is_dead()`, and inserts a `Dead` marker component; downstream systems (`Without<Dead>` on movement, a cleanup system `With<Dead>`) react to the marker — not yet implemented
- **`Dead` marker component** — not yet added; will be a zero-sized component in `components/combat.rs` alongside `Health`; used to filter dead entities out of movement/AI queries and into cleanup/animation systems

### Characters

- **Components:** `GridPosition((u32, u32))` — authoritative grid position (inner tuple), lives in `components/movement.rs`; `Path(VecDeque<(u32,u32)>)` — remaining waypoints, lives in `components/movement.rs`; `Speed(f32)` — movement speed in tiles per second, lives in `components/movement.rs`; `Health` — current/max health, lives in `components/combat.rs`; `Colonist` — zero-sized marker in `character/characters.rs`, filters colonist-only queries so enemies are never accidentally included
- **Colonist bundle:** `Colonist`, `GridPosition`, `Health::new(COLONIST_HEALTH)`, `Speed(COLONIST_SPEED)`, `Sprite`, `Transform`, `Path` — `COLONIST_HEALTH` and `COLONIST_SPEED` are constants in `constants.rs`
- **Smooth movement:** `move_character` advances `Transform` toward the next waypoint each frame using `move_towards(target, speed * delta_secs)`; `GridPosition` is only updated when the character arrives at a waypoint (`distance_squared < 0.01`, avoiding a sqrt); `transform.translation` is only snapped to the tile center on normal arrival — not in the conflict branch, so the transform never visually lands on an occupied tile
- **Click-to-move:** `move_to_click` calls `cursor_to_grid` (shared utility in `map/map.rs`) to convert cursor window position → grid coordinates; start position is `path.0.front()` if a path is already in progress, otherwise `grid_pos.0` — keeps movement smooth mid-path by continuing from the current waypoint rather than snapping back to grid position
- **Click-time goal assignment:** before the assignment loop, `move_to_click` snapshots all colonist `GridPosition`s into a `mut HashSet<(u32, u32)>`; for each colonist, if the clicked goal is already in the set, searches 8 neighbours of the goal for a free passable tile; uses `actual_goal` (the neighbour, or the original goal if free) for `find_path`; inserts `actual_goal` into the set so subsequent colonists in the same click get distinct targets — prevents multiple colonists converging on the same world position
- **Arrival-time conflict detection:** `move_character` snapshots all `GridPosition`s into a `mut HashSet` before the movement loop; at arrival, checks `occupied.contains(next) && *next != grid_pos.0` — the guard excludes the start-waypoint case where `next` equals the colonist's current tile (which appears in `occupied` as their own position); if occupied, searches 8 neighbours of `next` for a free passable tile and replaces `path.0[0]` with it without updating `grid_pos` or snapping transform; on normal arrival, inserts the arrived tile into `occupied` so same-frame arrivals at the same tile are caught by subsequent iterations
- **Two-phase snapshot pattern:** both `move_to_click` and `move_character` use the same pattern — `query.iter()` collects `GridPosition` values into an owned `HashSet` (the immutable borrow ends when `.collect()` finishes), then `query.iter_mut()` for mutation; works because `(u32, u32)` is `Copy` so `.collect()` copies values, not references; the `HashSet` is then mutated mid-loop to track claims made within the same frame
- **Separation steering:** `separate_colonists` runs `.before(move_character)` each frame; same upper-triangle force pattern as `separate_enemies` — snapshots world positions, computes repulsion for each pair, applies via axis-separated wall collision using `tile_at`; uses `ENEMY_SEPARATION_STRENGTH` constant; handles the visual overlap that arrival-time detection cannot (colonists approaching the same tile before arrival triggers)
- **`tile_at` helper:** lives in `characters.rs`, same logic as in `enemy.rs` — converts a world `Vec2` to a grid coordinate, returns `None` if out of bounds or impassable; used by `separate_colonists` for wall collision
- **System ordering:** `separate_colonists.before(move_character)` and `move_to_click.before(move_character)` — explicit `.before()` constraints, not `.chain()`
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
- **Spawn:** `spawn_enemy` lives in `enemy_spawner.rs` under `EnemySpawnerPlugin`; enemy bundle is `Enemy`, `GridPosition`, `Health::new(ENEMY_HEALTH)`, `Speed(ENEMY_SPEED)`, `Sprite`, `Transform`; texture handle must be `.clone()`d for every spawn call since `Handle<Image>` is moved on first use; `GridPosition` and `Transform` must be initialised from the same grid coordinates; `ENEMY_HEALTH` is a constant in `constants.rs`

### Buildings

- **`BuildingPlugin`** lives in `buildings/buildings.rs` — registers `TileChangedEvent` and three systems: `place_wall_on_click`, `on_tile_change`, `on_tile_passability_change`
- **`TileChangedEvent { x, y }`** — fired whenever a tile's type changes; all downstream reactions (visuals, pathfinding) are driven by listeners on this event rather than being inlined at the change site
- **`place_wall_on_click`** — handles left-click input only; converts cursor to grid via `cursor_to_grid`, mutates `Map`, fires `TileChangedEvent`; no pathfinding or rendering logic
- **`on_tile_change`** — visual listener; updates `TileTextureIndex` on the tilemap entity for the changed tile; `TileStorage::single()` is resolved once before the event loop
- **`on_tile_passability_change`** — pathfinding listener; rebuilds the colonist flow field once per frame (regardless of how many tile-change events fired), then clears any colonist `Path` that contains the changed tile coordinate; uses `Local<Vec<(u32,u32)>>` as a reusable positions buffer
- **`cursor_to_grid`** — shared utility in `map/map.rs`; takes `&Camera`, `&GlobalTransform`, `Vec2` cursor pos, `&Map`; returns `Option<(u32, u32)>`; used by both `place_wall_on_click` and `move_to_click` in `characters.rs`

### Rendering

- **Texture sampler** — `DefaultPlugins.set(ImagePlugin::default_nearest())` in `main.rs` sets nearest-neighbor sampling globally; this is required for pixel art to remain crisp at all zoom levels — Bevy's default bilinear sampler blurs upscaled sprites; since the entire game is pixel art the global setting is correct and no per-asset override is needed
- **Tilemap transform offset** — `bevy_ecs_tilemap` centers each tile's sprite at its grid position in local space, so tile `(0,0)` is centered at the tilemap entity's `Transform` origin. To align with the character/gizmo coordinate system (where tile centers sit at `tx * TILE_SIZE + TILE_SIZE/2 - MAP_WIDTH * TILE_SIZE/2`), the tilemap transform must be `-(map.width * TILE_SIZE)/2 + TILE_SIZE/2` in x and the same in y — without the `+ TILE_SIZE/2` the tiles appear shifted half a tile left/down relative to all other coordinate systems

### Camera

- **Spawn** — `setup` is a `Startup` system that spawns `Camera2d`; must be registered in `CameraPlugin::build` or nothing renders
- **Zoom** — `zoom_camera` reads `Res<AccumulatedMouseScroll>` and applies a multiplicative scale change to `OrthographicProjection.scale` each frame: `scale = (scale * (1.0 - delta.y * sensitivity)).clamp(0.3, 3.0)`; sensitivity is `0.1`; multiplicative scaling feels consistent at all zoom levels; lower clamp bound is a UX decision — nearest-neighbor keeps pixels crisp but extreme zoom-in shows very large blocky pixels, so the clamp prevents unintentionally rough-looking close-up views
- **Pan** — `pan_camera` checks `ButtonInput<MouseButton>::pressed(Middle)` and reads `Res<AccumulatedMouseMotion>`; translates the camera by the mouse delta multiplied by `ortho.scale` so panning speed stays consistent regardless of zoom level; x is negated (drag right = pan left), y is added as-is (screen and world y felt correct without negation)
- **Projection access** — `OrthographicProjection` is not a standalone component in Bevy 0.18; access it via `Query<(&mut Transform, &Projection)>` and match `Projection::Orthographic(ref mut ortho)` to read or write `ortho.scale`
- **Input resources** — Bevy 0.18 provides `AccumulatedMouseScroll` and `AccumulatedMouseMotion` as frame-accumulated resources; prefer these over `EventReader<MouseWheel>`/`EventReader<MouseMotion>` for per-frame input reading

### Audio

- **Entity-based audio** — Bevy 0.15+ replaced the `Audio` resource with a component model; playing audio means spawning an entity with `AudioPlayer` and `PlaybackSettings` components; despawning the entity stops playback
- **Ambient music** — loaded and spawned once in a `Startup` system in `systems/sound.rs`; `PlaybackSettings::LOOP` keeps it running for the lifetime of the app
- **Asset paths** — `AssetServer::load` paths are relative to the `assets/` folder and must never include `assets/` as a prefix — Bevy prepends it automatically; capitalisation must match the filesystem exactly

### Tile System

- **Hybrid approach:** map data lives in a `Resource` (flat array, indexed `x + y * width`), visuals are entities/tilemap, dynamic actors (colonists, enemies, buildings) are entities with grid position components
- **Tiles are destructible and buildable** — walls can be broken by players and enemies, floors can be built on
- **Tile changes:** mutate the map resource → fire `TileChangedEvent { x, y }` → two listeners react: `on_tile_change` updates the tilemap visual (texture index), `on_tile_passability_change` rebuilds the colonist flow field and clears any colonist `Path` that passes through the changed tile; adding new tile-change sources (enemy wall breaks, deconstruction) only requires firing the event — listeners handle all downstream effects automatically
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