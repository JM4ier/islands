use crate::{map::*, flow::*};

pub fn create_flow_map(map: &Map, sources: usize, range: usize) -> Map {
    let width = map.width();
    let height = map.height();
    let mut flow_map = Map::new(width, height);

    let random_position = || (rand::random::<usize>() % width, rand::random::<usize>() % height);

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

    flow_map
}

