use generator;
use macroquad::prelude::*;
use map;

#[macroquad::main("Roguelike")]
async fn main() {
    let map = generator::make_world();
    let mut fov = map::Viewshed::new_at(35, 50, 50, &map);
    loop {
        if true || is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            fov.update(mx as i32 / 6, my as i32 / 6, &map);
        }
        render(&map, &fov);
        next_frame().await;
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
