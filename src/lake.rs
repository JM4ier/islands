use crate::map::*;
use std::collections::*;

pub fn lake_map(
    map: &Map,
    rivers: &Map,
    cell_targets: &Vec<Vec<(usize, usize)>>,
    ocean: f32,
    range: usize,
) -> Map {
    let width = map.width();
    let height = map.height();

    let mut lake_origins = Vec::new();
    let mut lakes = Map::new(width, height);

    let at_border = |x, y| x < range || x >= width - range || y < range || y >= height - range;

    for x in 0..width {
        for y in 0..height {
            let xy = (x, y);
            let z = map[xy];

            if z < ocean || at_border(x, y) {
                lakes[xy] = 1.0;
            } else if xy == cell_targets[x][y] && rivers[xy] > 100.0 {
                lake_origins.push(Point { x, y, z });
            }
        }
    }

    for origin in lake_origins.into_iter() {
        let mut queue = BinaryHeap::new();
        let mut points = Vec::new();

        macro_rules! enq {
            ($x:expr, $y:expr) => {{
                let xy = ($x, $y);
                if lakes[xy] == 0.0 {
                    queue.push(Point {
                        x: xy.0,
                        y: xy.1,
                        z: map[xy],
                    });
                    lakes[xy] = -1.0;
                    points.push(xy);
                }
                lakes[xy] > 0.0
            }};
        }

        if enq!(origin.x, origin.y) {
            continue;
        }

        while let Some(Point { x, y, z }) = queue.pop() {
            if z < ocean || at_border(x, y) {
                break;
            }

            let (nx, ny) = cell_targets[x][y];
            if enq!(x - 1, y) || enq!(x + 1, y) || enq!(x, y - 1) || enq!(x, y + 1) || enq!(nx, ny)
            {
                break;
            }
        }

        for point in points.into_iter() {
            lakes[point] = 1.0;
        }
    }

    let mut groups = vec![vec![0usize; height]; width];
    let mut max = vec![0.0];
    let mut area = vec![0];

    for x in 0..width {
        for y in 0..height {
            if groups[x][y] == 0 && lakes[(x, y)] > 0.0 {
                let group = max.len();
                max.push(ocean);
                area.push(0);

                let mut queue = Vec::new();

                macro_rules! enq {
                    ($x: expr, $y: expr) => {
                        let (x, y) = ($x, $y);
                        if groups[x][y] == 0 && lakes[(x, y)] > 0.0 {
                            queue.push((x, y));
                            groups[x][y] = group;
                        }
                    };
                }

                enq!(x, y);

                while let Some((x, y)) = queue.pop() {
                    area[group] += 1;
                    max[group] = max[group].max(map[(x, y)]);

                    let next = |val, limit| val + (val + 1 < limit) as usize;
                    let prev = |val| val - (val > 0) as usize;

                    enq!(prev(x), y);
                    enq!(next(x, width), y);
                    enq!(x, prev(y));
                    enq!(x, next(y, height));
                }
            }

            let group = groups[x][y];
            if area[group] > 10 {
                lakes[(x, y)] = max[group];
            } else {
                lakes[(x, y)] = 0.0;
            }
        }
    }

    lakes
}
