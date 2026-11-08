use crate::constants::TILE_SIZE;
use crate::map::{Map, TileType};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::prelude::*;
use crate::systems::y_to_z;

/// Manages all map rendering systems
pub struct MapRendererPlugin;
impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tilemap);
        app.add_systems(Startup, spawn_edge_walls);
    }
}

/// Tags the edge walls for rendering purposes
#[derive(Component)]
pub struct EdgeWall(pub (u32, u32));

/// Startup system that spawns the tilemap entities from the MAp resource and applies tile textures from the spritesheet
pub fn spawn_tilemap(map: Res<Map>, asset_server: Res<AssetServer>, mut command: Commands) {
    let texture_handle: Handle<Image> = asset_server.load("Floors/tilesheet_6x3_128x112px.png"); // Loads the SpriteSheet
    let map_size = TilemapSize {
        x: map.width,
        y: map.height,
    }; // Used to put aside correct amount of resources
    let tilemap_entity = command.spawn_empty().id(); // Reserves a unique ID for the tilemap entity so tiles can reference it before it's fully built

    let mut tile_storage = TileStorage::empty(map_size); // Maps grid positions to tile entities for fast lookup

    for x in 0..map.width {
        for y in 0..map.height {
            let tile_pos = TilePos { x, y };
            let tile_entity = command
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(map.get(x, y).texture_index()),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity); // Adds the tile to the entity storage
        }
    }

    // Completes the tilemap entity with rendering config
    command.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TilemapGridSize {
            x: TILE_SIZE.x,
            y: TILE_SIZE.y,
        },
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TilemapTileSize {
            x: TILE_SIZE.x,
            y: TILE_SIZE.y,
        },
        transform: Transform::from_xyz(
            -(map.width as f32 * TILE_SIZE.x) / 2.0 + TILE_SIZE.x / 2.0,
            -(map.height as f32 * TILE_SIZE.y) / 2.0 + TILE_SIZE.y / 2.0,
            0.0,
        ),
        ..Default::default()
    });
}

/// Startup system that spawns a bottom anchored wall sprite for every tile
pub fn spawn_edge_walls(map: Res<Map>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let wall_edge = asset_server.load("Walls/wall_edge_01.png");

    for y in 0..map.height {
        for x in 0..map.width {
            if map.is_edge_wall(x, y) {
                let face_exposed = y > 0 && !matches!(map.get(x, y - 1).tile_type, TileType::Wall);

                if !face_exposed {
                    continue;
                }

                let base = (map.grid_to_world(x, y) - Vec2::new(0.0, TILE_SIZE.y / 2.0));

                commands.spawn((
                    EdgeWall((x, y)),
                    Transform::from_translation(base.extend(y_to_z(base.y))),
                    Sprite {
                        image: wall_edge.clone(),
                        ..default()
                    },
                    Anchor::BOTTOM_CENTER,
                ));
            }
        }
    }
}