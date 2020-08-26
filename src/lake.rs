use std::collections::*;
use std::cmp::*;
use crate::{map::*, flow::*};

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

pub fn lake_map(map: &Map, lake_origins: &[(usize, usize, f32)], ocean: f32, range: usize) -> Map {
    let width = map.width();
    let height = map.height();

    let mut lake_origins: Vec<_> = lake_origins.iter().map(|&(x, y, z)| Point{ x, y, z }).collect();
    lake_origins.sort();

    let mut lake_map = Map::new(width, height);

    let (min_terrain, _) = map.minmax();

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

    adjust_depth(map, &mut lake_map);

    lake_map
}

fn adjust_depth(terrain: &Map, lakes: &mut Map) {
    let width = terrain.width();
    let height = terrain.height();

    let mut groups = vec![vec![0usize; height]; width];
    let mut extrema = vec![(0.0, 0.0)];
    let mut area = vec![0usize];

    for x in 0..width {
        for y in 0..height {
            if groups[x][y] == 0 {

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
            let hfac = (altitude - min) / (max - min);

            if area[group] > 5 {
                lakes[(x, y)] = (1.0 - hfac).powf(1.2);
            } else {
                lakes[(x, y)] = 0.0;
            }
        }
    }
}

