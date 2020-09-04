use crate::{flow::*, map::*};

/// Calculates the flow map of the terrain given in `map`.
///
/// It does so by distributing water on each cell of the map and drawing
/// all of the waters path onto the flow_map, which it returns.
///
pub fn create_flow_map(map: &Map, cell_targets: &Vec<Vec<(usize, usize)>>) -> Map {
    let width = map.width();
    let height = map.height();
    let mut flow_map = Map::new(width, height);

    // how many other cells 'target' this cell
    let mut targets = vec![vec![0i32; height]; width];
    // how much volume flows through each cell
    let mut volume = vec![vec![1.0f32; height]; width];

    // finding how many times each cell is targeted
    for x in 0..width {
        for y in 0..height {
            let (nx, ny) = cell_targets[x][y];
            if (nx, ny) != (x, y) {
                targets[nx][ny] += 1;
            }
        }
    }

    for x in 0..width {
        for y in 0..height {
            if targets[x][y] == 0 {
                let (mut x, mut y) = (x, y);
                loop {
                    // prevent processing this cell twice
                    targets[x][y] = -1;

                    let (nx, ny) = cell_targets[x][y];

                    draw_line(&mut flow_map, x as _, y as _, nx as _, ny as _, &|h| {
                        h + volume[x][y]
                    });
                    volume[nx][ny] += volume[x][y];
                    targets[nx][ny] -= 1;

                    if targets[nx][ny] == 0 {
                        // if all the cells which target (nx, ny) have been processed, (nx, ny) can
                        // now be processed
                        x = nx;
                        y = ny;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    flow_map
}
