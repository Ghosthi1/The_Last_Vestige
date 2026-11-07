use super::map::Map;
use crate::constants::{ROOM_ATTEMPTS, ROOM_MAX_SIZE, ROOM_MIN_SIZE, CORRIDOR_WIDTH, CORRIDOR_WIDTH_WEIGHTS};
use crate::map::TileType;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;
use rand::{Rng, RngExt};

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

    /// Returns the centre of the room as a tuple of (x, y) coordinates
    fn centre(&self) -> (u32, u32) {
        (self.pos.0 + self.width / 2, self.pos.1 + self.height / 2)
    }

    /// Returns the distance from this room to another room
    fn distance_to(&self, other: &Room) -> u32 {
        let (x1, y1) = self.centre();
        let (x2, y2) = other.centre();
        let dx = x1.abs_diff(x2);
        let dy = y1.abs_diff(y2);
        dx + dy
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

/// Builds a list of links between rooms
fn link_rooms(rooms: &[Room]) -> Vec<(usize, usize)> {
    let mut connected = vec![false; rooms.len()];
    connected[0] = true;
    let mut links = Vec::with_capacity(rooms.len() - 1);

    for _ in 1..rooms.len() {
        let mut best_distance = u32::MAX;
        let mut best_pair = (0, 0);

        for a in 0..rooms.len() {
            if !connected[a] {
                continue;
            }
            for b in 0..rooms.len() {
                if connected[b] {
                    continue;
                }
                let distance = rooms[a].distance_to(&rooms[b]);
                if distance < best_distance {
                    best_distance = distance;
                    best_pair = (a, b);
                }
            }
        }
        links.push(best_pair);
        connected[best_pair.1] = true;
    }
    links
}

/// Carves a corridor between two rooms by setting the tiles between them to be floor tiles
fn carve_corridor(map: &mut Map, from: (u32, u32), to: (u32, u32), width: u32, rng: &mut impl Rng) {
    let (x1, y1) = from;
    let (x2, y2) = to;

    let corner = if rng.random_bool(0.5) {
        (x1, y2)
    } else {
        (x2, y1)
    };

    let half = width / 2;
    let left = x1.min(x2);
    let bottom = y1.min(y2);

    carve_room(
        map,
        &Room {
            pos: (left - half, corner.1 - half),
            width: x1.abs_diff(x2) + width,
            height: width,
        },
    );
    carve_room(
        map,
        &Room {
            pos: (corner.0 - half, bottom - half),
            width,
            height: y1.abs_diff(y2) + width,
        },
    );
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

    let links = link_rooms(&rooms);

    let width_dist = WeightedIndex::new(CORRIDOR_WIDTH_WEIGHTS).unwrap();

    for (a, b) in links {
        let index = width_dist.sample(&mut rng);
        let width = CORRIDOR_WIDTH[index];
        carve_corridor(&mut map, rooms[a].centre(), rooms[b].centre(), width, &mut rng);
    }

    map
}
