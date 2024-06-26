use std::collections::{HashMap, HashSet};

use macroquad::rand;
use map::{Map, Rect, Tile};

type Pattern = [Tile; 9];

const SIZE: i32 = 40;

fn get_pattern_at(map: &Map, x: i32, y: i32) -> Pattern {
    [
        map.get_tile(x, y),
        map.get_tile(x + 1, y),
        map.get_tile(x + 2, y),
        map.get_tile(x, y + 1),
        map.get_tile(x + 1, y + 1),
        map.get_tile(x + 2, y + 1),
        map.get_tile(x, y + 2),
        map.get_tile(x + 1, y + 2),
        map.get_tile(x + 2, y + 2),
    ]
}

pub struct InteractiveMutator {
    weights: HashMap<Pattern, f64>,
    k_max: Pattern,
    w_max: f64,
}

pub fn make_mutator() -> InteractiveMutator {
    use std::fs::File;
    let decoder = png::Decoder::new(File::open("CaveMaze.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let (ct, bd) = reader.output_color_type();
    let info = reader.next_frame(&mut buf).unwrap();
    println!("Read the data");
    println!("Buffer size was {}", buf.len());
    println!("Colour type and bit depth are {:?}, {:?}", ct, bd);
    println!("Pixel dimensions are {}x{}", info.width, info.height);
    println!("Bytes per scanline is {}", info.line_size);
    println!("{:?}", buf);
    // note to set: Map is the WRONG data structure to use here in the long term
    let mut map = Map::new();
    for y in 0..(info.height as i32) {
        for x in (0..info.width as i32) {
            if buf[(3 * (y * info.width as i32 + x)) as usize] == 0 {
                print!("#");
                map.set_tile(x, y, Tile::Wall);
            } else {
                print!(".");
                map.set_tile(x, y, Tile::Floor);
            }
        }
        println!();
    }
    let mut weights: HashMap<Pattern, f64> = HashMap::new();
    for y in 0..(info.height as i32 - 2) {
        for x in (0..info.width as i32 - 2) {
            let p: Pattern = get_pattern_at(&map, x, y);
            *weights.entry(p).or_default() += 1.0;
        }
    }
    println!("cataloguing complete, {} patterns observed", weights.len());
    let f = ((SIZE * SIZE) as f64) / ((info.width * info.height) as f64);
    println!("adjustment factor is {}", f);
    for v in weights.values_mut() {
        *v *= f;
    }
    let (&k_max, &w_max) = weights.iter().max_by(|a, b| a.1.total_cmp(b.1)).unwrap();
    println!("max weight is {}, key is {:?}", w_max, k_max);
    InteractiveMutator {
        weights,
        w_max,
        k_max,
    }
}

fn calc_chi_sq(weights: &HashMap<Pattern, f64>, freqs: &HashMap<Pattern, i32>) -> f64 {
    let mut result = 0.0;
    for (k, v) in freqs.iter() {
        let exp = weights.get(k).cloned().unwrap_or(0.01);
        result += (*v as f64 - exp).powf(2.0) / exp;
    }
    result
}

fn update_chi_sq(
    weights: &HashMap<Pattern, f64>,
    freqs: &HashMap<Pattern, i32>,
    d_freqs: &HashMap<Pattern, i32>,
    old_chi_sq: f64,
) -> f64 {
    let mut result = old_chi_sq;
    for (k, dv) in d_freqs {
        let v = freqs.get(k).cloned().unwrap_or_default();
        let exp = weights.get(k).cloned().unwrap_or(0.01);
        result -= (v as f64 - exp).powf(2.0) / exp;
        result += ((v + dv) as f64 - exp).powf(2.0) / exp;
    }
    result
}

pub fn mutate_map(map: &mut Map, mutator: &InteractiveMutator, n: u32, temperature: f64) {
    let mut freqs: HashMap<Pattern, i32> = HashMap::new();
    for p in mutator.weights.keys() {
        freqs.insert(*p, 0);
    }
    for y in 0..(SIZE - 2) {
        for x in 0..(SIZE - 2) {
            let p = get_pattern_at(map, x, y);
            *(freqs.entry(p).or_default()) += 1;
        }
    }
    let mut chi_sq = calc_chi_sq(&mutator.weights, &freqs);
    println!("chi squared at start: {}", chi_sq);
    for _ in 0..n {
        // VERY heavily unoptimized
        let cx = macroquad::rand::gen_range(0, SIZE);
        let cy = macroquad::rand::gen_range(0, SIZE);
        let x_min = (cx - 2).max(0);
        let x_max = (cx + 2).min(SIZE - 3);
        let y_min = (cy - 2).max(0);
        let y_max = (cy + 2).min(SIZE - 3);
        let mut d_freqs: HashMap<Pattern, i32> = HashMap::new();
        for x in x_min..=x_max {
            for y in y_min..=y_max {
                *d_freqs.entry(get_pattern_at(map, x, y)).or_default() -= 1;
            }
        }
        let mut opts: Vec<(f64, Tile, f64, _)> = Vec::new();
        for cand_tile in [Tile::Wall, Tile::Floor] {
            let mut temp_d_freqs = d_freqs.clone();
            map.set_tile(cx, cy, cand_tile);
            for x in x_min..=x_max {
                for y in y_min..=y_max {
                    *temp_d_freqs.entry(get_pattern_at(map, x, y)).or_default() += 1;
                }
            }
            let new_chi_sq = update_chi_sq(&mutator.weights, &freqs, &temp_d_freqs, chi_sq);
            let p = f64::exp((chi_sq - new_chi_sq) / temperature);
            opts.push((p, cand_tile, new_chi_sq, temp_d_freqs));
        }
        let mut r = macroquad::rand::gen_range(0.0, opts.iter().map(|t| t.0).sum());
        while r > opts.last().unwrap().0 {
            r -= opts.last().unwrap().0;
            opts.pop();
        }
        let (_, new_tile, new_chi_sq, new_freqs) = opts.pop().unwrap();
        map.set_tile(cx, cy, new_tile);
        for (k, v) in new_freqs {
            *(freqs.entry(k).or_default()) += v;
        }
        chi_sq = new_chi_sq;
    }
    println!("chi squared at end: {}", chi_sq);
    println!(
        "freq for max-weighted pattern is {}, target is {}",
        freqs[&mutator.k_max], mutator.w_max
    )
}

pub fn make_world() -> Map {
    let mut map = Map::new();
    for y in 0..SIZE {
        for x in 0..SIZE {
            let r = 0; //macroquad::rand::rand();
            if r % 3 == 0 {
                map.set_tile(x, y, Tile::Floor)
            } else if r % 3 == 1 {
                map.set_tile(x, y, Tile::Water)
            } else {
                map.set_tile(x, y, Tile::Wall)
            }
        }
    }
    map
}
