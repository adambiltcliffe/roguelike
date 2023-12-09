use macroquad::rand;
use map;

pub fn make_world() -> map::Map {
    let mut map = map::Map::new();
    add_circle_to_map(&mut map, 50, 50, 10);
    for _ in 0..100 {
        let mut cx = 50;
        let mut cy = 50;
        while map.get_tile(cx, cy) == map::Tile::Empty {
            cx += rand::gen_range(-2, 2);
            cy += rand::gen_range(-2, 2);
        }
        add_circle_to_map(&mut map, cx, cy, 5);
    }
    for y in 25..75 {
        for x in 75..150 {
            if rand::gen_range(75, 150) > x {
                map.set_tile(x, y, map::Tile::Empty);
            }
        }
    }
    map.set_tile(50, 51, map::Tile::Solid);
    map.set_tile(52, 50, map::Tile::Solid);
    map.set_tile(52, 51, map::Tile::Solid);
    map
}

fn within_dist(x1: i32, y1: i32, x2: i32, y2: i32, r: i32) -> bool {
    (x1 - x2).pow(2) + (y1 - y2).pow(2) <= (r as f32 + 0.5).powf(2.0).floor() as i32
}

fn add_circle_to_map(map: &mut map::Map, x: i32, y: i32, r: i32) {
    for ix in (-r - 2)..=(r + 2) {
        for iy in (-r - 2)..=(r + 2) {
            if within_dist(x + ix, y + iy, x, y, r) {
                if map.get_tile(x + ix, y + iy) == map::Tile::Void {
                    map.set_tile(x + ix, y + iy, map::Tile::Solid)
                }
                if rand::gen_range(0, 10) < 8 {
                    map.set_tile(x + ix, y + iy, map::Tile::Empty)
                };
            } else if within_dist(x + ix, y + iy, x, y, r + 2)
                && map.get_tile(x + ix, y + iy) == map::Tile::Void
            {
                map.set_tile(x + ix, y + iy, map::Tile::Solid);
            }
        }
    }
}
