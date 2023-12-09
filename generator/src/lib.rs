use macroquad::rand;
use map::{Map, Tile};

pub fn make_world() -> Map {
    let mut map = Map::new();
    add_room(&mut map, 50, 50, 7, 12);
    let mut corners = add_room(&mut map, 50, 63, 15, 9);
    for _ in 0..40 {
        if corners.is_empty() {
            break;
        }
        let nc = corners.swap_remove(rand::gen_range(0, corners.len()));
        corners.append(&mut try_corner(&mut map, nc));
    }
    map
}

#[derive(Debug)]
enum CornerType {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug)]
struct Corner {
    x: i32,
    y: i32,
    typ: CornerType,
}

impl Corner {
    fn new(x: i32, y: i32, typ: CornerType) -> Self {
        Self { x, y, typ }
    }
}

fn add_room(map: &mut Map, x: i32, y: i32, w: i32, h: i32) -> Vec<Corner> {
    let mut result = Vec::new();
    for cx in x..(x + w) {
        for cy in y..(y + h) {
            map.set_tile(cx, cy, Tile::Floor);
        }
    }
    map.set_tile(x - 1, y - 1, Tile::Wall);
    map.set_tile(x - 1, y + h, Tile::Wall);
    map.set_tile(x + w, y - 1, Tile::Wall);
    map.set_tile(x + w, y + h, Tile::Wall);
    for cx in x..(x + w) {
        map.set_tile(cx, y - 1, Tile::Wall);
        map.set_tile(cx, y + h, Tile::Wall);
        if map.get_tile(cx, y - 2) == Tile::Void {
            if map.get_tile(cx - 1, y - 2) == Tile::Wall {
                result.push(Corner::new(cx, y - 2, CornerType::BottomLeft));
            }
            if map.get_tile(cx + 1, y - 2) == Tile::Wall {
                result.push(Corner::new(cx, y - 2, CornerType::BottomRight));
            }
        }
        if map.get_tile(cx, y + h + 1) == Tile::Void {
            if map.get_tile(cx - 1, y + h + 1) == Tile::Wall {
                result.push(Corner::new(cx, y + h + 1, CornerType::TopLeft));
            }
            if map.get_tile(cx + 1, y + h + 1) == Tile::Wall {
                result.push(Corner::new(cx, y + h + 1, CornerType::TopRight));
            }
        }
    }
    for cy in y..(y + h) {
        map.set_tile(x - 1, cy, Tile::Wall);
        map.set_tile(x + w, cy, Tile::Wall);
        if map.get_tile(x - 2, cy) == Tile::Void {
            if map.get_tile(x - 2, cy - 1) == Tile::Wall {
                result.push(Corner::new(x - 2, cy, CornerType::TopRight));
            }
            if map.get_tile(x - 2, cy + 1) == Tile::Wall {
                result.push(Corner::new(x - 2, cy, CornerType::BottomRight));
            }
        }
        if map.get_tile(x + w + 1, cy) == Tile::Void {
            if map.get_tile(x + w + 1, cy - 1) == Tile::Wall {
                result.push(Corner::new(x + w + 1, cy, CornerType::TopLeft));
            }
            if map.get_tile(x + w + 1, cy + 1) == Tile::Wall {
                result.push(Corner::new(x + w + 1, cy, CornerType::BottomLeft));
            }
        }
    }
    result
}

fn bounds_for_corner(c: &Corner) -> (i32, i32, i32, i32) {
    let w = rand::gen_range(5, 15);
    let h = rand::gen_range(5, 15);
    match c.typ {
        CornerType::TopLeft => (c.x, c.y, w, h),
        CornerType::TopRight => (c.x - w + 1, c.y, w, h),
        CornerType::BottomLeft => (c.x, c.y - h + 1, w, h),
        CornerType::BottomRight => (c.x - w + 1, c.y - h + 1, w, h),
    }
}

fn try_corner(map: &mut Map, corner: Corner) -> Vec<Corner> {
    let (x, y, w, h) = bounds_for_corner(&corner);
    for cx in x..(x + w) {
        for cy in y..(y + h) {
            if map.get_tile(cx, cy) != Tile::Void {
                return Vec::new();
            }
        }
    }
    add_room(map, x, y, w, h)
}
