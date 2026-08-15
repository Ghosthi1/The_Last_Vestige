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

- Engine: Bevy 0.19.0
- Camera: orthographic top-down
- Platform: desktop only
- Entry point: `src/main.rs`

## Code Organisation

Files have specific, focused responsibilities — keep things clean and tidy. One file should not do many unrelated jobs. Prefer one plugin per feature area.

## Module Structure

```
src/
  main.rs          # App entry point, startup systems
  constants.rs     # Shared constants — TILE_SIZE, MAP_WIDTH, MAP_HEIGHT, OFFSETS, ENEMY_SPEED, ENEMY_STOP_RADIUS, ENEMY_SEPARATION_STRENGTH, ENEMY_HEALTH, ENEMY_RANGE, ENEMY_DAMAGE, COLONIST_HEALTH, COLONIST_SPEED, COLONIST_RANGE, COLONIST_DAMAGE
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
    combat.rs      # Health component — private current/max f32 fields; new(max), change_health(delta), is_dead(); Attacker component — damage/range/cooldown, new(damage, range, timer); Target(Option<Entity>) tuple struct — colonist-only, holds player-assigned attack target
    selected.rs    # Selected marker component — zero-sized, tags player-selected colonists
  colonists/
    mod.rs              # Declares submodules, re-exports Colonist, CharacterPlugin, ColonistSpawnerPlugin, SelectionPlugin
    characters.rs       # Colonist marker component; CharacterPlugin; separate_colonists, move_character, move_to_click systems; tile_at helper
    colonist_spawner.rs # ColonistSpawnerPlugin; spawn_colonist Startup system — spawns colonist bundle (Colonist, GridPosition, Health, Speed, Sprite, Transform, Path, Attacker, Target) at two hardcoded grid positions
    selection.rs        # SelectionPlugin; DragSelection resource (drag_start, is_dragging); dragselection system (click-select nearest colonist in range, or drag-box-select via Rect); draw_selection_indicator system (gizmo circle on selected colonists)
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
  combat/
    mod.rs         # Declares submodule, re-exports CombatPlugin
    combat.rs      # CombatPlugin; enemy_attack system (nearest colonist auto-targeting); colonist_attack system (spot mode via Target or passive nearest-enemy-in-range)
  death/
    mod.rs         # Declares submodule, re-exports DeathPlugin
    death.rs       # DeathPlugin; tag_dead system (generic, ordered .after(colonist_attack).after(enemy_attack)); dead_enemies_handler, dead_colonists_handler (each .after(tag_dead)) — split out of combat/ so unrelated plugins (loot, animation, morale) can order against tag_dead without depending on combat internals
  ui/
    mod.rs         # Declares hud submodule, re-exports UiPlugin
    hud.rs         # UiPlugin; spawn_hud_root Startup system — spawns the full-screen root Node all HUD panels are children of
```

### Assets

- `assets/Floors/tilesheet_6x3_128px.png` — spritesheet, 6 columns × 3 rows of 128×128 tiles: row 0 = 6 floor visual variants, row 1 = wall, row 2 = door (closed, open, locked columns — locked reserved, not wired up yet). Replaces the old `PlaceHolder_tileset.png` placeholder (no longer referenced anywhere in `src/`). `TILE_SIZE = 128.0` defined in `src/constants.rs` as a shared `pub const`, imported via `use crate::constants::TILE_SIZE` wherever tile sizing is needed
- `assets/enemeys/Spiders/Grunt.png` — sprite for the Grunt enemy; loaded via `AssetServer` in `spawn_enemy` and set on the `Sprite` `image` field; `custom_size` is `Vec2::splat(TILE_SIZE)` but the grunt is intentionally drawn smaller than the canvas for visual style — hitbox size will be defined independently when collision is added
- `assets/Sound/Background/ambient_spaceship.ogg` — looping ambient soundtrack; loaded and spawned as an audio entity in `Systems/ambient.rs` via `AmbientPlugin`
- `assets/Colonists/Knight/` — knight sprite sheets, replacing the old single-image `Knight_1.png`/`Knight_1-Sheet.png`/`Knight_1.aseprite` (deleted). Two skin colours (silver, bronze) are baked into each sheet rather than being separate files — see [Character Animation](#character-animation) for how they're indexed:
  - `space_knight_attack_sheet_4x2_128px.png`, `space_knight_carry_sheet_4x2_128px.png`, `space_knight_idle_sheet_4x2_128px.png`, `space_knight_walk_sheet_4x2_128px.png`, `space_knight_work_sheet_4x2_128px.png` — all 512×256, 4 columns × 2 rows of 128px tiles; row 0 = silver skin (4-frame cycle), row 1 = bronze skin (4-frame cycle)
  - `space_knight_sleep_sheet_2x1_128px.png` — 256×128, 2 columns × 1 row; column 0 = silver (single static frame), column 1 = bronze (single static frame) — not animated, unlike the other five
  - `space_knight_bronze.png`, `space_knight_silver_v2.png` — reference art for the developer only, not loaded by the game; not wired into code and not intended to be

## Architecture Decisions

### Pathfinding

- **A\* implementation** lives in `ai/a_star.rs` — `find_path` takes a `&Map`, start, and goal as `(u32, u32)` grid coords, returns `Option<Vec<(u32, u32)>>` where `None` means no path exists; the vec includes both start and goal; two private helpers: `idx(x, y, width)` converts 2D grid coords to a flat array index, `reconstruct_path(came_from, goal, width)` walks the `came_from` array backwards from goal to start and reverses the result
- **8-directional movement** with Chebyshev heuristic (`max(dx, dy)`)
- **Passability** is derived from `TileType` via `is_passable()` — no separate field, so it's always in sync with tile state
- **Lazy deletion** pattern for the open set — duplicate nodes are allowed in the heap, skipped via `closed_set`; `g_scores` prevents `came_from` being overwritten by worse paths
- **Non-uniform movement cost** — cardinal moves cost 10, diagonal moves cost 14 (approximating √2 × 10); this naturally produces visually direct paths by making unnecessary diagonals more expensive. Heuristic is scaled by 10 to remain admissible.
- **`find_path` validates inputs upfront** — returns `None` immediately if start or goal are out of bounds, or if the goal tile is impassable; the expensive search is never started in those cases
- **No diagonal corner-cutting** — when expanding a diagonal neighbour `(nx, ny)` from current node `(x, y)`, both bordering cardinal tiles must also be passable: `(nx, node.pos.1)` and `(node.pos.0, ny)`; if either is a wall the diagonal is skipped; prevents paths that squeeze through the gap between two diagonally adjacent walls

### Flow Fields

- **Purpose:** shares one BFS computation across all entities targeting the same goal — every reachable tile gets a direction pointing toward the goal, so N colonists pay O(map) not O(N × path)
- **`FlowLayer` enum** — defined but not yet wired up; intended for future dynamic layer selection (e.g. passing a layer type to a system rather than accessing fields directly by name)
- **`FlowField` struct** holds `width`, `height`, `directions: Vec<Option<(i8, i8)>>`, `cost_so_far: Vec<u32>`, `valid_goals: Vec<(u32, u32)>`, and `open_set: BinaryHeap<(Reverse<u32>, u32, u32)>` — `directions` is `None` for impassable/unreachable tiles, `Some((0,0))` for the goal tile itself; `cost_so_far`, `valid_goals`, and `open_set` are all reusable buffers stored as fields and cleared at the start of each rebuild to avoid per-call heap allocation
- **`build_flow_fields(&mut self, map, goals)`** takes a slice of goal positions — seeds all goals into the heap at cost 0 so the Dijkstra expands from all simultaneously; each tile's direction points toward whichever goal is cheapest to reach. Same 10/14 cardinal/diagonal cost model as A* for consistency
- **Multiple goals:** seeding multiple positions at cost 0 before the loop is all that's needed — the BFS naturally produces a "nearest goal" field for free
- **Lazy deletion** — same pattern as A*: duplicate heap entries are allowed, stale ones skipped via `cost_so_far` comparison on pop
- **`OFFSETS` constant** lives in `constants.rs` and is shared with `a_star.rs` — both use the same 8-directional neighbourhood
- **`build_flow_fields` validates goals upfront** — filters goals into a `valid_goals` vec before seeding; skips invalid or impassable goals so they are never seeded; returns early if no valid goals remain
- **No diagonal corner-cutting** — when expanding a diagonal neighbour `(nx, ny)` from current tile `(x, y)`, both bordering cardinal tiles must be passable: `(cx, y)` and `(x, cy)` where `cx = x + dx` and `cy = y + dy`; if either is a wall the diagonal is skipped; same rule as A* for consistency
- **`FlowFields` resource** has named fields (`colonists`, `structures`, `walls`) — one `FlowField` per layer, accessed directly without hashing; implements `Default` using `MAP_WIDTH`/`MAP_HEIGHT` so new layers only require adding a field and a line to the `Default` impl; inserted in `main.rs` as `FlowFields::default()`
- **`AiPlugin`** in `ai/ai_plugins.rs` owns the rebuild system — `pub fn rebuild_colonist_flow_field` runs every `Update` with two queries: both filtered `With<Colonist>` so enemies with `GridPosition` are never included as goals; one also filtered on `Changed<GridPosition>` as a cheap early-return gate; a `RemovedComponents<Colonist>` parameter is also read (and fully drained, not just peeked) each frame the gate is checked, since despawning a colonist doesn't set `Changed` on any surviving entity — without this, the field went stale on colonist death and only recovered once some other colonist happened to move and re-trigger the `Changed<GridPosition>` gate; uses `Local<Vec<(u32,u32)>>` for the positions buffer so it is allocated once and reused each frame; rebuilds the `colonists` field directly
- **Rebuild trigger:** two triggers now feed the gate — `GridPosition` is written in `move_character` (`grid_pos.0 = *next`) when a colonist arrives at a waypoint, marking the component changed; and a colonist despawning (death) is caught via `RemovedComponents<Colonist>`, since removal isn't a mutation Bevy's `Changed` filter can see
- **Layer design:** `FlowFields` fields represent targets things navigate *toward* — `colonists` means "goal is colonist positions, used by enemies"; colonists themselves use A* for player-directed movement; layers are accessed directly by field name (`flow_fields.colonists`) rather than via `FlowLayer` dispatch

### Combat

- **`Health` component** — lives in `components/combat.rs`; private `current: f32` and `max: f32` fields; constructed via `Health::new(max)` which sets `current = max` and `debug_assert!`s `max > 0.0`; fields are private so all access goes through methods
- **`change_health(delta: f32)`** — adds `delta` to `current` (negative for damage, positive for healing); clamps `current` to `[0.0, max]`; when `current` hits `0.0` the entity is considered dead — `Health` itself never reacts to this, detection is handled entirely by an external system (`tag_dead_enemies`, see below)
- **`is_dead() -> bool`** — returns `self.current <= 0.0`; called by an external system that queries for dead entities and handles despawning, tile events, and animations — `Health` itself cannot interact with the world
- **Death handling pattern** — `Health` is pure data; death detection is a single generic system (`tag_dead`) that queries any entity with `Health`, calls `is_dead()`, and inserts a `Dead` marker component, regardless of entity type; reaction to the marker is split by type — `dead_enemies_handler` and `dead_colonists_handler` each react to `Dead` scoped to their own entity type and currently just despawn, with a `TODO` on each for future type-specific behaviour (death animation, loot drop, morale/game-state effects); implemented for both enemies and colonists — no movement/AI queries filter on `Without<Dead>` yet since nothing currently needs to
- **`Dead` marker component** — zero-sized component in `components/combat.rs` alongside `Health` (stays here for now — not moved to `death/` since it's plain data with no plugin of its own; revisit if a non-combat death cause, e.g. hunger, is ever added); inserted by the generic `tag_dead` system (no entity-type filter); consumed separately by `dead_enemies_handler` (`With<Dead>, With<Enemy>`) and `dead_colonists_handler` (`With<Dead>, With<Colonist>`) — tagging is generic, reaction is split by type since colonist and enemy death are expected to diverge in behaviour
- **`Attacker` component** — lives in `components/combat.rs`; private `damage: f32`, `range: f32`, `cooldown: Timer` fields; constructed via `Attacker::new(damage, range, timer)`; range is in world units (multiply tile counts by `TILE_SIZE`); cooldown is a `Timer::from_seconds(n, TimerMode::Repeating)` — ticked every frame regardless of range so the first hit after closing distance doesn't fire instantly; both colonists and enemies carry this component
- **`Target` component** — lives in `components/combat.rs`; tuple struct `Target(pub Option<Entity>)`; colonist-only; `None` = passive mode (attack nearest enemy in range), `Some(entity)` = spot mode (wait for that specific enemy to enter range); stale entity handling is implemented — in `colonist_attack`, the `Some(entity)` arm falls back to `target.0 = None` when the target is not found in that frame's `enemy_snapshot` (e.g. the target despawned), so a colonist whose spot-target dies drops back to passive/nearest-enemy mode instead of going idle
- **`tag_dead` system** — lives in `death/death.rs` under `DeathPlugin` (moved out of `combat/`); queries `(Entity, &Health)` filtered `Without<Dead>, Changed<Health>` — no entity-type filter, so it tags any dead entity regardless of type; `Changed<Health>` means it only runs against entities that took damage (or were healed) this frame rather than scanning the whole population; inserts `Dead` when `health.is_dead()`; registered with both `.after(colonist_attack)` and `.after(enemy_attack)` chained — it must run after both, since either can zero out a `Health` it doesn't itself own (colonists damage enemies, enemies damage colonists); registering it twice with one `.after()` each was tried and rejected — Bevy doesn't deduplicate a function added via separate `add_systems` calls, so that produced two system instances running redundantly every frame; `colonist_attack`/`enemy_attack` are imported from `crate::combat` purely for the ordering call — `DeathPlugin` doesn't otherwise depend on combat internals, and this cross-plugin `.after()` is the pattern any future plugin (loot, animation, morale) should follow to order against death without depending on combat
- **`dead_enemies_handler` / `dead_colonists_handler` systems** — live in `death/death.rs` alongside `tag_dead`, not split into `enemys/`/`colonists/` yet since both are still despawn-only and identical in shape — move each into its owning feature module once it grows real type-specific behaviour (loot table on enemy death, say); each queries `Entity` filtered `With<Dead>` plus their respective `With<Enemy>`/`With<Colonist>`; both currently just despawn; both registered `.after(tag_dead)`; kept as separate systems from tagging (rather than combined) so future type-specific reactions (death animation, loot drop for enemies; morale/game-state effects for colonists) can be filled in independently without restructuring `tag_dead` — each has a `TODO` marking that it's despawn-only for now
- **Why split from `combat/`** — loot, animation, morale, and job-cancellation reactions to death all need query shapes or cross-frame state unrelated to combat (e.g. morale needs to iterate *other* colonists, not the dead entity); giving `tag_dead` its own plugin lets those future plugins order `.after(tag_dead)` without depending on `CombatPlugin` at all

### Characters

- **Components:** `GridPosition((u32, u32))` — authoritative grid position (inner tuple), lives in `components/movement.rs`; `Path(VecDeque<(u32,u32)>)` — remaining waypoints, lives in `components/movement.rs`; `Speed(f32)` — movement speed in tiles per second, lives in `components/movement.rs`; `Health` — current/max health, lives in `components/combat.rs`; `Colonist` — zero-sized marker in `character/characters.rs`, filters colonist-only queries so enemies are never accidentally included
- **Colonist bundle:** `Colonist`, `GridPosition`, `Health::new(COLONIST_HEALTH)`, `Speed(COLONIST_SPEED)`, `Sprite`, `Transform`, `Path`, `Attacker::new(COLONIST_DAMAGE, COLONIST_RANGE, Timer)`, `Target(None)` — constants in `constants.rs`
- **Smooth movement:** `move_character` advances `Transform` toward the next waypoint each frame using `move_towards(target, speed * delta_secs)`; `GridPosition` is only updated when the character arrives at a waypoint (`distance_squared < 0.01`, avoiding a sqrt); `transform.translation` is only snapped to the tile center on normal arrival — not in the conflict branch, so the transform never visually lands on an occupied tile
- **Click-to-move:** `move_to_click` calls `cursor_to_grid` (shared utility in `map/map.rs`) to convert cursor window position → grid coordinates; start position is `path.0.front()` if a path is already in progress, otherwise `grid_pos.0` — keeps movement smooth mid-path by continuing from the current waypoint rather than snapping back to grid position
- **Click-time goal assignment:** before the assignment loop, `move_to_click` snapshots all colonist `GridPosition`s into a `mut HashSet<(u32, u32)>`; for each colonist, if the clicked goal is already in the set, searches 8 neighbours of the goal for a free passable tile; uses `actual_goal` (the neighbour, or the original goal if free) for `find_path`; inserts `actual_goal` into the set so subsequent colonists in the same click get distinct targets — prevents multiple colonists converging on the same world position
- **Arrival-time conflict detection:** `move_character` snapshots all `GridPosition`s into a `mut HashSet` before the movement loop; at arrival, checks `occupied.contains(next) && *next != grid_pos.0` — the guard excludes the start-waypoint case where `next` equals the colonist's current tile (which appears in `occupied` as their own position); if occupied, searches 8 neighbours of `next` for a free passable tile and replaces `path.0[0]` with it without updating `grid_pos` or snapping transform; on normal arrival, inserts the arrived tile into `occupied` so same-frame arrivals at the same tile are caught by subsequent iterations
- **Separation steering:** `separate_colonists` runs `.before(move_character)` each frame; same upper-triangle force pattern as `separate_enemies` — snapshots world positions, computes repulsion for each pair, applies via axis-separated wall collision using `tile_at`; uses `ENEMY_SEPARATION_STRENGTH` constant; handles the visual overlap that arrival-time detection cannot (colonists approaching the same tile before arrival triggers)
- **`tile_at` helper:** lives in `characters.rs`, same logic as in `enemy.rs` — converts a world `Vec2` to a grid coordinate, returns `None` if out of bounds or impassable; used by `separate_colonists` for wall collision
- **System ordering:** `separate_colonists.before(move_character)` and `move_to_click.before(move_character)` — explicit `.before()` constraints, not `.chain()`
- **Tilemap offset:** the tilemap is centered on screen — tile world position = `tile_coord * TILE_SIZE + TILE_SIZE/2 - map_size * TILE_SIZE/2`; this places entities at the **center** of each tile; all coordinate conversions must account for this

### Character Animation

- **Status: planned, not yet implemented** — no `TextureAtlas`/`TextureAtlasLayout` usage exists anywhere in `src/` yet; `spawn_colonist` still sets a single static `Sprite.image`. This section records the agreed design ahead of implementation
- **Mechanism** — Bevy 0.19's `Sprite` has a `texture_atlas: Option<TextureAtlas>` field, where `TextureAtlas { layout: Handle<TextureAtlasLayout>, index: usize }`. Changing which sub-frame is shown = changing `index`. Changing which *animation* is shown (idle → walk) = swapping `Sprite.image` itself, since each animation state lives in its own sheet file (see [Assets](#assets))
- **Two shared layouts, not six** — `attack`/`carry`/`idle`/`walk`/`work` all share identical grid geometry (4 cols × 2 rows × 128px), so one `TextureAtlasLayout` built via `TextureAtlasLayout::from_grid` covers all five; `sleep`'s 2×1 grid needs a second, separate layout. A layout is index math only — it doesn't care which image it's paired with
- **Row/column = skin colour, not direction** — on the 4×2 sheets, silver is row 0 (atlas indices 0–3), bronze is row 1 (indices 4–7); on `sleep`, silver = index 0, bronze = index 1 (single static frame each)
- **Planned pieces (not yet built):**
  - a resource holding the six `Handle<Image>`s plus the two `Handle<TextureAtlasLayout>`s, populated once in a `Startup` system — loaded once, not per-colonist, same rationale as texture handle `.clone()`ing in `spawn_enemy`
  - a component identifying which animation state a colonist is in (idle/walk/attack/carry/work/sleep)
  - a component identifying skin colour (silver/bronze), driving the row/index offset
  - a per-entity animation timer that advances the frame index each tick and wraps within the current state+skin's 4-frame range (or stays fixed at 1 frame for `sleep`)
  - a system that, on state change, swaps `Sprite.image` and `TextureAtlas.layout`/`index` together — swapping only `index` is not enough when the state change also changes which sheet is active

### Selection

- **`Selected` marker component** — zero-sized, lives in `components/selected.rs`; tags the currently-selected colonist(s); inserted/removed by `dragselection`, read by `draw_selection_indicator`
- **`DragSelection` resource** — lives in `colonists/selection.rs`; `#[derive(Resource, Default)]`; `drag_start: Option<Vec2>` (window-space cursor position where the left mouse button went down), `is_dragging: bool`
- **`dragselection` system** — disambiguates click vs. drag using a `5.0`-pixel movement threshold measured from `drag_start`: on `just_pressed`, records `drag_start`; while `pressed`, sets `is_dragging` once the cursor moves past the threshold and draws a live selection-box gizmo (`gizmos.rect_2d`) between `drag_start` and the current cursor, both converted to world space via `viewport_to_world_2d`; on `just_released`, branches on `is_dragging`
- **Click-select (not dragging):** converts cursor to world space, finds the nearest colonist by `Transform` distance, and selects it only if within `TILE_SIZE * 0.6` — clicking empty space clears selection without picking a colonist
- **Drag-select (dragging):** builds a `Rect::from_corners(start_world, end_world)` and selects every colonist whose `Transform` translation falls inside it via `rect.contains()`
- **Replace, not add:** both branches clear `Selected` from every colonist before applying the new selection — there is no additive/shift-click multi-select yet, a single click or drag always replaces the prior selection
- **`draw_selection_indicator` system** — queries `Transform` filtered `With<Selected>`, draws a gizmo circle (`gizmos.circle_2d`, radius `TILE_SIZE * 0.5`) at each selected colonist's position every frame
- **`SelectionPlugin`** — registers both systems on `Update`; no explicit ordering between them and `move_character`/`move_to_click` since selection only reads `Transform`, never mutates it

### Enemies

- **`Enemy` marker component** — zero-sized, lives in `enemys/enemy.rs`; used to filter enemy-only queries and distinguish enemies from colonists who share `GridPosition` and `Speed`
- **Continuous movement** — enemies move in world space, not tile-to-tile; `Transform` is authoritative, `GridPosition` is derived from it each frame by `(translation + offset) / TILE_SIZE`, floored to `u32`; this allows more than 8 enemies to surround a single colonist
- **Flow-field movement** — each frame `move_enemy` looks up the flow field direction for the enemy's current `GridPosition`, converts the `(i8, i8)` to a normalised `Vec2`, scales by `speed * delta_secs`, and applies via axis-separated wall collision (see below); normalisation ensures diagonal movement is not faster than cardinal
- **Colonist proximity stop** — before applying velocity, `move_enemy` collects all colonist positions into a `Vec<Vec2>` snapshot once before the enemy loop, then checks if any colonist is within `TILE_SIZE * ENEMY_STOP_RADIUS` (distance squared); if so, the enemy stops moving that frame; `ENEMY_STOP_RADIUS = 0.7` lives in `constants.rs`
- **Separation steering** — `separate_enemies` runs `.before(move_enemy)` each frame; snapshots all enemy positions into a `Vec<Vec2>`, then iterates the upper triangle of pairs (`j > i`) to compute repulsion once per pair and accumulate into a `forces` vec — `forces[i] += force`, `forces[j] -= force`; force uses a smooth linear falloff `(1.0 - dist / TILE_SIZE) * diff / dist`, reusing the single `length()` call; a second pass applies `forces[i]` to each transform via axis-separated wall collision; `ENEMY_SEPARATION_STRENGTH = 10.0` lives in `constants.rs`
- **Axis-separated wall collision** — both `move_enemy` and `separate_enemies` apply movement one axis at a time; before adding a delta to `transform.translation.x`, a test position `(current_pos + Vec2::new(delta_x, 0.0))` is passed to `tile_at`; if it returns `None` (wall or out of bounds) the x movement is skipped; same check independently for y; this prevents enemies being pushed into walls by separation forces while still allowing sliding along wall faces — the `tile_at` helper in `enemy.rs` converts a world `Vec2` to a tile coordinate, returning `None` if out of bounds or impassable
- **Query disjointness** — `move_enemy` accesses `&mut Transform` for enemies and `&Transform` for colonists; Bevy requires explicit `Without<Colonist>` on the enemy query and `Without<Enemy>` on the colonist query to prove they never overlap, otherwise it panics with `B0001` at startup
- **System ordering:** `separate_enemies.before(move_enemy)`, `move_enemy.after(rebuild_colonist_flow_field)` — separation is applied before flow-field movement each frame; flow field is always current before enemies read it
- **Spawn:** `spawn_enemy` lives in `enemy_spawner.rs` under `EnemySpawnerPlugin`; enemy bundle is `Enemy`, `GridPosition`, `Health::new(ENEMY_HEALTH)`, `Speed(ENEMY_SPEED)`, `Sprite`, `Transform`, `Attacker::new(ENEMY_DAMAGE, ENEMY_RANGE, Timer)`; texture handle must be `.clone()`d for every spawn call since `Handle<Image>` is moved on first use; `GridPosition` and `Transform` must be initialised from the same grid coordinates; constants in `constants.rs`

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

### Floor Variants

- **Cosmetic variation, not tile type** — `TileData` (`map/map.rs`) has a `floor_variant: u32` field alongside `tile_type`; it selects which of 6 floor art variants a `Floor` tile renders as, entirely independent of `TileType` — walls and doors have no variants
- **`texture_index()` lives on `TileData`, not `TileType`** — moved off `TileType` since computing the final sheet index needs both `tile_type` and `floor_variant` together; `TileType` alone can no longer answer "what's my texture." `TileType::is_passable()` stays on `TileType` since passability only depends on the type, not the variant
- **Row-major sheet layout** — `assets/Floors/tilesheet_6x3_128px.png` is 6 columns × 3 rows of 128px tiles; `bevy_ecs_tilemap` slices a `TilemapTexture::Single` image row-major, so `index = row * 6 + column`. Row 0 = floor variants (columns 0–5 map directly to `floor_variant`), row 1 = wall (fixed at index `6`, no variants yet), row 2 = door (column 0 = closed → index `12`, column 1 = open → index `13`; column 2 is reserved for a locked state but `TileType::Door` only carries `is_open: bool` so far, so locked is unimplemented)
- **Generation:** `generate_map` (`map/map_gen.rs`) rolls a weighted-random `floor_variant` per tile via `rand::distr::weighted::WeightedIndex` (weights `[50, 10, 10, 10, 1, 10]` — variant 0 dominant, the rest sprinkled in, one intentionally rarer) and `rand::rng()` for the thread-local source; `WeightedIndex::sample` requires the `rand::prelude::Distribution` trait in scope

### HUD / UI

- **Tool choice: native `bevy_ui`** — considered `bevy_egui` (rejected: immediate-mode look reads as editor/debug UI, not a diegetic in-game HUD) and `bevy_lunex` (rejected for now: better suited to non-rectangular/angular HUD layout, but a smaller third-party crate carrying the same version-lag risk that blocked the Bevy 0.19 upgrade with `bevy_ecs_tilemap`); modern `bevy_ui` (rounded corners, box shadows, gradients) covers the sci-fi panel look via 9-slice panel textures and translucent `BackgroundColor`s without that dependency risk — a custom `UiMaterial`/WGSL shader is the fallback for animated effects (scanlines, pulsing glow) if plain nodes aren't enough
- **Root node pattern** — `UiPlugin::build` registers `spawn_hud_root` on `Startup`; it spawns a single entity with `Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }` as the parent every HUD panel will be a child of; `BackgroundColor` was added temporarily to visually confirm the node's placement/size, then removed — the root itself must stay invisible (an opaque full-screen `BackgroundColor` blocks the world view underneath), visible color belongs only on child panels sized to their own content
- **Resource bar categorisation** — the top bar is split by urgency rather than shown as one flat row of icons: **life-support** (oxygen, food, water, energy) is always visible and built first, since zero on any of these means colonists start dying; **population** is a colony-status count, not a depletable resource, so it gets its own slot rather than sitting in the resource-bar style; **stockpile materials** (scrap metal, refined metals, bullets) are deferred — likely belong in a build/loadout panel where they're actually spent rather than the always-on HUD bar, with ammo count as a possible exception since combat is real-time
- **`LifeSupport` resource** — lives in `ui/hud.rs`; private `oxygen: f32`, `food: f32`, `water: f32`, `energy: f32` fields, `#[derive(Resource)]`; starting values (`100.0` each) are set via a `Default` impl rather than a struct literal, since the fields are private and `main.rs` (a different module) can't construct a struct literal with private fields — `main.rs` inserts it via `.insert_resource(LifeSupport::default())`, same shape as the existing `FlowFields::default()` line; no accessor methods yet since nothing outside `hud.rs` reads or mutates it
- **Current status:** `LifeSupport` resource in place and inserted; next up is spawning the four life-support values as `Text` children of the root node (`spawn_hud_root` gains a `Res<LifeSupport>` param, children added via `.with_children(...)`), each tagged with its own marker component (`OxygenText`, `FoodText`, `WaterText`, `EnergyText`) so a later update-on-change system can find the right entity to update when `LifeSupport` changes; text display is plain `"Label: value"` strings — no bars or icons for the life-support group

## Bevy 0.19 Upgrade Notes

Upgraded from 0.18.1 to **Bevy 0.19.0** (`bevy_ecs_tilemap` bumped to `0.19.0` alongside it, resolving the prior blocker). `cargo build` is clean with no source changes required beyond `Cargo.toml`.

- **Audio feature no longer implied** — actioned: `audio` added to the `bevy` features list in `Cargo.toml` explicitly (`features = ["dynamic_linking", "audio"]`); without it `ambient_spaceship.ogg` would silently stop playing
- **Resources as Components** — no impact observed; no query in this codebase is broad enough (e.g. `Query<()>`) to collide with resource entities
- **`Assets::get_mut` return type changed** — no impact; this project doesn't call `.get_mut()` on an `Assets<T>` resource
- **Scene system renamed** (`bevy_scene` → `bevy_world_serialization`) — no impact; scenes aren't used yet, revisit naming when they are
- **Rendering pipeline overhaul** — no impact; no custom render code in this project, `map_renderer.rs` builds and runs unaffected

If new breaking changes surface while working in 0.19, add them here.

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
- **Warn about bad designs early** — if a design direction will cause pain (especially Bevy ECS anti-patterns common in colony/sim games e.g. storing too much state in single entities, overusing Resources instead of Components), raise it before they build too far
- **Wait for the developer to drive** — don't suggest next steps or features unprompted
- **Keep responses short and low-density** — favor short, direct answers over exhaustive ones; explain the why, but in tight bullets or short sentences — never padded paragraphs, even in summaries or status reports
- **Give multi-step instructions one step at a time** — when a change spans multiple files, steps, or distinct concepts (even within a single file), give a single step, then wait for the developer to confirm or complete it before giving the next; don't dump the whole sequence in one response. This applies within a single explanation too — if a step has more than one distinct part (e.g. "the tool to use" and "where it plugs in"), split those into separate turns rather than bundling them
- **Be more explicit and concrete, not just brief** — short is good, but don't compress to the point of vagueness; name the exact type, field, function, or file involved rather than describing it abstractly. Concise and clear beat concise and terse

## Planned Features / TODO

- **Sprite animations / sprite sheets** — in progress for the knight colonist, see [Character Animation](#character-animation); art is in (`assets/Colonists/Knight/`, idle/walk/attack/carry/work/sleep, silver + bronze skins), wiring (`TextureAtlas`, state/skin components, timer system) is designed but not yet built; enemies and other colonist types still need art + wiring from scratch
- **Game saves (binary serialization)** — serialize world state (map, colonist positions/health, enemy state, buildings) to a compact binary format for save/load; consider `bincode` + `serde` derives or a custom flat-buffer approach for performance
- **Spatial queries** — replace O(N²) nearest-neighbour loops in combat and separation systems with a spatial structure; candidates: KD-tree (static or semi-static entities) or spatial hashing (dynamic, grid-aligned entities like colonists and enemies)
- **Procedural generation** — expand `map_gen.rs` with proper proc-gen (BSP rooms, cellular automata caves, or wave-function collapse); hook into the future chunk system so new chunks generate on reveal
- **UI** — in progress, see [HUD / UI](#hud--ui); tool decided (`bevy_ui`), root node in place, life-support resource group underway; still open: population slot, stockpile materials panel, colonist status bars, selected-unit info panel, minimap