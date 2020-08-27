use std::collections::*;
use crate::map::*;
use crate::lake::Point;

pub fn create_heightmap(rivers: &Map, lakes: &Map) -> Map {
    let (width, height) = (rivers.width(), rivers.height());
    let mut terrain = Map::new(width, height);

    let mut q = BinaryHeap::new();
    for x in 0..width {
        for y in 0..height {
            let z = lakes[(x, y)];
            if z > 0.0 {
                q.push(Point{ x, y, z });
            }
        }
    }

    while let Some(point) = q.pop() {
        let Point{x, y, z} = point;
        if terrain[(x, y)] > 0.0 {
            continue;
        }

        terrain[(x, y)] = z;

        if x == 0 || x == width-1 || y == 0 || y == height-1 {
            continue;
        }

        for dx in -1..=1 {
            for dy in -1..=1 {
                let nx = ((x as isize) + dx) as usize;
                let ny = ((y as isize) + dy) as usize;

                if terrain[(nx, ny)] > 0.0 {
                    continue;
                }

                let dz = 1.0 / (1.0 + rivers[(x, y)]);
                q.push(Point { x: nx, y: ny, z: z+dz });
            }
        }

    }

    terrain
}

