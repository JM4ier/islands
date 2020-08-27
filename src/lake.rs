use std::collections::*;
use std::cmp::*;
use crate::{map::*, flow::*};

#[derive(PartialEq, Debug)]
pub struct Point{
    pub x: usize, 
    pub y: usize, 
    pub z: f32
}

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

pub fn lake_map(map: &Map, flow_map: &Map, ocean: f32, range: usize) -> Map {
    let width = map.width();
    let height = map.height();

    // find lake origins where water cant flow anywhere
    let mut lake_origins = vec![];
    for x in range..(width-range) {
        for y in range..(height-range) {
            if (x, y) == next_target(map, range, x, y) && flow_map[(x, y)] > 100.0 {
                lake_origins.push(Point{x, y, z: flow_map[(x, y)]});
            }
        }
    }
    lake_origins.sort();

    let mut lake_map = Map::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let z = map[(x, y)];
            if z < ocean {
                lake_map[(x, y)] = 1.0;
            }
        }
    }

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

        enq(&mut pq, &mut lake_map, origin.x, origin.y);

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

        }

        lake_map.map(|h| {
            match h {
                h if h < -1.0 => 1.0,
                h if h <  0.0 => 0.0,
                h => h,
            }
        });
    }

    let _gfx_depth_fun = |min: f32, max: f32, height: f32| {
        (1.0 - (height-min) / (max-min)).powf(1.2)
    };
    let water_level_fun = |_, max, _| max;
    adjust_depth(map, &mut lake_map, water_level_fun);

    lake_map
}

fn adjust_depth<F>(terrain: &Map, lakes: &mut Map, depth_fun: F) 
    where F: Fn(f32, f32, f32) -> f32
{
    let width = terrain.width();
    let height = terrain.height();

    let mut groups = vec![vec![0usize; height]; width];
    let mut extrema = vec![(0.0, 0.0)];
    let mut area = vec![0usize];

    for x in 0..width {
        for y in 0..height {
            if groups[x][y] == 0 && lakes[(x, y)] > 0.0 {

                let group = extrema.len();
                extrema.push((std::f32::MAX, std::f32::MIN));
                area.push(0);

                let mut q = Vec::new();
                q.push((x, y)); groups[x][y] = group;

                while let Some((x, y)) = q.pop() {

                    area[group] += 1;
                    let altitude = terrain[(x, y)];
                    let (min, max) = extrema[group];
                    extrema[group] = (min.min(altitude), max.max(altitude));

                    if x > 0 && groups[x-1][y] == 0 && lakes[(x-1, y)] > 0.0 {
                        q.push((x-1, y));
                        groups[x-1][y] = group;
                    }
                    if x < width-1 && groups[x+1][y] == 0 && lakes[(x+1, y)] > 0.0 {
                        q.push((x+1, y));
                        groups[x+1][y] = group;
                    }
                    if y > 0 && groups[x][y-1] == 0 && lakes[(x, y-1)] > 0.0 {
                        q.push((x, y-1));
                        groups[x][y-1] = group;
                    }
                    if y < height-1 && groups[x][y+1] == 0 && lakes[(x, y+1)] > 0.0 {
                        q.push((x, y+1));
                        groups[x][y+1] = group;
                    }
                }
            }

            let group = groups[x][y];
            let (min, max) = extrema[group];
            let altitude = terrain[(x, y)];

            if area[group] > 5 {
                lakes[(x, y)] = depth_fun(min, max, altitude);
            } else {
                lakes[(x, y)] = 0.0;
            }
        }
    }
}

