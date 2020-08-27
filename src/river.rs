use crate::{map::*, flow::*, lake::Point};

pub fn create_flow_map(map: &Map, sources: usize, range: usize) -> Map {
    let width = map.width();
    let height = map.height();
    let mut flow_map = Map::new(width, height);

    let mut points = Vec::with_capacity(width * height);

    for x in 0..width {
        for y in 0..height {
            points.push(Point{ x, y, z: map[(x, y)]});
        }
    }

    points.sort(); // sort highest first

    let mut target_nums = vec![vec![1usize; height]; width];

    for point in points.into_iter() {
        let Point{x, y, z} = point;

        if x < range || x >= width-range || y < range || y >= height-range {
            break;
        }

        let (nx, ny) = next_target(map, range, x, y);

        draw_line(&mut flow_map, x as _, y as _, nx as _, ny as _, &|h| h + target_nums[x][y] as f32);

        target_nums[nx][ny] += target_nums[x][y];
    }

    flow_map
}

