use super::map::Map;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;
use crate::map::TileType;

/// A rectangular room carved out of the wall map
struct Room {
    /// The Bottom left corner of the room
    pos: (u32,u32),
    /// The width of the room
    width: u32,
    /// The height of the room
    height: u32,
}


/// Carves a room out of the map by setting all the tiles in the room to be floor tiles
fn carve_room(map: &mut Map, room: &Room){
    for y in room.pos.1..room.pos.1 + room.height {
        for x in room.pos.0..room.pos.0 + room.width {
            let tile = map.set(x,y);
            tile.tile_type = TileType::Floor;
        }
    }
}

/// Creates a Map of the given dimensions, then randomly assigns a floor to it and returns the finished map.
pub fn generate_map(width: u32, height: u32) -> Map {
    let mut map = Map::new(width, height);

    let weights = [50u32, 10, 10, 10, 1, 10]; // weights for how often a tile gets chosen
    let dist = WeightedIndex::new(weights).unwrap();
    let mut rng = rand::rng();

    for x in 0..map.width {
        for y in 0..map.height {
            let index = dist.sample(&mut rng);
            let tile = map.set(x, y);
            tile.floor_variant = index as u32;
            tile.tile_type = TileType::Wall;
        }
    }

    carve_room(&mut map, &Room {pos: (25,25), width: 20, height: 20});

    map
}
