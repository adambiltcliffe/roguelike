use crate::Map;

const P: f32 = 0.8; // permissiveness, 0.5 = min, 1.0 = max
const INFINITY: f32 = 10000.0;

#[derive(Clone)]
struct Line {
    near_x: f32,
    near_y: f32,
    far_x: f32,
    far_y: f32,
    normal_x: f32,
    normal_y: f32,
    dot: f32,
}

impl Line {
    fn new(near_x: f32, near_y: f32, far_x: f32, far_y: f32) -> Self {
        let normal_x = near_y - far_y;
        let normal_y = far_x - near_x;
        let dot = far_x * normal_x + far_y * normal_y;
        Self {
            near_x,
            near_y,
            far_x,
            far_y,
            normal_x,
            normal_y,
            dot,
        }
    }

    fn point_above(&self, x: f32, y: f32) -> bool {
        x * self.normal_x + y * self.normal_y < self.dot
    }

    fn point_below(&self, x: f32, y: f32) -> bool {
        x * self.normal_x + y * self.normal_y > self.dot
    }
}

#[derive(Clone)]
struct View {
    shallow: Line,
    steep: Line,
    alive: bool,
    steep_bumps: Vec<(f32, f32)>,
    shallow_bumps: Vec<(f32, f32)>,
}

impl View {
    fn new() -> Self {
        let shallow = Line::new(1.0 - P, P, INFINITY, 0.0);
        let steep = Line::new(P, 1.0 - P, 0.0, INFINITY);
        Self {
            shallow,
            steep,
            alive: true,
            steep_bumps: Vec::new(),
            shallow_bumps: Vec::new(),
        }
    }

    fn add_shallow_bump(&mut self, x: f32, y: f32) {
        self.shallow = Line::new(self.shallow.near_x, self.shallow.near_y, x, y);
        self.shallow_bumps.push((x, y));
        // maintain the invariant that no previous steep bumps are above the shallow line
        for (bx, by) in &self.steep_bumps {
            if self.shallow.point_above(*bx, *by) {
                self.shallow = Line::new(*bx, *by, self.shallow.far_x, self.shallow.far_y);
            }
        }
    }

    fn add_steep_bump(&mut self, x: f32, y: f32) {
        self.steep = Line::new(self.steep.near_x, self.steep.near_y, x, y);
        self.steep_bumps.push((x, y));
        // maintain the invariant that no previous shallow bumps are below the steep line
        for (bx, by) in &self.shallow_bumps {
            if self.steep.point_below(*bx, *by) {
                self.steep = Line::new(*bx, *by, self.steep.far_x, self.steep.far_y);
            }
        }
    }
}

pub struct Viewshed {
    radius: i32,
    diameter: i32,
    data: Vec<bool>,
    offs_x: i32,
    offs_y: i32,
}

impl Viewshed {
    fn new(radius: i32) -> Self {
        let diameter = radius * 2 + 1;
        Self {
            radius,
            diameter,
            data: vec![false; (diameter * diameter) as usize],
            offs_x: 0,
            offs_y: 0,
        }
    }

    pub fn new_at(radius: i32, x: i32, y: i32, map: &Map) -> Self {
        let mut result = Self::new(radius);
        result.update(x, y, map);
        result
    }

    pub fn update(&mut self, x: i32, y: i32, map: &Map) {
        self.offs_x = x - self.radius;
        self.offs_y = y - self.radius;
        self.data = vec![false; (self.diameter * self.diameter) as usize];
        self.set_visible(x, y);
        self.update_quadrant(x, y, 1, 1, map);
        self.update_quadrant(x, y, -1, 1, map);
        self.update_quadrant(x, y, 1, -1, map);
        self.update_quadrant(x, y, -1, -1, map);
    }

    fn update_quadrant(&mut self, x: i32, y: i32, dx: i32, dy: i32, map: &Map) {
        // x, y is our viewpoint origin
        // c+cx, y+cy is the top-left of the cell under consideration
        // (in our top-left origin co-ordinate system)
        let mut views = vec![View::new()];
        for sx in 0..=(self.radius * 2) {
            for cy in 0..=sx {
                let cx = sx - cy;
                if cx <= self.radius && cy <= self.radius {
                    let d2 = cx.pow(2) + cy.pow(2);
                    if (d2 as f32).sqrt().round() as i32 <= self.radius {
                        let tx = x + cx * dx;
                        let ty = y + cy * dy;
                        let mut new_views = Vec::new();
                        for v in views.iter_mut() {
                            // if bottom left is above shallow line or top right is below steep line, ignore
                            //   (if the very corner only is on the line, still ignore)
                            if v.steep.point_below((cx + 1) as f32, (cy) as f32)
                                || v.shallow.point_above((cx) as f32, (cy + 1) as f32)
                            {
                                continue;
                            }
                            // otherwise, if square does not block vis:
                            // mark as visible if we can see enough of the square depending on P
                            if !map.get_tile(tx, ty).blocks_vision() {
                                // we can see the square unless the "shrunk" bottom left is above the
                                // shallow line or the "shrunk" top right is below the steep line
                                if !v.steep.point_below((cx) as f32 + P, (cy + 1) as f32 - P)
                                    && !v.shallow.point_above((cx + 1) as f32 - P, (cy) as f32 + P)
                                {
                                    self.set_visible(tx, ty);
                                }
                                continue;
                            }
                            // otherwise (square does block vis):
                            self.set_visible(tx, ty);
                            let bl_below_steep = v.steep.point_below((cx) as f32, (cy + 1) as f32);
                            let tr_above_shallow =
                                v.shallow.point_above((cx + 1) as f32, (cy) as f32);
                            if bl_below_steep && tr_above_shallow {
                                // view is fully blocked
                                v.alive = false;
                            } else if bl_below_steep {
                                // TODO update near point of the steep line using the list of shallow bumps
                                v.add_steep_bump((cx + 1) as f32, cy as f32);
                            } else if tr_above_shallow {
                                // TODO update near point of the shallow line using the list of steep bumps
                                v.add_shallow_bump(cx as f32, (cy + 1) as f32);
                            } else {
                                // wall is in middle of view, split the view in two
                                let mut v2 = v.clone();
                                // add as a steep bump to one view and a shallow bump to the other
                                v.add_steep_bump((cx + 1) as f32, (cy) as f32);
                                v2.add_shallow_bump((cx) as f32, (cy + 1) as f32);
                                new_views.push(v2);
                            }
                        }
                        views.retain(|v| v.alive);
                        views.append(&mut new_views);
                    }
                }
            }
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        let ix = x - self.offs_x;
        let iy = y - self.offs_y;
        if ix < 0 || iy < 0 || ix >= self.diameter || iy >= self.diameter {
            return false;
        }
        self.data[(iy * self.diameter + ix) as usize]
    }

    fn set_visible(&mut self, x: i32, y: i32) {
        self.data[((y - self.offs_y) * self.diameter + (x - self.offs_x)) as usize] = true;
    }
}
