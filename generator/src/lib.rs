use macroquad::rand;
use map::{Map, Rect, Tile};

struct MapgenData {
    map: Map,
    rooms: Vec<Rect>,
    walls: Vec<InternalWall>,
    corners: Vec<Corner>,
}

pub fn make_world() -> Map {
    let mut data = MapgenData {
        map: Map::new(),
        rooms: Vec::new(),
        walls: Vec::new(),
        corners: Vec::new(),
    };
    add_room(&mut data, &Rect::new(50, 50, 7, 12));
    add_room(&mut data, &Rect::new(50, 63, 15, 9));
    for _ in 0..40 {
        if data.corners.is_empty() {
            break;
        }
        let nc = data
            .corners
            .swap_remove(rand::gen_range(0, data.corners.len()));
        try_corner(&mut data, nc);
    }
    for (i1, r1) in data.rooms.iter().enumerate() {
        for (i2, r2) in data.rooms.iter().enumerate() {
            if r1.x + r1.w + 1 == r2.x {
                let max_top = r1.y.max(r2.y);
                let min_bottom = (r1.y + r1.h - 1).min(r2.y + r2.h - 1);
                if max_top <= min_bottom {
                    data.walls.push(InternalWall::Vertical {
                        x: r2.x - 1,
                        y: max_top,
                        h: min_bottom - max_top + 1,
                    });
                }
            }

            if r1.y + r1.h + 1 == r2.y {
                let max_left = r1.x.max(r2.x);
                let min_right = (r1.x + r1.w - 1).min(r2.x + r2.w - 1);
                if max_left <= min_right {
                    data.walls.push(InternalWall::Horizontal {
                        x: max_left,
                        y: r2.y - 1,
                        w: min_right - max_left + 1,
                    });
                }
            }
        }
    }
    for w in data.walls {
        match w {
            InternalWall::Vertical { x, y, h } => {
                data.map.set_tile(x, y + rand::gen_range(0, h), Tile::Floor);
            }
            InternalWall::Horizontal { x, y, w } => {
                data.map.set_tile(x + rand::gen_range(0, w), y, Tile::Floor);
            }
        }
    }
    data.map
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

enum InternalWall {
    Vertical { x: i32, y: i32, h: i32 },
    Horizontal { x: i32, y: i32, w: i32 },
}

fn add_room(data: &mut MapgenData, rect: &Rect) {
    data.rooms.push(rect.clone());
    let Rect { x, y, w, h } = *rect;
    for cx in x..(x + w) {
        for cy in y..(y + h) {
            data.map.set_tile(cx, cy, Tile::Floor);
        }
    }
    data.map.set_tile(x - 1, y - 1, Tile::Wall);
    data.map.set_tile(x - 1, y + h, Tile::Wall);
    data.map.set_tile(x + w, y - 1, Tile::Wall);
    data.map.set_tile(x + w, y + h, Tile::Wall);
    for cx in x..(x + w) {
        data.map.set_tile(cx, y - 1, Tile::Wall);
        data.map.set_tile(cx, y + h, Tile::Wall);
        if data.map.get_tile(cx, y - 2) == Tile::Void {
            if data.map.get_tile(cx - 1, y - 2) == Tile::Wall {
                data.corners
                    .push(Corner::new(cx, y - 2, CornerType::BottomLeft));
            }
            if data.map.get_tile(cx + 1, y - 2) == Tile::Wall {
                data.corners
                    .push(Corner::new(cx, y - 2, CornerType::BottomRight));
            }
        }
        if data.map.get_tile(cx, y + h + 1) == Tile::Void {
            if data.map.get_tile(cx - 1, y + h + 1) == Tile::Wall {
                data.corners
                    .push(Corner::new(cx, y + h + 1, CornerType::TopLeft));
            }
            if data.map.get_tile(cx + 1, y + h + 1) == Tile::Wall {
                data.corners
                    .push(Corner::new(cx, y + h + 1, CornerType::TopRight));
            }
        }
    }
    for cy in y..(y + h) {
        data.map.set_tile(x - 1, cy, Tile::Wall);
        data.map.set_tile(x + w, cy, Tile::Wall);
        if data.map.get_tile(x - 2, cy) == Tile::Void {
            if data.map.get_tile(x - 2, cy - 1) == Tile::Wall {
                data.corners
                    .push(Corner::new(x - 2, cy, CornerType::TopRight));
            }
            if data.map.get_tile(x - 2, cy + 1) == Tile::Wall {
                data.corners
                    .push(Corner::new(x - 2, cy, CornerType::BottomRight));
            }
        }
        if data.map.get_tile(x + w + 1, cy) == Tile::Void {
            if data.map.get_tile(x + w + 1, cy - 1) == Tile::Wall {
                data.corners
                    .push(Corner::new(x + w + 1, cy, CornerType::TopLeft));
            }
            if data.map.get_tile(x + w + 1, cy + 1) == Tile::Wall {
                data.corners
                    .push(Corner::new(x + w + 1, cy, CornerType::BottomLeft));
            }
        }
    }
}

fn bounds_for_corner(c: &Corner) -> Rect {
    let w = rand::gen_range(5, 15);
    let h = rand::gen_range(5, 15);
    match c.typ {
        CornerType::TopLeft => Rect::new(c.x, c.y, w, h),
        CornerType::TopRight => Rect::new(c.x - w + 1, c.y, w, h),
        CornerType::BottomLeft => Rect::new(c.x, c.y - h + 1, w, h),
        CornerType::BottomRight => Rect::new(c.x - w + 1, c.y - h + 1, w, h),
    }
}

fn try_corner(data: &mut MapgenData, corner: Corner) {
    let Rect { x, y, w, h } = bounds_for_corner(&corner);
    for cx in x..(x + w) {
        for cy in y..(y + h) {
            if data.map.get_tile(cx, cy) != Tile::Void {
                return;
            }
        }
    }
    add_room(data, &Rect { x, y, w, h })
}
