use crate::{map::*, flow::*};

pub fn create_flow_map(map: &Map, range: usize) -> Map {
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

    let mut volume = vec![vec![1.0f32; height]; width];
    let circle = circle(range);

    for point in points.into_iter() {
        let Point{x, y, z: _} = point;

        if x < range || x >= width-range || y < range || y >= height-range {
            break;
        }

        let (nx, ny) = next_target(map, x, y, &circle);

        draw_line(&mut flow_map, x as _, y as _, nx as _, ny as _, &|h| h + volume[x][y]);

        volume[nx][ny] += volume[x][y];
    }

    flow_map
}

