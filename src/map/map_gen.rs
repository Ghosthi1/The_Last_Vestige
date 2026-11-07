use super::map::Map;
use crate::constants::{ROOM_ATTEMPTS, ROOM_MAX_SIZE, ROOM_MIN_SIZE};
use crate::map::TileType;
use rand::RngExt;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;

/// A rectangular room carved out of the wall map
struct Room {
    /// The Bottom left corner of the room
    pos: (u32, u32),
    /// The width of the room
    width: u32,
    /// The height of the room
    height: u32,
}
impl Room {
    /// Checks if this room overlaps with another room or touches another room
    fn overlaps(&self, other: &Room) -> bool {
        self.pos.0 <= other.pos.0 + other.width
            && self.pos.0 + self.width >= other.pos.0
            && self.pos.1 <= other.pos.1 + other.height
            && self.pos.1 + self.height >= other.pos.1
    }
}

/// Carves a room out of the map by setting all the tiles in the room to be floor tiles
fn carve_room(map: &mut Map, room: &Room) {
    for y in room.pos.1..room.pos.1 + room.height {
        for x in room.pos.0..room.pos.0 + room.width {
            let tile = map.set(x, y);
            tile.tile_type = TileType::Floor;
        }
    }
}

/// Creates a Map of the given dimensions, with walls and rooms carved out of it
pub fn generate_map(width: u32, height: u32) -> Map {
    let mut map = Map::new(width, height);

    let mut rooms = vec![Room {
        pos: (21, 21),
        width: 19,
        height: 19,
    }];

    let weights = [50u32, 10, 10, 10, 1, 10]; // weights for how often a tile gets chosen
    let dist = WeightedIndex::new(weights).unwrap();
    let mut rng = rand::rng();

    // fill the map with walls and floor tiles based on the weights
    for y in 0..map.height {
        for x in 0..map.width {
            let index = dist.sample(&mut rng);
            let tile = map.set(x, y);
            tile.floor_variant = index as u32;
            tile.tile_type = TileType::Wall;
        }
    }

    // generates the rooms
    for _ in 0..ROOM_ATTEMPTS {
        let room_width = rng.random_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
        let room_height = rng.random_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);

        let room_x = rng.random_range(1..=map.width - room_width - 1);
        let room_y = rng.random_range(1..=map.height - room_height - 1);

        let new_room = Room {
            pos: (room_x, room_y),
            width: room_width,
            height: room_height,
        };

        if rooms.iter().any(|r| r.overlaps(&new_room)) {
            continue;
        }

        rooms.push(new_room);
    }

    for room in &rooms {
        carve_room(&mut map, room);
    }

    map
}
