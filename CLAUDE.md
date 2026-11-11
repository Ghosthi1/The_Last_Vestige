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

**The Last Vestige** — a 2D sci-fi colony builder / horde survival game set aboard a derelict ship (see [Game Vision](#game-vision)).

- Engine: Bevy 0.20.1
- Camera: orthographic 2D, RimWorld-style view (top-down map, standing characters). Floors and walls are re-rendered top-down at 128×128; characters and the Grunt are still placeholder art (see [Art](#art-decided))
- Platforms: PC (Windows + Linux) and Steam Deck first; Xbox/PlayStation are on the back shelf
- Entry point: `src/main.rs`

The game was briefly remade in Unity DOTS and is now back on Bevy. The design decisions in [Game Vision](#game-vision) were made during that period and are the **target design**. Some of the Bevy code recorded under [Architecture Decisions](#architecture-decisions) predates them (e.g. a single small map, two hardcoded colonists, pixel-art placeholders) — where they differ, Game Vision is where the game is heading, and Architecture Decisions is what the code does today.

## Claude's Role: this is a learning project

The developer is building this to learn, and the game needs to handle **thousands of enemies** at once.

- **No code by default.** Explain concepts, approaches, and Bevy/Rust patterns so the developer writes the code themselves. This includes small inline snippets — no code at all unless the developer explicitly asks for an example. The only exception is this CLAUDE.md, which Claude keeps up to date.
- **Boilerplate only on explicit request** — e.g. plugin skeletons, empty component/resource structs, empty system stubs, `Cargo.toml` entries. Keep the logic for the developer to write.
- **One-liner fixes:** describe the change and say where it goes. Let the developer type it unless they ask Claude to.
- **Level:** Intermediate Rust, beginner Bevy — assume Rust is solid, but explain Bevy-specific concepts (ECS, systems, plugins, resources, events) thoroughly, including the why behind them
- **Always explain why** — not just what to do, but the reasoning and tradeoffs behind it. For anything on a hot path (enemies, flow fields, separation, combat), explain in performance terms: data layout, cache use, allocations, change detection, system ordering, parallelism
- **Point out problems, never fix them** — flag bugs, issues, and inefficiencies; let the developer resolve them
- **Flag working-but-suboptimal code** — if something works but is inefficient or could be done more sensibly, say so
- **Warn about bad designs early** — if a design direction will cause pain (especially Bevy ECS anti-patterns common in colony/sim games e.g. storing too much state in single entities, overusing Resources instead of Components), raise it before they build too far
- **Wait for the developer to drive** — don't suggest next steps or features unprompted. The one exception is offering a documentation pass after a finished step (see [Documentation Standards](#documentation-standards))
- **Keep responses short and low-density** — favor short, direct answers over exhaustive ones; explain the why, but in tight bullets or short sentences — never padded paragraphs, even in summaries or status reports
- **Give multi-step instructions one step at a time** — when a change spans multiple files, steps, or distinct concepts (even within a single file), give a single step, then wait for the developer to confirm or complete it before giving the next; don't dump the whole sequence in one response. This applies within a single explanation too — if a step has more than one distinct part (e.g. "the tool to use" and "where it plugs in"), split those into separate turns rather than bundling them
- **Be more explicit and concrete, not just brief** — short is good, but don't compress to the point of vagueness; name the exact type, field, function, or file involved rather than describing it abstractly. Concise and clear beat concise and terse
- **Teaching note:** a full step-by-step trace of an algorithm was confusing; a small concrete example of just the data/storage, followed by small one-step-at-a-time instructions, worked much better
- **Don't build infrastructure ahead of the system that shapes it**

## Documentation Standards

- Use `///` doc comments on all `pub` structs, enums, and methods — these show in IDE tooltips and `cargo doc`
- Doc comments on the type itself explain what it represents and how it fits the data flow (who writes it, who reads it); doc comments on methods explain what they do and return
- Fields should have `///` comments if their purpose isn't immediately obvious from the name
- Regular `//` comments are for non-obvious implementation details only — explain *why* (performance, ordering, change detection), not restate what the code does
- When code changes, update its docs and this CLAUDE.md in the same step
- The developer wants help keeping code well documented for long-term maintainability — flag missing or inadequate doc comments when reviewing code
- **Document together, not for them:** when the developer finishes a step, offer to go through the new code's docs together — file by file, talking through what each part does, with the developer deciding or writing the wording. Never document whole files in one go

## Game Vision

2D colony builder / horde survival game set aboard a **derelict gothic space-hulk-style ship** (not a planet): an amalgamation of many wrecked ships fused together, with void gaps and damaged sections inside its bounds. Humanity's last survivors build a makeshift colony inside the ship and expand outward while holding back a vast horde. It's colony builder / horde survival, not an escape.

Inspirations: RimWorld, Dwarf Fortress, Space Haven, Songs of Syx, Sir, We Have an Orc Problem (horde scale), World War Z / Space Marine 2 (horde "sea" feel).

### Performance target

- **Minimum 30 fps** on a low-end "potato" PC (an old office PC is the min-spec test bench).
- **Enemy count and all gameplay are identical across graphics settings.** Only visuals scale: particles, dust, lights/shadows, post-processing, render scale, far-sprite animation rate, shader detail.
- Rough frame budget at 30 fps: ~20 ms simulation, the rest for rendering.
- 2D bottleneck to watch: **overdraw/fill rate** (stacked transparent sprites + particles). Prefer alpha-cutout, cap particles.
- No allocations in per-frame code.
- Graphics tier auto-selected on first launch (integrated GPU → Low; don't trust VRAM on iGPUs; CPU below min spec → warning, not a tier change). Player can override.

### World & map

- **Very large but bounded** (truly infinite was considered and rejected): target ~5000×5000 tiles (round to 5120), **3 floors**, all simulated at the same time, connected by **lifts**.
- Too big to generate or simulate at once, so it's split into **chunks** (e.g. 64×64) that are created when revealed. The player **works outward** from a starting area. Chunk states: dormant (not allocated), frontier, active (fully simulated).
- Corridor widths vary: 1–2 (ducts), 3 (standard chokepoints), 5+ (arteries), open halls/bays (where the horde "sea" shows).
- **Crash events (idea, 2026-11-10, not decided):** start with a small cluster of wrecks; after the tutorial, more ships crash into the hulk at several places at once so expansion isn't linear. Same pattern as ship shakes (map change → connectivity check). A crash only needs to add ship records; tiles get stamped when a chunk is revealed. Open: where crashes are allowed (avoid or hit the colony?), whether the director triggers them, and storing each crashed ship's seed for saves.
- **Ship shakes:** random events that slightly change the map (new walls, debris). Telegraphed (rumble, dust, shadows of falling debris). Connectivity is checked after each shake so colonists aren't sealed in.
- **Enemies can destroy walls.** Walls have a high but finite pathing cost scaled by HP, so enemies chew through when the way around is long. Without shakes, the area around the base slowly gets cleared.
- **Furniture and debris don't block pathing**: they raise the cell's cost (enemies push through slowly). Only out-of-bounds cells are truly impassable. **Void (space) is passable for colonists and enemies alike, but has a high pathing cost** (decided 2026-11-10), so routes through space are a last resort rather than a first choice. Limits on colonists in space (suits, oxygen, airlocks) are still open.
- Flow-field cost includes crowd density to reduce jamming in narrow corridors.

### Procedural generation

- The ship is **procedurally generated**.
- **Generate in layers, big to small:** macro (void / hull / damaged, ship style; low-frequency noise suits this) → meso (rooms + corridors inside the hull) → micro (furniture, debris).
- Because the map is bounded, a cheap **global coarse pass** runs up front: hulk outline, which ship style each region has, lift positions (superseded: lifts are now placed per ship, see wreck generation below), and the player's starting area. Chunks then fill in the detail on demand.
- **Wreck generation (design session, 2026-11-10)** — the hulk is many wrecked ships fused together (see [Map Generation](#map-generation) for the layout direction). Decisions so far:
  - **Wrecks and floors:** wrecks have different deck counts (big 3, medium 2, small fragments 1), so upper floors shrink to just the tall ships, with void where smaller ships are. This answers "are all floors the same size?": no. Lifts can only sit in ships tall enough to reach their floors. **Built first as every wreck having all 3 decks**; varying deck counts come later without restructuring
  - **Ship classes as data:** a small set of presets (e.g. cruiser, freighter, escort, fragment). Each class sets its size range, deck count, hull shape (pointed prow, blunt, engine block) and the interior style of each deck; generation picks a class, then rolls within its ranges. Chosen so ships are recognisable and mean something to the player (a freighter = cargo bays), and so room types can later be tied to class
  - **Orientation: the 4 grid directions only.** 45° diagonals were picked, then dropped by the developer. Hull edges are always straight lines of tiles; pointed prows and wreckage in the joins break up the boxy look
  - **Starting map: a mix.** The global pass places a medium-sized hulk at the start (enough ships to expand into, and unrevealed space for the horde to travel through). The rest of the bounded map starts as open void, which crash events gradually fill with new ships in several places at once. The outer bounds never change, so chunk counts, array sizes and the abstract horde's travel area stay predictable. Supersedes the "crash events" idea under [World & map](#world--map)
  - **Deck interiors: class decides, floor biases.** There's one shared list of interior styles (the three old floor styles below, plus new ones such as cargo bay, single spine, blocks of rooms). Each class lists the styles its decks can use, and each floor nudges the odds (e.g. bottom decks lean towards machinery, top decks towards corridors). Ships keep their identity while each floor still has a general feel; setting the bias to zero gives pure class choice. The floor styles below are now entries in that list, not fixed per floor
  - **Joins: a mix of rammed and docked.** Rammed (the usual case): ships overlap and the overlap becomes rubble (debris, broken walls, patches of floor), giving wide, messy, high-cost chokepoints. Docked: ships sit close, linked by a short airlock tube or gangway, giving clean, narrow (1–3 tile) killing zones. Class data can weight which kind a ship gets (e.g. freighters often docked, warships usually rammed)
  - **Wreck damage: per-ship decay plus join damage (decided).** Each ship rolls a `decay` value from its class's range; it sets the baseline debris, breaches, missing walls and collapsed rooms across the whole ship. On top of that, joins always get heavy damage (whatever the ship's decay), plus a few random breaches. So even a low-decay ship has a wrecked area where it fused. Feeds the per-room `decay` parameter of the debris pass. Per-section damage (bow/middle/stern each intact/damaged/wrecked) was considered and left as a possible later modifier on top of this
  - **Ship sizes: wide spread (decided).** Rough lengths: fragments ~20 tiles, escorts ~40–150, freighters ~100–300, cruisers ~300–500 (several 64-tile chunks long). A few landmark capital ships surrounded by lots of small wreckage: the big ships become recognisable regions, and the small ones fill gaps and add joins (chokepoints). Fits the deck rule, since only big ships reach the upper floors, so those floors become a few large islands. Exact ranges are placeholder numbers in the class data, to tune later
  - **Starting hulk: one capital ship at the core (decided).** 1 cruiser + 3–5 mid-size ships + ~10–20 fragments, roughly 800×800 tiles. The colony starts in a smaller ship docked to the cruiser: a compact, defendable base with a narrow docked join as the first killing zone. The cruiser is close and visible but too big to hold early, so it's the natural expansion goal. The rest of the 5120 map stays void for crash events. Counts and footprint are placeholders to tune
  - **Lifts: count set by class (decided).** Each class has a lift-count range (e.g. cruiser 2–4, freighter 1–2, escort 0–1, fragment 0), rolled per ship and placed while that ship's interior is generated: reserved before rooms, spaced apart, preferably on a main corridor. If the reachability flood fill fails, re-roll that ship's lifts. Lifts belong to ships, not to the global coarse pass (supersedes that part of the coarse pass bullet above). Cruisers become the vertical hubs
  - **The colony's starting ship has no lift (decided).** The cruiser is the only way up early on, which adds to its value as the expansion goal
  - **All wreck-generation topics from this session are now covered**
  - **Mockups** from this session (throwaway Python, not game code) were made in the session scratchpad and aren't kept: fused wrecks (spine/twin-corridor/blocks interiors, rubble at overlaps, breaches), and a RimWorld-style vs isometric horde corridor comparison
- **Floor styles** (index 0 = bottom); room sizes vary within each floor:
  - **Bottom: maintenance.** A few large rooms filled with big machinery (multi-cell obstacle blocks, with a clear gap kept around each). Little corridor.
  - **Middle: rooms and organised corridors.** Medium rooms, a lot of corridor with some randomness, some dead ends, few loops. Not a literal maze: the layout should produce natural chokepoints for the horde.
  - **Top: mostly corridors**, with many small rooms.
- **One generator with different parameters per floor**, not three generators. Parameters are data-driven: room count, room size range, dead-end count, extra link (loop) count, corridor width, machinery density.
- **Algorithm (decided): connect rooms directly.** Place non-overlapping rooms → link them into a tree using nearest neighbours → carve each link as a straight or L-shaped corridor → add a few extra links for loops → add short dead-end stubs. Doors go where a corridor meets a room. Bottom-floor machinery is an extra pass. "Rooms and mazes" was tried and rejected: a true maze has too many twisting 1-wide paths for any one to stand out as a chokepoint. In a tree, every corridor is the only route between two parts of the map, and extra links decide where the horde gets alternative routes.
- **Lifts can stop on any set of floors**: 1–2, 2–3, 1–3 (skipping 2), or 1–2–3. Lifts are generated *before* rooms; every floor reserves the cells of the lifts that stop on it. The lift set must keep every floor reachable — a flood fill after generation checks this.
- **Theme is a derelict, cluttered amalgamation, not a pristine ship.** Mess is wanted: random flaws within groups (missing/shifted pieces), plus a final debris pass driven by coherent noise and a per-floor/per-room `decay` parameter. Debris = extra cost, so it affects horde routing.
- **Still open (the developer's call):**
  - Which links get which corridor width (a corridor is a rectangle, so carving any width is simple).
  - How corridors cross chunk boundaries, since links span the map; the global coarse pass could plan the main routes.
  - Room types (hospital, factory, kitchen…): suggested assigning them after layout on the room graph (greedy colouring so no same type sits next to each other, with per-floor limits, size limits and floor theme), with rules as data.
  - Whether a 1–3 lift's shaft blocks cells on floor 2.
  - Room interiors generated by rules per room type (e.g. a table needs chairs). Suggested: place **groups** (table + chairs as one footprint), each with an anchor rule (wall/corner/centre), clearance and min/max counts, placed required-first, reusing the build system's placement-validity check. Comes after layout and type assignment.

### Horde

- Three tiers:
  1. **Abstract:** a group record moving through unloaded chunks at real travel speed. Enemies can spawn on the far side of the map but only become real once they reach a loaded chunk.
  2. **Blob:** one entity (centre, count, radius, pooled HP) following the flow field.
  3. **Individuals:** full entities near colonists/defences.
- Split/merge with hysteresis. **Tier distances are fixed and never tied to graphics settings.** Spawn into loaded chunks at their borders, spread over frames.
- Build individuals first; add blobs when profiling calls for it.
- **Guiding principle:** do the least work per enemy that the player can't notice. Distance decides the level of detail: full individual behaviour near colonists, condensed groups far away. Condensing is only a simulation shortcut, so the horde must still look huge on screen.
- **What tiers don't fix:** per-frame allocations and map-sized arrays stay the same whatever the enemy count, and a fight in an open bay can still put thousands of individuals near colonists, so the spatial hash is still needed.
- **Staggered updates (suggested, not decided):** update only a fraction of far-away enemies each frame to spread out the CPU cost. Distances would have to be fixed and independent of graphics settings, like the tier distances.
- **"Sea of enemies" look:** desynced animations (random start offsets); scale/tint/variant variation; soft separation; noise on flow direction; density-driven pile-ups at walls/doors (sprites drawn larger, nudged up, darkened underneath where density is high); far blobs rendered as a density-masked "crawling carpet" shader.
- **Breeders (idea, 2026-11-10, not decided):** the horde has physical sources, breeder structures that pop out enemies. They can already be inside the hulk or arrive with crashed ships. Implications: spawn points become entities on the map instead of map-edge spawning; destroying breeders could be a player goal; breeders in unrevealed chunks would need to spawn abstract-tier groups, since nothing there is fully simulated; a "breeder crash" fits as a director event.
- **Worm creature:** fills a whole corridor; a common event the enemy director can spawn. Only the head pathfinds (own cost field, can tunnel); segments follow its trail. It isn't part of the flow fields: it crushes or pushes whatever is in its tiles (including the horde). Needs clear, repeatable tells and buildable counters (bulkheads, reinforced walls, traps).

### Enemy director

- **Utility AI** (decided) plus a pacing state machine: build-up → peak → relax.
- Earns threat points over time. A colony snapshot every few seconds (defence strength, chokepoint concentration, recent losses, time since last event) drives each event's score. Weighted-random pick among the top affordable options.
- Events are data-driven (cost, cooldown, scoring curve) and run on a slow tick.
- Build a debug overlay (scores, threat points, state) and log decisions with their snapshot.

### Colonists

- **No colonist limit** (replaces the earlier "a handful of colonists" assumption), so colonists need to scale like enemies.
- Controlling many is meant to be a struggle at first; past a point the player relies on **automation**: crews/squads, work priorities, zones/standing orders, conditional rules. Overseer/leader colonists run crews. Crises (breaches, shakes) still force manual intervention.
- One job system: direct orders are just top-priority jobs; automation rules generate jobs.
- **Stories:** every colonist gets a light Dwarf Fortress-style history. The player **selects leaders**, who are the characters they get attached to (deeper detail, portraits, mechanical crew bonuses, scarce slots).
- Colonists **level up**; skills and traits unlock actions, not just bigger numbers.

### Possession ("hero moment")

- The player can briefly take direct control of **any** colonist (twin-stick style). Their abilities depend on their skills: a veteran leader may melee well; a random colonist may not be able to melee at all.
- **Subtle boost, not a power fantasy.** A small precision buff (aim, interaction speed, dodge), and survival depends on the squad around you. Getting surrounded is fatal. The value is in clutch actions (rescuing the wounded, sealing doors, repairing under fire, killing an elite). Possession feeds the colonist's story and builds the leader group.

### Player building (goal)

- The player picks a building from a **UI menu**, a **ghost preview** follows the mouse (tinted for valid/invalid), and it only places on a valid spot.
- Buildable pieces: **walls, doors, and crafting buildings**. Pieces can be **multi-cell**. Placing **costs resources**.
- One shared validity check used by both the ghost and the placing system, so they never disagree; the placing system has the final say (the ghost's tint is only a hint).
- The player **may wall off a lift**, so pathfinding must handle unreachable cells.
- Building types aren't decided yet — don't push building definitions until the developer brings it up.
- Still open: how a cell records that a crafting building occupies it (a new `TileType`, or a separate per-cell occupancy layer).

### UI & input

- Colony builder = UI-heavy.
- Plan for controller support: Halo Wars-style selection (select on-screen/all, paint-select, radial menus, cursor magnetism/snapping, virtual mouse). UI swaps by **active input device**, not platform.

### Art (decided)

- **Camera style changed to RimWorld style (decided 2026-11-10)** — chosen over isometric (Space Haven) and the old steep three-quarter view after comparing mockups of a horde corridor. The map is drawn top-down; characters and objects are drawn standing up. Why: the horde stays readable (almost no occlusion, crowd density and pile-ups easy to judge at chokepoints), less overdraw than isometric, simpler grid/picking/twin-stick aiming. Tradeoff: flatter look, so mood relies on lighting and shadows. Still open: character camera angle and number of facings (likely 3–4 instead of 8)
- **Tile size: 128×128 square (decided 2026-11-11, in code).** `TILE_SIZE` in `constants.rs` is `Vec2::new(128.0, 128.0)`. It's still a `Vec2` and many uses read `.x`, a leftover from the old 128×112 grid; it could become a plain `f32`
- **Walls are flat, RimWorld-style (decided 2026-11-11, from a RimWorld screenshot in `Refrences/`).** No visible south face: a wall is a flat fill with a thin outline on every side facing a non-wall, and no outline where it joins another wall. This replaced the earlier plan of a detailed south-face strip plus perimeter trim. Each wall picks one of **16 tiles** from its 4 cardinal neighbours: bitmask N=1, E=2, S=4, W=8, bit set = that neighbour is a wall, off-map counts as wall; `wall_<n>.png` is mask `n` (0 = pillar, 5 = vertical, 10 = horizontal, 15 = solid mass). With thick generated wall mass, edge walls are mostly T tiles (3 wall neighbours). Known gaps: a 4-neighbour mask can't see diagonal-only floor, so concave corners may lack an outline; a soft floor shadow next to walls would fall in the neighbouring cell, so it can't be baked in and would need its own overlay layer (not built). Current renders are first-pass: the outlines (~6px, `676C71`) are lighter than the fill (`5C5F62`) rather than near-black, and the wall file isn't using the `Box` pixel filter, so edge pixels are ~78–96% opaque; the developer is leaving this for a later detail pass
- **3D models in Blender, pre-rendered to sprites.** Orthographic camera with the same settings for every asset of a kind. Not pixel art or vector.
- **Top-down render rig (map tiles, 2026-11-11):** 1 tile = 1 Blender unit (1m), modelled true-scale at scale 1 (Ctrl+A). Camera orthographic, Location `(0, 0, 10)` (in ortho the height doesn't change size; it only has to be above everything and inside the clip range), Rotation `(0, 0, 0)`, Orthographic Scale `1`. Output `128 × 128` PNG RGBA, Film Transparent on, View Transform `Standard`, Film > Pixel Filter `Box` width `1` (without it, edge pixels come out partly transparent and show as seams). The old 61° rig (Rotation `28.955025°`, 128×112 output, Shift Y for tall sprites) is superseded; characters will get their own angled rig once the angle is decided
- **Texture resolution per asset type (2026-11-10):** texture resolution is separate from on-screen size (`TILE_SIZE` / `custom_size`), so asset types can differ. After a test, the developer preferred 256px over 128px, but **floors and walls stay at 128px**: they are everywhere and never the focus, and a higher-res option can be added later as a graphics setting. **Characters will probably use 256px** (not final). Enemies not yet decided. Render high-res masters in Blender so smaller sizes can be exported from them
- **Blender source files** live outside the repo at `F:\Blender\The_Last_Vestage\`, so they aren't under version control: `render_rig.blend` is the clean rig to copy for new assets, and `floor_tile.blend` is the plain floor , `floor_tile_oil.blend` the oil-spill variant (an extra gradient + noise mask in `floor_panel` that blends in near-black and lowers `Specular IOR Level`), and `floot_tile_drain.blend` the drain variant (a copy with a square hole, `drain_dark` bottom and an arrayed `grate_bar`). The rim and bolts use the plain `floor_steel` material, never the noise-driven `floor_panel`, so neighbouring tiles always match at their edges. Rendered sprites are what go into `assets/`. The floor files now use the top-down rig. Walls are in `Enviroment\Walls\`: `wall.blend` was saved over with the flat wall (the old 61° wall and its `wall_steel` material are gone). It has a full-tile top plane plus four thin outline planes, one per edge, named `North`/`East`/`South`/`West` (just above the top in Z, origin at the tile centre so rotated copies land on each edge), and a text block `blender_wall_gen.py` that loops masks 0–15, sets each outline's `hide_render` from its bit, and renders `assets/Walls/wall_<n>.png`. Run it from Blender's Text Editor (`bpy` only exists inside Blender, so not from PyCharm). `wall_filler.blend` still holds the `wall_cut` material
- Mood reference: grimdark gothic ship interior, makeshift colony (tarps, laundry lines, sandbags, fire barrels, catwalks), armoured knight-like leaders. Palette: dark steel, rust, muted military green; accents of warm fire orange and toxic green glow. Give knights an original identity (avoid looking like Warhammer IP).
- Pipeline: modular meshes → shared humanoid rig (Rigify) → short animation loops → render 8 directions with a transparent background → output **colour + normal map** (+ emission mask) passes → Blender Python batch-render script → pack with the Sprite Sheet CLI tool (colour and normal atlases with an identical layout; see [Assets](#assets) for how it's run) → in-game sprites lit using the normal maps.
- Colonists: 8 directions, with body and weapon split so weapons rotate freely for aiming. Horde creatures are low-profile (crawling/hunched) with fewer directions and frames.
- Effects: particles for dust; scripted debris (growing shadow → fall → impact → tile change); sprite-sheet collapses; shaders for haze and glows.
- The art currently in `assets/` (knight sheets, floor tilesheet, Grunt) is **placeholder pixel art** until the Blender pipeline produces final art.

## Code Organisation

Files have specific, focused responsibilities — keep things clean and tidy. One file should not do many unrelated jobs. Prefer one plugin per feature area.

**`constants.rs`** is a temporary home for values being tweaked; most are expected to move out once settled. Groups are wrapped in `// region` / `// endregion` comments so RustRover can fold them. When moving a value out:
- Keep anything used in several places as a shared named constant (`TILE_SIZE`, `MAP_WIDTH`, `MAP_HEIGHT`, `OFFSETS`).
- Make single-use values a private `const` at the top of the file that uses them, rather than a bare number in the code.
- Per-type or per-floor values (enemy stats, room parameters) become data once there's more than one type or floor style.

## Module Structure

```
src/
  main.rs          # App entry point, startup systems
  constants.rs     # Shared constants — TILE_SIZE, MAP_WIDTH, MAP_HEIGHT, OFFSETS, ENEMY_SPEED, ENEMY_STOP_RADIUS, ENEMY_SEPARATION_STRENGTH, ENEMY_HEALTH, ENEMY_RANGE, ENEMY_DAMAGE, ENEMY_COUNT, ENEMY_SPAWN_EDGE_BAND, COLONIST_HEALTH, COLONIST_SPEED, COLONIST_RANGE, COLONIST_DAMAGE, ROOM_ATTEMPTS, ROOM_MIN_SIZE, ROOM_MAX_SIZE, CORRIDOR_WIDTH, CORRIDOR_WIDTH_WEIGHTS, EXTRA_LINKS, CAMERA_WASD_SPEED, CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM, CAMERA_ZOOM_SPEED
  map/
    mod.rs         # Declares submodules, re-exports Map, TileData, TileType, MapRendererPlugin
    map.rs         # TileType, TileData, Map struct and constructor — no generation logic; Map::grid_to_world and Map::world_to_grid conversions; Map::tile_texture_index(x, y) — sheet index for any tile (walls: WALL_SHEET_START + 4-neighbour mask built from WALL_NEIGHBOURS via the private is_wall_at); cursor_to_grid(camera, camera_transform, cursor_pos, map) shared utility
    map_gen.rs     # Room struct (private) with overlaps, centre, distance_to; carve_room; link_rooms; extra_links; carve_corridor; generate_map — fills the map with wall, places random non-overlapping rooms, carves them, then links them with corridors
    map_renderer.rs # Spawns the bevy_ecs_tilemap entities from the Map resource — one layer for floors, walls and doors
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
    animation.rs   # ColonistAnimationState, EnemyAnimationState enums; AnimationPlayback component (image, layout, frame_index, frame_count, timer) — see Character Animation
  animation/
    mod.rs              # Declares submodules, re-exports their plugins/types
    knight_assets.rs    # KnightAnimation resource; load_knight_animation Startup system
    playback.rs         # advance_animation — shared, entity-type-agnostic frame-advance system
    colonist_animation.rs # ColonistAnimationPlugin; decide_colonist_animation system
  colonists/
    mod.rs              # Declares submodules, re-exports Colonist, CharacterPlugin, ColonistSpawnerPlugin, SelectionPlugin
    characters.rs       # Colonist marker component; CharacterPlugin; separate_colonists, move_character, move_to_click systems; tile_at helper
    colonist_spawner.rs # ColonistSpawnerPlugin; spawn_colonist Startup system — spawns colonist bundle (Colonist, GridPosition, Health, Speed, Sprite, Transform, Path, Attacker, Target, AnimationPlayback) at two hardcoded grid positions
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
    camera.rs      # CameraPlugin; CameraTruePosition component; setup (Startup — spawns Camera2d); zoom_camera (scroll wheel, multiplicative scale on OrthographicProjection); pan_camera (middle mouse drag, delta scaled by ortho.scale); camera_wasd_move (WASD, scaled by ortho.scale); snap_camera_to_true_position (rounds Transform to the screen pixel grid)
    sound.rs       # AmbientPlugin; startup system that loads and spawns the looping ambient audio entity
    y_sort.rs      # YSortPlugin; YSort marker; y_to_z(y) helper with Y_SORT_BASE / Y_SORT_SCALE; apply_y_sort system (PostUpdate) — see Rendering
  combat/
    mod.rs         # Declares submodule, re-exports CombatPlugin
    combat.rs      # CombatPlugin; enemy_attack system (nearest colonist auto-targeting); colonist_attack system (spot mode via Target or passive nearest-enemy-in-range)
  death/
    mod.rs         # Declares submodule, re-exports DeathPlugin
    death.rs       # DeathPlugin; tag_dead system (generic, ordered .after(colonist_attack).after(enemy_attack)); dead_enemies_handler, dead_colonists_handler (each .after(tag_dead)) — split out of combat/ so unrelated plugins (loot, animation, morale) can order against tag_dead without depending on combat internals
  debugs/
    mod.rs         # Declares debugs submodule, re-exports DebugPlugin
    debugs.rs      # DebugPlugin; DebugSettings resource (grid_lines, ui_elements: bool); DebugUi marker; controls system (F1 toggles grid_lines, F2 toggles ui_elements); grid_lines system (gizmo grid over the map, early-returns when off); ui_elements system (shows/hides everything tagged DebugUi); enemy counter and FPS counter text
  ui/
    mod.rs         # Declares hud and life_support submodules, re-exports hud
    hud.rs         # UiPlugin; OxygenText, FoodText, WaterText, EnergyText markers; spawn_hud_root Startup system — spawns the full-screen root Node and the life-support bar; update_life_support_hud system (runs only when LifeSupport changes)
    life_support.rs # LifeSupportPlugin; LifeSupport resource
```

### Assets

All current art is placeholder (see [Art](#art-decided)).

- `assets/Floors/tilesheet_6x4_128px.png` (+ `.json`) — the sheet the game loads (`map_renderer.rs`); 768×512, 6 columns × 4 rows of 128×128 cells, row-major. Indices `0`–`5` = `floor_tile_01`–`06`, indices `6`–`21` = `wall_0`–`wall_15` (so wall mask `n` is index `6 + n`, `WALL_SHEET_START` in `map.rs`), last 2 slots empty. No door art yet
- **Packing:** built with the developer's own packer (github.com/Ghosthi1/SpriteSheetPacker_CLI, installed with `cargo install --git`). The exe is `Sprite_Sheet_Packer`, not `sprite-packer` as its README says, since `Cargo.toml` names the package that and has no `[[bin]]`. It keeps the given file order (stable tallest-first sort), so pass files explicitly in index order rather than a folder, which would read `wall_10` before `wall_2`. From the repo root in PowerShell: `$files = (1..6 | % { "assets/Floors/floor_tile_0$_.png" }) + (0..15 | % { "assets/Walls/wall_$_.png" }); Sprite_Sheet_Packer -o assets/Floors/tilesheet_6x4_128px -w 768 $files`
- `assets/Floors/floor_tile_01.png` … `floor_tile_06.png` — 128×128 top-down renders, one per value of the shared `Value` node that drives the 4D noise/Voronoi `W` inputs (01–03 and 06 from `floor_tile.blend`; 04 from `floor_tile_oil.blend`; 05 from the drain file); source images for the sheet, not loaded by the game. **Known issue:** the drain was saved as `floor_tile_01` (slot 0, weight 50) and slot 4 has no drain, so about half of all floor shows a drain until 01 (plain, value 1) and 05 (drain, value 5) are re-rendered and the sheet re-packed
- `assets/Walls/wall_0.png` … `wall_15.png` — the 16 flat wall tiles, 128×128, one per neighbour mask (see [Art](#art-decided)); rendered by `blender_wall_gen.py`
- `assets/Walls/wall_sheet.png` (+ `.json`) — walls-only 4×4 pack (512×512); not loaded by the game
- `assets/Walls/wall_edge_01.png`, `wall_strip.png`, `wall_top_01.png`, `wall_filler.png` — leftovers from the face-strip experiments and the old 128×112 setup; **unused**, can be deleted
- `assets/Floors/tilesheet_6x3_128x112px.png` — the previous 128×112 sheet, no longer loaded
- `Refrences/` (repo root) — art references, e.g. the RimWorld screenshot the flat-wall decision was based on
- `assets/Floors/tilesheet_6x3_128px.png` — old 128×128 placeholder sheet, no longer loaded; its row-2 door cells (128×128) can be cropped out as placeholder doors for the new sheet. Spritesheet, 6 columns × 3 rows of 128×128 tiles: row 0 = 6 floor visual variants, row 1 = wall, row 2 = door (closed, open, locked columns — locked reserved, not wired up yet). Replaces the old `PlaceHolder_tileset.png` placeholder (no longer referenced anywhere in `src/`). This sheet predates the 128×112 grid (see [Art](#art-decided)). `TILE_SIZE` is defined in `src/constants.rs` as a shared `pub const` `Vec2`, imported via `use crate::constants::TILE_SIZE` wherever tile sizing is needed
- `assets/enemeys/Spiders/Grunt.png` — sprite for the Grunt enemy; loaded via `AssetServer` in `spawn_enemy` and set on the `Sprite` `image` field; `custom_size` is `Vec2::splat(TILE_SIZE.x)` but the grunt is intentionally drawn smaller than the canvas for visual style — hitbox size will be defined independently when collision is added
- `assets/Sound/Background/ambient_spaceship.ogg` — looping ambient soundtrack; loaded and spawned as an audio entity in `systems/sound.rs` via `AmbientPlugin`
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
- **`change_health(delta: f32)`** — adds `delta` to `current` (negative for damage, positive for healing); clamps `current` to `[0.0, max]`; when `current` hits `0.0` the entity is considered dead — `Health` itself never reacts to this, detection is handled entirely by an external system (`tag_dead`, see below)
- **`is_dead() -> bool`** — returns `self.current <= 0.0`; called by an external system that queries for dead entities and handles despawning, tile events, and animations — `Health` itself cannot interact with the world
- **Death handling pattern** — `Health` is pure data; death detection is a single generic system (`tag_dead`) that queries any entity with `Health`, calls `is_dead()`, and inserts a `Dead` marker component, regardless of entity type; reaction to the marker is split by type — `dead_enemies_handler` and `dead_colonists_handler` each react to `Dead` scoped to their own entity type and currently just despawn, with a `TODO` on each for future type-specific behaviour (death animation, loot drop, morale/game-state effects); implemented for both enemies and colonists — no movement/AI queries filter on `Without<Dead>` yet since nothing currently needs to
- **`Dead` marker component** — zero-sized component in `components/combat.rs` alongside `Health` (stays here for now — not moved to `death/` since it's plain data with no plugin of its own; revisit if a non-combat death cause, e.g. hunger, is ever added); inserted by the generic `tag_dead` system (no entity-type filter); consumed separately by `dead_enemies_handler` (`With<Dead>, With<Enemy>`) and `dead_colonists_handler` (`With<Dead>, With<Colonist>`) — tagging is generic, reaction is split by type since colonist and enemy death are expected to diverge in behaviour
- **`Attacker` component** — lives in `components/combat.rs`; private `damage: f32`, `range: f32`, `cooldown: Timer` fields; constructed via `Attacker::new(damage, range, timer)`; range is in world units (multiply tile counts by `TILE_SIZE`); cooldown is a `Timer::from_seconds(n, TimerMode::Repeating)` — ticked every frame regardless of range so the first hit after closing distance doesn't fire instantly; both colonists and enemies carry this component
- **`Target` component** — lives in `components/combat.rs`; tuple struct `Target(pub Option<Entity>)`; colonist-only; `None` = passive mode (attack nearest enemy in range), `Some(entity)` = spot mode (wait for that specific enemy to enter range); stale entity handling is implemented — in `colonist_attack`, the `Some(entity)` arm falls back to `target.0 = None` when the target is not found in that frame's `enemy_snapshot` (e.g. the target despawned), so a colonist whose spot-target dies drops back to passive/nearest-enemy mode instead of going idle
- **`tag_dead` system** — lives in `death/death.rs` under `DeathPlugin` (moved out of `combat/`); queries `(Entity, &Health)` filtered `Without<Dead>, Changed<Health>` — no entity-type filter, so it tags any dead entity regardless of type; `Changed<Health>` means it only runs against entities that took damage (or were healed) this frame rather than scanning the whole population; inserts `Dead` when `health.is_dead()`; registered with both `.after(colonist_attack)` and `.after(enemy_attack)` chained — it must run after both, since either can zero out a `Health` it doesn't itself own (colonists damage enemies, enemies damage colonists); registering it twice with one `.after()` each was tried and rejected — Bevy doesn't deduplicate a function added via separate `add_systems` calls, so that produced two system instances running redundantly every frame; `colonist_attack`/`enemy_attack` are imported from `crate::combat` purely for the ordering call — `DeathPlugin` doesn't otherwise depend on combat internals, and this cross-plugin `.after()` is the pattern any future plugin (loot, animation, morale) should follow to order against death without depending on combat
- **`dead_enemies_handler` / `dead_colonists_handler` systems** — live in `death/death.rs` alongside `tag_dead`, not split into `enemys/`/`colonists/` yet since both are still despawn-only and identical in shape — move each into its owning feature module once it grows real type-specific behaviour (loot table on enemy death, say); each queries `Entity` filtered `With<Dead>` plus their respective `With<Enemy>`/`With<Colonist>`; both currently just despawn; both registered `.after(tag_dead)`; kept as separate systems from tagging (rather than combined) so future type-specific reactions (death animation, loot drop for enemies; morale/game-state effects for colonists) can be filled in independently without restructuring `tag_dead` — each has a `TODO` marking that it's despawn-only for now
- **Why split from `combat/`** — loot, animation, morale, and job-cancellation reactions to death all need query shapes or cross-frame state unrelated to combat (e.g. morale needs to iterate *other* colonists, not the dead entity); giving `tag_dead` its own plugin lets those future plugins order `.after(tag_dead)` without depending on `CombatPlugin` at all

### Characters

- **Components:** `GridPosition((u32, u32))` — authoritative grid position (inner tuple), lives in `components/movement.rs`; `Path(VecDeque<(u32,u32)>)` — remaining waypoints, lives in `components/movement.rs`; `Speed(f32)` — movement speed in tiles per second, lives in `components/movement.rs`; `Health` — current/max health, lives in `components/combat.rs`; `Colonist` — zero-sized marker in `colonists/characters.rs`, filters colonist-only queries so enemies are never accidentally included
- **Colonist bundle:** `Colonist`, `GridPosition`, `Health::new(COLONIST_HEALTH)`, `Speed(COLONIST_SPEED)`, `Sprite`, `Transform`, `Path`, `Attacker::new(COLONIST_DAMAGE, COLONIST_RANGE, Timer)`, `Target(None)`, `AnimationPlayback` — constants in `constants.rs`
- **Smooth movement:** `move_character` advances `Transform` toward the next waypoint each frame using `move_towards(target, speed * delta_secs)`; `GridPosition` is only updated when the character arrives at a waypoint (`distance_squared < 0.01`, avoiding a sqrt); `transform.translation` is only snapped to the tile center on normal arrival — not in the conflict branch, so the transform never visually lands on an occupied tile
- **Click-to-move:** `move_to_click` calls `cursor_to_grid` (shared utility in `map/map.rs`) to convert cursor window position → grid coordinates; start position is `path.0.front()` if a path is already in progress, otherwise `grid_pos.0` — keeps movement smooth mid-path by continuing from the current waypoint rather than snapping back to grid position
- **Click-time goal assignment:** before the assignment loop, `move_to_click` snapshots all colonist `GridPosition`s into a `mut HashSet<(u32, u32)>`; for each colonist, if the clicked goal is already in the set, searches 8 neighbours of the goal for a free passable tile; uses `actual_goal` (the neighbour, or the original goal if free) for `find_path`; inserts `actual_goal` into the set so subsequent colonists in the same click get distinct targets — prevents multiple colonists converging on the same world position
- **Arrival-time conflict detection:** `move_character` snapshots all `GridPosition`s into a `mut HashSet` before the movement loop; at arrival, checks `occupied.contains(next) && *next != grid_pos.0` — the guard excludes the start-waypoint case where `next` equals the colonist's current tile (which appears in `occupied` as their own position); if occupied, searches 8 neighbours of `next` for a free passable tile and replaces `path.0[0]` with it without updating `grid_pos` or snapping transform; on normal arrival, inserts the arrived tile into `occupied` so same-frame arrivals at the same tile are caught by subsequent iterations
- **Separation steering:** `separate_colonists` runs `.before(move_character)` each frame; same upper-triangle force pattern as `separate_enemies` — snapshots world positions, computes repulsion for each pair, applies via axis-separated wall collision using `tile_at`; uses `ENEMY_SEPARATION_STRENGTH` constant; handles the visual overlap that arrival-time detection cannot (colonists approaching the same tile before arrival triggers)
- **`tile_at` helper:** identical copies live in `characters.rs` and `enemy.rs` — calls `Map::world_to_grid`, then returns `None` if the tile is impassable; used for wall collision. Could become one method on `Map`
- **System ordering:** `separate_colonists.before(move_character)` and `move_to_click.before(move_character)` — explicit `.before()` constraints, not `.chain()`
- **Tilemap offset:** the tilemap is centered on screen. `Map::grid_to_world(x, y)` returns a tile's centre in world space and `Map::world_to_grid(world)` returns the tile containing a point (`None` if off the map); all grid/world conversion goes through these two methods rather than being written inline

### Character Animation

- **Status: in progress** — `components/animation.rs` is built (state enums + shared playback component, see below); the `KnightAnimation` resource is built and populated at startup, and `spawn_colonist` renders colonists from it. `AnimationPlayback` fields are `pub` (same rationale as `KnightAnimation` — binary crate, no external consumers) and an instance is attached to each colonist at spawn time, seeded to the idle state. The shared frame-advance system is built and running, and colonist state-switching (Idle/Walk/Attack) is now wired up — see `animation/colonist_animation.rs` below. Still missing: Carry/Work/Sleep states and skin-colour selection. This section records the agreed design; items are moved from "Planned pieces" to their own bullet once built. Note: the knight sheets are placeholder art — final sprites come from the Blender pipeline (8 directions, colour + normal passes), which will change the sheet layouts this section relies on
- **`animation/playback.rs` (built)** — `advance_animation`, the shared frame-advance system described in the design above; entity-type-agnostic, never matches on `ColonistAnimationState`/`EnemyAnimationState`, so the same system will serve enemies once they get playback components too. Query is `Query<(&mut AnimationPlayback, &mut Sprite)>`; each tick calls `playback.timer.tick(time.delta())`. Unconditionally, every frame (not gated on `just_finished()`), syncs `sprite.image` from `playback.image` whenever they differ (`if sprite.image != playback.image { sprite.image = playback.image.clone(); }`) — this has to run immediately on a state change rather than waiting for the current frame's timer to elapse, otherwise a colonist would keep showing its old animation for up to one frame-duration after `decide_colonist_animation` already switched state. On `just_finished()` (the timer is `TimerMode::Repeating` so it resets itself) advances `frame_index = (frame_index + 1) % frame_count` and writes that into `sprite.texture_atlas`'s `index` via `if let Some(atlas) = &mut sprite.texture_atlas`. Registered in `AnimationPlugin::build` via `app.add_systems(Update, advance_animation)` — `Update`, not `Startup`, since it must run every frame, unlike the one-shot `load_knight_animation`. Declared/re-exported from `animation/mod.rs` the same way as `knight_assets`. Still doesn't sync the atlas's `layout` — only `image` and `index` — since every state built so far (Idle/Walk/Attack) shares `layout_4x2`; revisit once `Sleep` (which uses `layout_2x1`) is wired in
- **`animation/colonist_animation.rs` (built)** — `decide_colonist_animation`, the first "decide what should play" system, colonist-specific; arbitrates `Attack` > `Walk` > `Idle` each frame and writes the result into `AnimationPlayback`. Registered via `ColonistAnimationPlugin` (its own plugin struct, added to `main.rs` as a **separate** `.add_plugins(ColonistAnimationPlugin)` call rather than folded into the main plugin tuple — that tuple was already at Bevy's max supported arity, so a 16th entry wouldn't compile; multiple `.add_plugins()` calls are equivalent, so this is a mechanical workaround, not a design choice) — `app.add_systems(Update, decide_colonist_animation.before(advance_animation))`; the explicit `.before()` matters because a state change and its frame/timer reset must land in the same frame, before `advance_animation` ticks the (possibly stale) old timer
- **Combat and animation deliberately stay decoupled** — `decide_colonist_animation` does its own lightweight enemy-in-range check (snapshot enemy `Transform`s, then `.any()` distance-squared test against `Attacker.range`) rather than reading a flag set by `colonist_attack` in `combat/combat.rs`. This duplicates part of `colonist_attack`'s range logic, but keeps `combat.rs` with zero knowledge of animation, matching how other systems in this codebase (e.g. `move_enemy`) already take their own position snapshots rather than share state across unrelated systems. Query shape differs from `colonist_attack` too: `.any()` only needs "is *any* enemy in range," not "which is the *nearest* enemy," so no closest-match tracking is needed here
- **Per-state frame duration** — `decide_colonist_animation` also computes a `desired_duration` alongside `desired_image` (Attack: `0.3`s/frame, Walk/Idle: `0.2`s/frame — Attack tuned slower than the initial `0.2` since the swing read as too fast at the shared idle speed) and calls `animation.timer.set_duration(Duration::from_secs_f32(desired_duration))` inside the same state-change guard as the image/frame_index reset. `.set_duration()` is intentionally distinct from `.reset()` — `.reset()` only zeroes elapsed progress, `.set_duration()` changes how long each subsequent frame lasts; both are needed on a state change, and only inside the `if animation.image != desired_image` guard so the timer isn't rebuilt every frame a colonist stays in the same state
- **Known gap:** attack animation loops continuously and independently while a colonist is in range — it is not synced to `Attacker.cooldown` (colonists' cooldown is `1.0`s, the attack animation cycle is `4 * 0.3 = 1.2`s), so the visual swing and the actual damage tick can drift out of phase. Not yet addressed
- **`animation/knight_assets.rs` (built)** — `KnightAnimation` resource with `pub` fields for the six `Handle<Image>`s (`idle`, `walk`, `attack`, `carry`, `work`, `sleep`) and the two `Handle<TextureAtlasLayout>`s (`layout_4x2`, `layout_2x1`); fields are fully `pub` rather than `pub(crate)` or getter-gated — acceptable since this is a binary crate with no external consumers, so the usual privacy motivation doesn't apply. Populated by `load_knight_animation`, a `Startup` system taking `Commands`, `Res<AssetServer>`, and `ResMut<Assets<TextureAtlasLayout>>` — loads all six sheets via `asset_server.load`, builds both layouts via `TextureAtlasLayout::from_grid` (`UVec2::new(128, 128)`, `4×2` and `2×1` respectively) registered into `Assets<TextureAtlasLayout>` via `.add()`, then `commands.insert_resource(knight_animation)`. Registered via `AnimationPlugin` (`app.add_systems(Startup, load_knight_animation)`), added to the plugin tuple in `main.rs`
- **`spawn_colonist` updated** — takes `Res<KnightAnimation>` instead of `Res<AssetServer>`; no longer loads `Knight_1.png`. `Sprite.image` is set to `knight_animation.idle.clone()`; `Sprite.texture_atlas` is set to `Some(TextureAtlas { layout: knight_animation.layout_4x2.clone(), index: 0 })` — `index: 0` is hardcoded (silver skin, frame 0 of idle) since the skin-colour-picking component described below doesn't exist yet. `ColonistSpawnerPlugin` orders `spawn_colonist.after(load_knight_animation)` — a cross-plugin `.after()`, same pattern as `tag_dead` in [Combat](#combat), since `spawn_colonist` reads `Res<KnightAnimation>` which must be inserted first. Bundle also now includes an `AnimationPlayback`, duplicating the same `idle`/`layout_4x2`/`index: 0` values already given to `Sprite`, plus `frame_count: 4` and `timer: Timer::from_seconds(0.2, TimerMode::Repeating)` — the 0.2s-per-frame duration is a tunable feel choice, not derived from anything else
- **`components/animation.rs` (built)** — defines:
  - `ColonistAnimationState` enum — `Idle`, `Walk`, `Attack`, `Carry`, `Work`, `Sleep`; `#[derive(Clone, Copy, PartialEq, Eq, Debug)]`
  - `EnemyAnimationState` enum — `Idle`, `Walk`, `Attack`, `Death`; same derives; unused (`dead_code`) until enemy animation wiring exists, which is expected
  - `AnimationPlayback` component — `image: Handle<Image>`, `layout: Handle<TextureAtlasLayout>`, `frame_index: usize`, `frame_count: usize`, `timer: Timer`; `#[derive(Component)]`; entity-type-agnostic — doesn't know if it's attached to a colonist or enemy, only the per-entity-type "decide" systems know that. `decide_colonist_animation` (`animation/colonist_animation.rs`) is built; the enemy equivalent is not — enemies don't carry an `AnimationPlayback` at all yet, so `EnemyAnimationState` remains unused
  - re-exported through `components/mod.rs` (`pub mod animation;` / `pub use animation::*;`), same pattern as `combat`/`movement`/`selected`
- **Mechanism** — Bevy's `Sprite` has a `texture_atlas: Option<TextureAtlas>` field, where `TextureAtlas { layout: Handle<TextureAtlasLayout>, index: usize }`. Changing which sub-frame is shown = changing `index`. Changing which *animation* is shown (idle → walk) = swapping `Sprite.image` itself, since each animation state lives in its own sheet file (see [Assets](#assets))
- **Two shared layouts for the knight, not six** — `attack`/`carry`/`idle`/`walk`/`work` all share identical grid geometry (4 cols × 2 rows × 128px), so one `TextureAtlasLayout` built via `TextureAtlasLayout::from_grid` covers all five; `sleep`'s 2×1 grid needs a second, separate layout. A layout is index math only — it doesn't care which image it's paired with
- **Row/column = skin colour, not direction** — on the 4×2 sheets, silver is row 0 (atlas indices 0–3), bronze is row 1 (indices 4–7); on `sleep`, silver = index 0, bronze = index 1 (single static frame each)
- **Module placement: shared `animation/` module, not nested in `colonists/`** — mirrors why `death/` split out of `combat/`: once more than one entity type needs to react independently (there, loot/morale/animation reacting to `Dead`; here, colonists and enemies both needing frame playback), splitting out avoids duplicating near-identical systems
- **Split the "what should play" decision from "how playback advances"** — colonist states (idle/walk/attack/carry/work/sleep) and enemy states (expected: idle/walk/attack/death) don't overlap enough to share one enum; forcing them into one would leave each entity type carrying variants it never uses (e.g. an enemy `Carry` state) and dead match arms in any system that reads it. Two separate enums avoid that:
  - `ColonistAnimationState` and `EnemyAnimationState` — each entity type's own enum, read only by that entity type's own small "decide what should be playing" system (colonist job/state logic vs. enemy AI/combat state)
  - a shared, enum-agnostic playback component — current `Handle<Image>`, current `Handle<TextureAtlasLayout>`, frame index, timer, frame count — that both "decide" systems write into
  - one shared frame-advance system, living in `animation/`, that only ever reads/writes the playback component; it never matches on either enum, so it's genuinely one system for both entity types rather than two parallel ones
- **Remaining gaps (not yet built):**
  - **Colonist states `Carry`, `Work`, `Sleep`** — `decide_colonist_animation` only arbitrates `Attack`/`Walk`/`Idle`; nothing in the game yet has a signal for "this colonist is carrying/working/sleeping" to drive the other three `ColonistAnimationState` variants, since no job/task system exists yet to produce that signal. `Sleep` additionally needs the `layout` field synced (see `advance_animation` note above), not just `image`, since it uses `layout_2x1` instead of `layout_4x2`
  - **Skin-colour selection** — no component identifies silver vs. bronze on a colonist; every colonist hardcodes row 0 (silver) via `index: 0` at spawn and `decide_colonist_animation` never touches the row offset when switching state. Until this exists, every colonist is silver, always
  - **Enemy animation, entirely** — `EnemyAnimationState` is defined but unused; enemies have no `AnimationPlayback` component, no sprite sheet loading (still a single static `Grunt.png`, see [Assets](#assets)), and no "decide" system. This is a from-scratch build, not an extension of the colonist wiring, though `advance_animation` (the frame-advance system) is already entity-type-agnostic and will work for enemies unmodified once they have the component
  - **Attack/cooldown drift** — see the "Known gap" bullet above; the attack animation loop isn't synced to `Attacker.cooldown`, so the visual swing and the actual damage tick can drift out of phase

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
- **Horde spawn mode (temporary, for capturing footage)** — `spawn_enemy` currently spawns `ENEMY_COUNT` enemies (`constants.rs`) at random passable tiles within `ENEMY_SPAWN_EDGE_BAND` tiles of the map's four borders, replacing the three hardcoded spawn positions this was built with. Marked with a doc comment as temporary — intended to be replaced with real wave/spawn-point logic (see [Horde](#horde) and [Enemy director](#enemy-director)) once that is built, not a permanent feature
- **Known hang risk in `spawn_enemy`** — its `while spawned < ENEMY_COUNT` loop retries until it finds enough passable edge-band tiles and has no attempt limit. On the wall-filled map, the edge band can contain no floor at all, so any `ENEMY_COUNT` above `0` can freeze startup forever. `ENEMY_COUNT` is `0` for now while map generation is being built
- **Known performance ceiling** — `separate_enemies`' pairwise separation loop is O(N²); measured on dev hardware, `ENEMY_COUNT = 5000` holds ~80fps, while `8000` drops well below 30fps and reads as visibly stuttery. `5000` was chosen as the value used for README/portfolio footage specifically because it's the largest count that stayed smooth, not because it's a target for real gameplay. **The actual fix** — replacing the O(N²) pairwise scan with a spatial hash or grid-based neighbour lookup (already listed under [Planned Features / TODO](#planned-features--todo) as "Spatial queries") — has not been done; this measurement is the concrete evidence for why that refactor matters, not a substitute for it

### Buildings

- **`BuildingPlugin`** lives in `buildings/buildings.rs` — registers `TileChangedEvent` and three systems: `place_wall_on_click`, `on_tile_change`, `on_tile_passability_change`
- **`TileChangedEvent { x, y }`** — fired whenever a tile's type changes; all downstream reactions (visuals, pathfinding) are driven by listeners on this event rather than being inlined at the change site
- **`place_wall_on_click`** — handles left-click input only; converts cursor to grid via `cursor_to_grid`, mutates `Map`, fires `TileChangedEvent`; no pathfinding or rendering logic
- **`on_tile_change`** — visual listener; updates `TileTextureIndex` for the changed tile **and its 4 cardinal neighbours** (their wall masks change too), looping over `WALL_NEIGHBOURS` with `u32::checked_add_signed` plus a `>= map.width/height` check to skip off-map cells; indices come from `Map::tile_texture_index`; `TileStorage::single()` is resolved once before the event loop. Works but suboptimal: it writes through `commands.entity(..).insert(TileTextureIndex(..))` (5 queued commands per change) where querying `&mut TileTextureIndex` and writing directly would be cheaper — matters for mass changes like ship shakes
- **`on_tile_passability_change`** — pathfinding listener; rebuilds the colonist flow field once per frame (regardless of how many tile-change events fired), then clears any colonist `Path` that contains the changed tile coordinate; uses `Local<Vec<(u32,u32)>>` as a reusable positions buffer
- **`cursor_to_grid`** — shared utility in `map/map.rs`; takes `&Camera`, `&GlobalTransform`, `Vec2` cursor pos, `&Map`; returns `Option<(u32, u32)>`; projects the cursor to world space then calls `Map::world_to_grid`; used by both `place_wall_on_click` and `move_to_click` in `characters.rs`
- See [Player building (goal)](#player-building-goal) for where this is heading (menu, ghost preview, multi-cell, costs)

### Rendering

- **Texture sampler** — `DefaultPlugins.set(ImagePlugin::default_nearest())` in `main.rs` sets nearest-neighbor sampling globally; this keeps the current placeholder pixel art crisp at all zoom levels — Bevy's default bilinear sampler blurs upscaled sprites. Final art is pre-rendered Blender sprites (see [Art](#art-decided)), not pixel art, so revisit this global setting when that art comes in
- **Tilemap transform offset** — `bevy_ecs_tilemap` centers each tile's sprite at its grid position in local space, so tile `(0,0)` is centered at the tilemap entity's `Transform` origin. The tilemap transform is therefore `map.grid_to_world(0, 0)` at z `0.0`, which keeps tiles aligned with every other coordinate system
- **Walls: one tilemap layer, linked tiles (2026-11-11)** — walls are ordinary tiles in the same tilemap as floors and doors (one layer, no overdraw, no extra entities); there are no wall sprites any more (`spawn_edge_walls`, `EdgeWall` and `Map::is_edge_wall` were removed). Each wall's index comes from `Map::tile_texture_index(x, y)`: it checks the 4 cardinal neighbours listed in `WALL_NEIGHBOURS` (`pub(crate)` `[(i32, i32, u32); 4]` of `(dx, dy, bit)`; grid `y + 1` is north) with the private `is_wall_at(i32, i32)` (signed so `x - 1` can't underflow; off-map returns `true`), ORs the bits into a mask and returns `WALL_SHEET_START` (`6`) + mask. Non-walls fall through to `TileData::texture_index()`. Index logic moved onto `Map` because a single `TileData` can't see its neighbours
- **Y-sorting** — lives in `systems/y_sort.rs` (`YSortPlugin`). `y_to_z(y)` returns `Y_SORT_BASE - y * Y_SORT_SCALE` (`100.0`, `0.0001`): further south draws in front, and z stays ~71–129 even at 5120 tiles, above the tilemap at `0`. Pass the y of the object's **base** (where it touches the floor), not its centre; for units the `Transform` already is the ground point. Static sprites would call `y_to_z` once at spawn (walls used to; they're now tiles). Moving things carry the `YSort` marker, and `apply_y_sort` (`PostUpdate`, `.before(TransformSystems::Propagate)`, filtered `With<YSort>, Changed<Transform>` so idle units are skipped) rewrites their z after all movement. **Movement must never write z:** `move_character` extends its target with the colonist's current z, otherwise `move_towards` fights the sort and the arrival check never passes. Colonists and enemies are tagged; the spawn z values are irrelevant since new entities count as `Changed`
- **Grid size vs tile size** — `TilemapGridSize` is the spacing between tiles and uses `TILE_SIZE` (128×128). `TilemapTileSize` is the image slice cut from the sheet. Both use `TILE_SIZE`, so the loaded sheet must have 128×128 cells
- **Debug tooling lives in `DebugPlugin`** (`debugs/debugs.rs`) — all debug-only features go in this one plugin so removing `.add_plugins(DebugPlugin)` from `main.rs` disables them all. `DebugSettings` resource holds per-feature toggle flags (`grid_lines`, `ui_elements`), inserted by the plugin itself so it disappears with the plugin. `controls` reads keyboard input (`Res<ButtonInput<KeyCode>>`) and writes flags (`ResMut<DebugSettings>`); each debug system reads its flag via `Res<DebugSettings>` and early-returns when off. The gizmo grid overlay (formerly the commented-out `debug` fn in `main.rs`) moved here as `grid_lines`, toggled by F1. The FPS and enemy counters moved here from `ui/`; both are tagged with the `DebugUi` marker, spawn `Visibility::Hidden`, and are shown or hidden together by `ui_elements`, toggled by F2. Gizmo lines are redrawn every frame (~`MAP_WIDTH + MAP_HEIGHT` lines) — fine for debug at the current map size, but not at the target ~5120 map; the `LineList` mesh noted under [Tile System](#tile-system) is the long-term approach

### Camera

- **Spawn** — `setup` is a `Startup` system that spawns `Camera2d`; must be registered in `CameraPlugin::build` or nothing renders
- **Zoom** — `zoom_camera` reads `Res<AccumulatedMouseScroll>` and applies a multiplicative scale change to `OrthographicProjection.scale` each frame: `scale = (scale * (1.0 - delta.y * CAMERA_ZOOM_SPEED)).clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM)`, currently `0.1` speed and a `0.3`–`3.0` clamp; multiplicative scaling feels consistent at all zoom levels; lower clamp bound is a UX decision — nearest-neighbor keeps pixels crisp but extreme zoom-in shows very large blocky pixels, so the clamp prevents unintentionally rough-looking close-up views
- **Pan** — `pan_camera` checks `ButtonInput<MouseButton>::pressed(Middle)` and reads `Res<AccumulatedMouseMotion>`; moves `CameraTruePosition` by the mouse delta multiplied by `ortho.scale` so panning speed stays consistent regardless of zoom level; x is negated (drag right = pan left), y is added as-is (screen and world y felt correct without negation)
- **WASD** — `camera_wasd_move` moves `CameraTruePosition` at `CAMERA_WASD_SPEED`, scaled by delta time and `ortho.scale`, so speed stays consistent at every zoom level
- **Pixel snapping** — `CameraTruePosition(Vec2)` holds the unrounded camera position, and pan and WASD write only to it. `snap_camera_to_true_position` runs `.after()` `zoom_camera`, `pan_camera` and `camera_wasd_move`, then rounds it to a multiple of `ortho.scale` and writes that to `Transform`. Keeping the unrounded value separate means small movements aren't lost to rounding
- **Projection access** — `OrthographicProjection` is not a standalone component in Bevy 0.18; access it via `Query<(&mut Transform, &Projection)>` and match `Projection::Orthographic(ref mut ortho)` to read or write `ortho.scale`
- **Input resources** — Bevy 0.18 provides `AccumulatedMouseScroll` and `AccumulatedMouseMotion` as frame-accumulated resources; prefer these over `EventReader<MouseWheel>`/`EventReader<MouseMotion>` for per-frame input reading

### Audio

- **Entity-based audio** — Bevy 0.15+ replaced the `Audio` resource with a component model; playing audio means spawning an entity with `AudioPlayer` and `PlaybackSettings` components; despawning the entity stops playback
- **Ambient music** — loaded and spawned once in a `Startup` system in `systems/sound.rs`; `PlaybackSettings::LOOP` keeps it running for the lifetime of the app
- **Asset paths** — `AssetServer::load` paths are relative to the `assets/` folder and must never include `assets/` as a prefix — Bevy prepends it automatically; capitalisation must match the filesystem exactly

### Tile System

- **Hybrid approach:** map data lives in a `Resource` (flat array, indexed `x + y * width`), visuals are entities/tilemap, dynamic actors (colonists, enemies, buildings) are entities with grid position components
- **Tiles are destructible and buildable** — walls can be broken by players and enemies, floors can be built on
- **Tile changes:** mutate the map resource → fire `TileChangedEvent { x, y }` → two listeners react: `on_tile_change` updates the tilemap visual (texture index), `on_tile_passability_change` rebuilds the colonist flow field and clears any colonist `Path` that passes through the changed tile; adding new tile-change sources (enemy wall breaks, deconstruction, ship shakes) only requires firing the event — listeners handle all downstream effects automatically
- **Grid coordinates are the source of truth for colonists** — colonist `Transform` is derived from `GridPosition`; for enemies the relationship is reversed: `Transform` is authoritative and `GridPosition` is derived from it each frame to support continuous swarming movement
- **Parallel arrays for rare data:** primary array holds only hot data (tile type, passability); oxygen, temperature, etc. live in separate resources indexed the same way for cache efficiency; truly sparse properties (affects <~5% of tiles) use `HashMap<(u32, u32), T>` instead
- **Keep `TileData` lean** — start small, add parallel resources only when actually needed
- **Map is very large but bounded** (~5000×5000, 3 floors; see [World & map](#world--map)), generated in chunks revealed as the player works outward; fog of war hides unexplored chunks. The current code is a single fixed-size map (`MAP_WIDTH` × `MAP_HEIGHT`, currently 100×100). When the chunk system is built, fire a chunk-reveal event so dependent systems (grid overlay, pathfinding) can react
- **Grid overlay deferred** — a `PrimitiveTopology::LineList` mesh is the right approach; build it once the chunk system exists so the mesh update hook has something to connect to

### Floor Variants

- **Cosmetic variation, not tile type** — `TileData` (`map/map.rs`) has a `floor_variant: u32` field alongside `tile_type`; it selects which of 6 floor art variants a `Floor` tile renders as, entirely independent of `TileType` — walls and doors have no variants
- **`texture_index()` lives on `TileData`, not `TileType`** — moved off `TileType` since computing the final sheet index needs both `tile_type` and `floor_variant` together; `TileType` alone can no longer answer "what's my texture." `TileType::is_passable()` stays on `TileType` since passability only depends on the type, not the variant
- **Row-major sheet layout** — `assets/Floors/tilesheet_6x4_128px.png` is 6 columns × 4 rows of 128×128 tiles; `bevy_ecs_tilemap` slices a `TilemapTexture::Single` image row-major, so `index = row * 6 + column`. Indices `0`–`5` = floor variants (map directly to `floor_variant`), `6`–`21` = the 16 linked walls (see [Rendering](#rendering)). Doors have no art in this sheet yet: `TileData::texture_index()` returns `1` for both open and closed as a temporary placeholder (doors show as floor). `TileType::Door` only carries `is_open: bool`, so a locked state is unimplemented
- **Generation:** `generate_map` (`map/map_gen.rs`) rolls a weighted-random `floor_variant` per tile via `rand::distr::weighted::WeightedIndex` (weights `[50, 10, 10, 1, 1, 10]` at `map_gen.rs:150` — variant 0 dominant, the rest sprinkled in, the oil spill (3) and drain (4) intentionally rare) and `rand::rng()` for the thread-local source; `WeightedIndex::sample` requires the `rand::prelude::Distribution` trait in scope

### Map Generation

- **Status: in progress** — this is the start of the generator in [Procedural generation](#procedural-generation). `generate_map` fills the whole map with `TileType::Wall`, places one hardcoded start room plus random non-overlapping rooms in a `Vec<Room>`, carves them all, then links them with corridors of random widths so every room is reachable, with a few extra links for loops. Dead ends are deferred (see below)
- **Wall first, then carve** — every tile starts as wall and generation only opens up floor. Each tile still gets a `floor_variant` during the fill, so carved tiles get their random floor look without another roll
- **`Room` struct** — private to `map/map_gen.rs`; `pos: (u32, u32)` is the bottom-left corner, plus `width: u32` and `height: u32`. `pos` is a tuple to match `GridPosition`, `Path` and `find_path`. Rooms are kept as data because overlap checks, doors and room types all need them after carving
- **`carve_room(map: &mut Map, room: &Room)`** — sets every tile in the rectangle to `TileType::Floor`. It borrows the `Room` so rooms can live in a `Vec<Room>` afterwards. `y` is the outer loop and `x` the inner one, matching the `x + y * width` layout so memory is walked in order
- **Hardcoded start room** — `pos: (21, 21)`, 19×19, first entry in `rooms`. It has to cover the colonists' hardcoded spawns, `(30, 30)` and `(35, 35)`, or they start inside wall. Being first means every random room is checked against it
- **Random placement** — `generate_map` loops `ROOM_ATTEMPTS` times, rolling a size between `ROOM_MIN_SIZE` and `ROOM_MAX_SIZE` (`constants.rs`, currently `5`–`15`) and a position from `1` to `map.width - room_width - 1` (same for height), which keeps a one-tile wall border around the map. A fixed attempt count always finishes, unlike "loop until N rooms". `random_range` needs `use rand::RngExt;` in rand 0.10
- **`Room::overlaps(&self, other: &Room) -> bool`** — rectangle overlap test using `<=`/`>=`, so touching rooms count as overlapping and at least one wall tile stays between rooms. Written with addition so the `u32`s can't underflow. A rejected room is skipped with `continue` and uses up its attempt
- **`Room::centre` / `Room::distance_to`** — `centre` is the tile at the room's middle, and corridors run between centres, so they always reach floor. `distance_to` is the Manhattan distance between centres in `u32`, which is the exact length of an L-shaped corridor, needs no square root, and can be compared with `Ord`
- **`link_rooms(rooms: &[Room]) -> Vec<(usize, usize)>`** — Prim's algorithm: starting from room 0, each pass finds the closest connected/unconnected pair, pushes it as a link and marks the new room connected. This gives a tree with `rooms.len() - 1` links, each a pair of indices into `rooms`. It rescans every pair each pass, so cost grows with the cube of the room count; fine now, but revisit for thousands of rooms
- **`carve_corridor(map, from, to, width, rng)`** — picks one of the two possible corners at random, then carves an L as two temporary `Room`s through `carve_room`. Each leg is extended by `width / 2` at both ends, so the outer corner has no notch. Widths should be odd, or the corridor comes out a tile short and off-centre. With `ROOM_MIN_SIZE = 5`, centres stay at least 3 tiles from the edge, so widths up to 5 can't reach the outer wall; a width above `ROOM_MIN_SIZE` would need clamping
- **Corridor widths** — each link rolls a width in `generate_map` via `WeightedIndex` over `CORRIDOR_WIDTH_WEIGHTS`, used as an index into `CORRIDOR_WIDTH` (`[1, 3, 5]`, weights `[2, 5, 1]`). The two arrays must stay the same length
- **`extra_links(rooms, links, rng)`** — runs `EXTRA_LINKS` times (currently `3`): picks a random room and links it to its nearest room that isn't already linked to it, checking both pair orders. Nearest, not random, so the loop stays local and doesn't bypass several chokepoints at once. Runs after `link_rooms` and before the carve loop, so extra links get widths and corridors like any other link
- **Dead ends: deferred by the developer, not built.** Planned approach: `carve_corridor` from a room's centre to a point in a straight line, which it already handles because a gap of 0 on one axis flattens the L
- The earlier maze version and odd room alignment were removed when the design changed
- **Layout direction (decided 2026-11-10): fused wrecks.** The current output reads as a roguelike dungeon, not a ship: rooms sit anywhere in solid rock, walls are thick, corridors tunnel centre to centre, and there's no hull. New direction: the map starts as void, several wreck hulls are stamped in (overlapping where they fuse), and each wreck's interior is generated inside its own bounds. The existing room/corridor code is expected to be reused per wreck. This is the macro layer from [Procedural generation](#procedural-generation), built first

### HUD / UI

- **Tool choice: native `bevy_ui`** — considered `bevy_egui` (rejected: immediate-mode look reads as editor/debug UI, not a diegetic in-game HUD) and `bevy_lunex` (rejected for now: better suited to non-rectangular/angular HUD layout, but a smaller third-party crate carrying the same version-lag risk that blocked the Bevy 0.19 upgrade with `bevy_ecs_tilemap`); modern `bevy_ui` (rounded corners, box shadows, gradients) covers the sci-fi panel look via 9-slice panel textures and translucent `BackgroundColor`s without that dependency risk — a custom `UiMaterial`/WGSL shader is the fallback for animated effects (scanlines, pulsing glow) if plain nodes aren't enough
- **Root node pattern** — `UiPlugin::build` registers `spawn_hud_root` on `Startup`; it spawns a single entity with `Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }` as the parent every HUD panel will be a child of; `BackgroundColor` was added temporarily to visually confirm the node's placement/size, then removed — the root itself must stay invisible (an opaque full-screen `BackgroundColor` blocks the world view underneath), visible color belongs only on child panels sized to their own content
- **Resource bar categorisation** — the top bar is split by urgency rather than shown as one flat row of icons: **life-support** (oxygen, food, water, energy) is always visible and built first, since zero on any of these means colonists start dying; **population** is a colony-status count, not a depletable resource, so it gets its own slot rather than sitting in the resource-bar style; **stockpile materials** (scrap metal, refined metals, bullets) are deferred — likely belong in a build/loadout panel where they're actually spent rather than the always-on HUD bar, with ammo count as a possible exception since combat is real-time
- **`LifeSupport` resource** — lives in `ui/life_support.rs`; `pub` `oxygen: f32`, `food: f32`, `water: f32`, `energy: f32` fields, `#[derive(Resource)]`; starting values (`100.0` each) come from a `Default` impl; inserted by `LifeSupportPlugin`, which is added in `main.rs`
- **Life-support bar** — `spawn_hud_root` spawns a 40px translucent bar as a child of the root node, with four `Text` children tagged `OxygenText`, `FoodText`, `WaterText` and `EnergyText`; text is plain `"Label: value"` strings, with no bars or icons
- **`update_life_support_hud` system** — rewrites the four texts, gated by `.run_if(resource_exists_and_changed::<LifeSupport>)` so it does nothing on frames where the values didn't change; uses a `ParamSet` because all four queries take `&mut Text`, which Bevy would otherwise reject as conflicting access. Nothing mutates `LifeSupport` yet, so the values stay at `100`
- Longer term, see [UI & input](#ui--input) (controller support, UI swapping by active input device)

### Memory & Performance Review

Reviewed by reading the code on 2026-11-06, not by profiling. Sizes are calculated estimates. None of the problems below have been fixed yet.

- **Stack:** no concerns; only small values and short-lived locals, with no recursion in these systems
- **Working well:** small, separate components suit ECS's packed arrays; `Map` and `FlowField` are flat arrays; `FlowField` and `rebuild_colonist_flow_field` reuse their buffers; the `Vec<Vec2>` snapshot in `separate_enemies` packs 8 bytes per enemy instead of reading a ~48-byte `Transform`
- **Per-frame heap allocations (breaks the "no allocations in per-frame code" rule):** about nine each frame, in `move_enemy`, `separate_enemies` (2), `separate_colonists` (2), `enemy_attack`, `colonist_attack`, `decide_colonist_animation` and `move_character` (`HashSet`). The `Vec::new()` + `push` snapshots also reallocate as they grow. `Local<Vec<...>>` avoids both
- **`find_path` allocations:** three map-sized arrays per call, so moving a group of colonists allocates them once per colonist
- **Wasted maths in `separate_enemies`:** `diff.length()` runs a square root before the distance check rejects the pair; about 12.5 million pairs per frame at 5000 enemies. Comparing `length_squared()` first skips most of them; only a spatial hash fixes the O(N²) itself
- **Colonist × enemy scans:** `enemy_attack`, `colonist_attack` and `decide_colonist_animation` each compare every colonist with every enemy, which multiplies with no colonist limit
- **`TileData` size:** about 8 bytes because `floor_variant` is a `u32` holding 0–5; as a `u8` it would be about 2 bytes
- **Whole-map arrays at target size (5120×5120, per floor):** `Map` ~210MB (~52MB with a `u8` variant), each `FlowField` ~183MB, each `find_path` call ~236MB. `FlowFields` allocates three layers though only `colonists` is used, which would be about 1.6GB across 3 layers and 3 floors. `build_flow_fields` also refills the whole field whenever a colonist changes tile. Chunks need to solve this, rather than these arrays scaling up

## Bevy Upgrade Notes

Now on **Bevy 0.20.1** with `bevy_ecs_tilemap` `0.20.0` and `rand` `0.10.3`, bumped in commit `738ef02`. No 0.20-specific migration notes were recorded. The notes below are from the 0.19 upgrade.

Upgraded from 0.18.1 to **Bevy 0.19.0** (`bevy_ecs_tilemap` bumped to `0.19.0` alongside it, resolving the prior blocker). `cargo build` is clean with no source changes required beyond `Cargo.toml`.

- **Audio feature no longer implied** — actioned: `audio` added to the `bevy` features list in `Cargo.toml` explicitly (`features = ["dynamic_linking", "audio"]`); without it `ambient_spaceship.ogg` would silently stop playing
- **Resources as Components** — no impact observed; no query in this codebase is broad enough (e.g. `Query<()>`) to collide with resource entities
- **`Assets::get_mut` return type changed** — no impact; this project doesn't call `.get_mut()` on an `Assets<T>` resource
- **Scene system renamed** (`bevy_scene` → `bevy_world_serialization`) — no impact; scenes aren't used yet, revisit naming when they are
- **Rendering pipeline overhaul** — no impact; no custom render code in this project, `map_renderer.rs` builds and runs unaffected

If new breaking changes surface while working in 0.20, add them here.

## Planned Features / TODO

- **Sprite animations / sprite sheets** — in progress for the knight colonist, see [Character Animation](#character-animation) (Idle/Walk/Attack wired; Carry/Work/Sleep, skin colour and enemy animation still to build); current sheets are placeholders until the Blender pipeline is running
- **Game saves (binary serialization)** — serialize world state (map, colonist positions/health, enemy state, buildings) to a compact binary format for save/load; consider `bincode` + `serde` derives or a custom flat-buffer approach for performance
- **Spatial queries** — replace O(N²) nearest-neighbour loops in combat and separation systems with a spatial structure; candidates: KD-tree (static or semi-static entities) or spatial hashing (dynamic, grid-aligned entities like colonists and enemies)
- **Procedural generation** — in progress, see [Map Generation](#map-generation); grow `map_gen.rs` into the layered generator described in [Procedural generation](#procedural-generation) (macro noise → rooms and linked corridors → clutter, 3 floor styles, lifts), hooked into the chunk system so new chunks generate on reveal
- **Chunks & 3 floors** — see [World & map](#world--map)
- **Horde tiers, enemy director, worm** — see [Horde](#horde) and [Enemy director](#enemy-director)
- **Colonist scale, jobs & automation, stories, possession** — see [Colonists](#colonists) and [Possession](#possession-hero-moment)
- **UI** — in progress, see [HUD / UI](#hud--ui); tool decided (`bevy_ui`), root node and life-support bar in place; still open: population slot, stockpile materials panel, colonist status bars, selected-unit info panel, minimap, build menu
