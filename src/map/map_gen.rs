use super::map::Map;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;


/// Creates a Map of the given dimensions, then randomly assigns a floor to it and returns the finished map.
pub fn generate_map(width: u32, height: u32) -> Map {
    let mut map = Map::new(width, height);

    let weights = [50u32, 10, 10, 10, 1, 10];     // weights for how often a tile gets chosen
    let dist = WeightedIndex::new(weights).unwrap();
    let mut rng = rand::rng();

    for x in 0..map.width {
        for y in 0..map.height {
            let index = dist.sample(&mut rng);
            let tile = map.set(x,y);
            tile.floor_variant = index as u32;
        }
    }

    map
}