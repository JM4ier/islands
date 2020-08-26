use std::collections::*;
use std::cmp::*;
use crate::{map::*, flow::*, lake::*};

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
            if (x, y) == next_target(map, range, x, y) && flow_map[(x, y)] > 100.0 {
                lake_origins.push((x, y, flow_map[(x, y)]));
            }
        }
    }

    println!("{} lake origins", lake_origins.len());

    let lakes = lake_map(map, &lake_origins, 20.0, range);
    let (_, max) = flow_map.minmax();

    for x in 0..width {
        for y in 0..height {
            let xy = (x,y);
            flow_map[xy] = flow_map[xy].max(lakes[xy] * max);
        }
    }

    flow_map
}

