mod fov;
mod geom;

pub use fov::Viewshed;
pub use geom::Rect;

use std::collections::HashMap;

const BLOCK_SIZE: i32 = 8;
const BLOCK_CELLS: usize = (BLOCK_SIZE * BLOCK_SIZE) as usize;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Tile {
    Wall,
    Floor,
    Void,
}

impl Tile {
    pub fn blocks_vision(&self) -> bool {
        match self {
            Tile::Wall => true,
            Tile::Floor => false,
            Tile::Void => false,
        }
    }
}

type BlockKey = (i32, i32);

pub struct Map {
    bounds: Rect,
    blocks: HashMap<BlockKey, [Tile; BLOCK_CELLS]>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            bounds: Rect::new(0, 0, 1, 1),
            blocks: HashMap::new(),
        }
    }

    fn key_index(x: i32, y: i32) -> (BlockKey, usize) {
        let kx = x.div_euclid(BLOCK_SIZE);
        let ky = y.div_euclid(BLOCK_SIZE);
        let cx = x.rem_euclid(BLOCK_SIZE);
        let cy = y.rem_euclid(BLOCK_SIZE);
        ((kx, ky), (cy * BLOCK_SIZE + cx) as usize)
    }

    pub fn get_bounds(&self) -> &Rect {
        &self.bounds
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Tile {
        let (k, idx) = Self::key_index(x, y);
        match self.blocks.get(&k) {
            Some(data) => data[idx],
            None => Tile::Void,
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        let (k, idx) = Self::key_index(x, y);
        self.blocks.entry(k).or_insert_with(|| {
            let max_x = self.bounds.x + self.bounds.w;
            let max_y = self.bounds.y + self.bounds.h;
            self.bounds.x = self.bounds.x.min(k.0 * BLOCK_SIZE);
            self.bounds.y = self.bounds.y.min(k.1 * BLOCK_SIZE);
            self.bounds.w = (max_x - self.bounds.x).max((k.0 + 1) * BLOCK_SIZE - self.bounds.x);
            self.bounds.h = (max_y - self.bounds.y).max((k.1 + 1) * BLOCK_SIZE - self.bounds.y);
            [Tile::Void; BLOCK_CELLS]
        })[idx] = tile;
    }
}
