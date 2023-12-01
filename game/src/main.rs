use macroquad::prelude::*;
use map;

#[macroquad::main("Roguelike")]
async fn main() {
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
            if macroquad::rand::gen_range(75, 150) > x {
                map.set_tile(x, y, map::Tile::Empty);
            }
        }
    }
    map.set_tile(50, 51, map::Tile::Solid);
    map.set_tile(52, 50, map::Tile::Solid);
    map.set_tile(52, 51, map::Tile::Solid);
    let mut fov = map::Viewshed::new_at(15, 50, 50, &map);
    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            fov.update(mx as i32 / 6, my as i32 / 6, &map);
        }
        render(&map, &fov);
        next_frame().await;
    }
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

fn render(map: &map::Map, fov: &map::Viewshed) {
    let b = map.get_bounds();
    for y in b.y..(b.y + b.h) {
        for x in b.x..(b.x + b.w) {
            let c = match (map.get_tile(x, y), fov.contains(x, y)) {
                (map::Tile::Solid, false) => DARKBLUE,
                (map::Tile::Solid, true) => GREEN,
                (map::Tile::Empty, true) => GRAY,
                (map::Tile::Empty, false) => DARKGRAY,
                (map::Tile::Void, _) => BLACK,
            };
            draw_rectangle(x as f32 * 6.0, y as f32 * 6.0, 6.0, 6.0, c);
        }
    }
}
