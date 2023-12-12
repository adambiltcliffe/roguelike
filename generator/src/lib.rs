use std::collections::{HashMap, HashSet};

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
    add_room(&mut data, &Rect::new(50, 50, 7, 10));
    add_room(&mut data, &Rect::new(50, 61, 10, 7));
    for _ in 0..100 {
        if data.corners.is_empty() {
            break;
        }
        let nc = data
            .corners
            .swap_remove(rand::gen_range(0, data.corners.len()));
        try_corner(&mut data, nc);
    }
    let mut conns: HashMap<usize, HashMap<usize, usize>> = HashMap::new();
    for i in 0..data.rooms.len() {
        conns.insert(i, HashMap::new());
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
                    let idx = data.walls.len() - 1;
                    conns.get_mut(&i1).unwrap().insert(i2, idx);
                    conns.get_mut(&i2).unwrap().insert(i1, idx);
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
                    let idx = data.walls.len() - 1;
                    conns.get_mut(&i1).unwrap().insert(i2, idx);
                    conns.get_mut(&i2).unwrap().insert(i1, idx);
                }
            }
        }
    }
    let mut closed = HashSet::new();
    let mut open = Vec::new();
    closed.insert(0);
    for (i, _) in conns.get(&0).unwrap() {
        open.push(i);
    }
    while let Some(r) = pop_random(&mut open) {
        closed.insert(*r);
        let mut wall_idxs = Vec::new();
        for (ri, wi) in conns.get(r).unwrap() {
            if closed.contains(ri) {
                wall_idxs.push(*wi);
            } else {
                if !open.contains(&ri) {
                    open.push(ri);
                }
            }
        }
        // wall_idxs contains the indices into data.walls which could connect this room
        let force = rand::gen_range(0, wall_idxs.len());
        for (i, wi) in wall_idxs.iter().enumerate() {
            if i == force || rand::gen_range(0, 100) < 35 {
                open_wall(&mut data.map, &data.walls[*wi]);
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

impl InternalWall {
    fn length(&self) -> i32 {
        match self {
            InternalWall::Vertical { h, .. } => *h,
            InternalWall::Horizontal { w, .. } => *w,
        }
    }
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
    let w = rand::gen_range(5, 12);
    let h = rand::gen_range(5, 12);
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

fn open_wall(map: &mut Map, w: &InternalWall) {
    if rand::gen_range(0, 5) == 0 {
        for i in 0..w.length() {
            open_tile_in_wall(map, w, i);
        }
    } else if w.length() > 3 && rand::gen_range(0, 5) == 0 {
        let i = rand::gen_range(1, w.length() - 2);
        open_tile_in_wall(map, w, i);
        open_tile_in_wall(map, w, i + 1);
    } else {
        open_tile_in_wall(map, w, rand::gen_range(0, w.length()));
    }
}

fn open_tile_in_wall(map: &mut Map, w: &InternalWall, i: i32) {
    match w {
        InternalWall::Vertical { x, y, .. } => {
            map.set_tile(*x, *y + i, Tile::Floor);
        }
        InternalWall::Horizontal { x, y, .. } => {
            map.set_tile(*x + i, *y, Tile::Floor);
        }
    }
}

fn pop_random<T>(v: &mut Vec<T>) -> Option<T> {
    if v.is_empty() {
        return None;
    }
    let idx = rand::gen_range(0, v.len());
    Some(v.swap_remove(idx))
}
