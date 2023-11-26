use crate::Map;

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
        for ix in 0..self.diameter {
            for iy in 0..self.diameter {
                let cx = self.offs_x + ix;
                let cy = self.offs_y + iy;
                let d2 = (x - cx).pow(2) + (y - cy).pow(2);
                if (d2 as f32).sqrt().round() as i32 <= self.radius {
                    if can_trace(map, x, y, cx, cy) {
                        self.data[(iy * self.diameter + ix) as usize] = true;
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
}

fn can_trace(map: &Map, x: i32, y: i32, tx: i32, ty: i32) -> bool {
    let xs = (x - tx).abs();
    let ys = (y - ty).abs();
    let steps = xs.max(ys);
    let mut cx = x as f32;
    let mut cy = y as f32;
    let dx = (tx - x) as f32 / steps as f32;
    let dy = (ty - y) as f32 / steps as f32;
    for _ in 0..steps {
        if map
            .get_tile(cx.round() as i32, cy.round() as i32)
            .blocks_vision()
        {
            return false;
        }
        cx += dx;
        cy += dy;
    }
    true
}
