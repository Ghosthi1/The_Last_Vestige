use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::map::Map;

/// Manages all map rendering systems
pub struct MapRendererPlugin;

const TILE_SIZE: f32 = 16.0;
impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tilemap);
    }
}
/// Startup system that spawns the tilemap entities from the MAp resource and applies tile textures from the spritesheet
pub fn spawn_tilemap(map: Res<Map>,asset_server: Res<AssetServer>, mut command:Commands) {

    let texture_handle: Handle<Image> = asset_server.load("PlaceHolder_tileset1.png"); // Loads the SpriteSheet
    let map_size = TilemapSize{x:map.width,y:map.height}; // Used to put aside correct amount of resources
    let tilemap_entity = command.spawn_empty().id(); // Reserves a unique ID for the tilemap entity so tiles can reference it before it's fully built

    let mut tile_storage = TileStorage::empty(map_size); // Maps grid positions to tile entities for fast lookup

    for x in 0..map.width {
        for y in 0..map.height {
            let tile_pos = TilePos{x,y};
            let tile_entity = command.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(map.get(x,y).tile_type.texture_index()),
                ..Default::default()
            }).id();
            tile_storage.set(&tile_pos, tile_entity); // Adds the tile to the entity storage
        }
    }

    // Completes the tilemap entity with rendering config
    command.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TilemapGridSize { x: TILE_SIZE, y: TILE_SIZE },
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TilemapTileSize {x:TILE_SIZE, y:TILE_SIZE},
        transform: Transform::from_xyz(
            -(map.width as f32 * TILE_SIZE) / 2.0,
            -(map.height as f32 * TILE_SIZE) / 2.0,
            0.0 ),
        ..Default::default()
    });
}