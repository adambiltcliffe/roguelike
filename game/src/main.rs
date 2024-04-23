use generator;
use macroquad::prelude::*;
use map;

#[macroquad::main("Roguelike")]
async fn main() {
    rand::srand(unsafe { std::mem::transmute::<f64, u64>(macroquad::time::get_time()) });
    let mut map = generator::make_world();
    let im = generator::make_mutator();
    let mut fov = map::Viewshed::new_at(35, 50, 50, &map);
    loop {
        if true || is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            fov.update(mx as i32 / 6, my as i32 / 6, &map);
        }
        render(&map, &fov);
        generator::mutate_map(&mut map, &im, 100, 100.0);
        next_frame().await;
    }
}

fn render(map: &map::Map, fov: &map::Viewshed) {
    let b = map.get_bounds();
    for y in b.y..(b.y + b.h) {
        for x in b.x..(b.x + b.w) {
            let c = match (map.get_tile(x, y), fov.contains(x, y)) {
                (map::Tile::Wall, false) => BROWN,
                (map::Tile::Wall, true) => YELLOW,
                (map::Tile::Floor, true) => GRAY,
                (map::Tile::Floor, false) => DARKGRAY,
                (map::Tile::Water, true) => SKYBLUE,
                (map::Tile::Water, false) => BLUE,
                (map::Tile::Void, _) => BLACK,
            };
            draw_rectangle(x as f32 * 6.0, y as f32 * 6.0, 6.0, 6.0, c);
        }
    }
}
