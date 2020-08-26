use std::collections::*;
use std::cmp::*;
use crate::map::*;

pub fn create_flow_map(map: &Map, sources: usize) -> Map {
    let width = map.width();
    let height = map.height();
    let mut flow_map = Map::new(width, height);

    let random_position = || (rand::random::<usize>() % width, rand::random::<usize>() % height);


    let range = 6;

    for _ in 0..sources {
        let (mut x, mut y) = random_position();
        loop {
            if x < range || x >= width-range || y < range || y >= height-range {
                break;
            }

            let (nx, ny) = next_target(map, range, x, y);

            draw_line(&mut flow_map, x as _, y as _, nx as _, ny as _, |h| h + 1.0);

            if x == nx && y == ny {
                break;
            }

            x = nx;
            y = ny;
        }
    }

    let mut lake_origins = vec![];

    for x in range..(width-range) {
        for y in range..(height-range) {
            if (x, y) == next_target(map, range, x, y) && flow_map[(x, y)] > 0.0 {
                lake_origins.push((x, y, flow_map[(x, y)]));
            }
        }
    }

    println!("{} lake origins", lake_origins.len());

    let lakes = lake_map(map, &lake_origins, 10.0, range);
    let (_, max) = flow_map.minmax();

    for x in 0..width {
        for y in 0..height {
            if lakes[(x, y)] > 0.0 {
                flow_map[(x, y)] = lakes[(x, y)] * max;
            }
        }
    }

    flow_map
}

#[derive(PartialEq, Debug)]
struct Point{x: usize, y: usize, z: f32}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.z.partial_cmp(&self.z)
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn lake_map(map: &Map, lake_origins: &[(usize, usize, f32)], ocean: f32, range: usize) -> Map {
    let width = map.width();
    let height = map.height();

    let mut lake_map = Map::new(width, height);

    let point = |x, y| Point{x, y, z: map[(x, y)]};

    for origin in lake_origins.iter() {
        let mut pq = BinaryHeap::new();
        let enq = |pq: &mut BinaryHeap<Point>, lake_map: &mut Map, x, y| {
            if lake_map[(x, y)] == 0.0 {
                pq.push(point(x, y));
                lake_map[(x, y)] = -1.0;
            }
            lake_map[(x, y)] > 0.0
        };

        enq(&mut pq, &mut lake_map, origin.0, origin.1);

        while let Some(point) = pq.pop() {
            let Point {x, y, z} = point;
            lake_map[(x, y)] = -2.0;

            if z < ocean || x < range || x >= width-range || y < range || y >= height-range {
                break;
            }

            let (nx, ny) = next_target(map, range, x, y);

            if enq(&mut pq, &mut lake_map, x-1, y) ||
               enq(&mut pq, &mut lake_map, x+1, y) ||
               enq(&mut pq, &mut lake_map, x, y-1) ||
               enq(&mut pq, &mut lake_map, x, y+1) ||
               enq(&mut pq, &mut lake_map, nx, ny)
            {
                    break;
            }

            draw_line(&mut lake_map, x as _, y as _, nx as _, ny as _, |_| -2.0);
        }

        lake_map.map(|h| {
            match h {
                h if h < -1.0 => 1.0,
                h if h <  0.0 => 0.0,
                h => h,
            }
        });
    }

    lake_map
}

fn next_target(map: &Map, range: usize, x: usize, y: usize) -> (usize, usize) {
    let (mut nx, mut ny) = (x, y);
    let mut lowest = std::f32::MAX;

    for px in (x-range)..=(x+range) {
        for py in (y-range)..=(y+range) {
            if (px-x)*(px-x) + (py-y)*(py-y) > range*range {
                continue;
            }

            let height = map[(px, py)];
            if height < lowest {
                lowest = height;
                nx = px;
                ny = py;
            }
        }
    }

    (nx, ny)
}

fn draw_line(map: &mut Map, x1: isize, y1: isize, x2: isize, y2: isize, fun: fn(f32) -> f32) {
    let sign = |x| if x > 0 { 1 } else { -1 };

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = sign(x2 - x1);
    let sy = sign(y2 - y1);

    let (dx, dy, swap) = if dy <= dx {
        (dx, dy, false)
    } else {
        (dy, dx, true)
    };

    let mut d = 2 * dy - dx;

    let (mut x, mut y) = (x1, y1);

    for _ in 0..dx {
        map[(x as _, y as _)] = fun(map[(x as _, y as _)]);

        while d >= 0 {
            if swap {
                x += sx;
            } else {
                y += sy;
            }
            d -= 2 * dx;
        }

        if swap {
            y += sy;
        } else {
            x += sx;
        }
        d += 2 * dy;
    }

    map[(x as _, y as _)] = fun(map[(x as _, y as _)]);
}

