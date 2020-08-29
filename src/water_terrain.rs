use std::collections::*;
use crate::map::*;

/// creates a height map based on the river and lake `Map`'s.
pub fn create_heightmap(rivers: &Map, lakes: &Map) -> Map {
    let (width, height) = (rivers.width(), rivers.height());
    let mut terrain = Map::new(width, height);

    // points sorted by lowest z coordinate first
    //
    // adding lakes points first, as the terrain is generated lowest
    // first beginning from the lakes
    let mut queue = BinaryHeap::new();
    for x in 0..width {
        for y in 0..height {
            let z = lakes[(x, y)];
            if z > 0.0 {
                queue.push(Point{ x, y, z });
                terrain[(x, y)] = z;
            }
        }
    }

    let mut done = vec![vec![false; height]; width];

    while let Some(point) = queue.pop() {
        let Point{x, y, z} = point;

        if done[x][y] {
            // terrain has already been generated here earlier, no need to do it again
            continue;
        }

        done[x][y] = true;

        if x == 0 || x == width-1 || y == 0 || y == height-1 {
            // neighbors are not searched for border positions, as this should all be 
            // ocean anyways
            continue;
        }

        for dx in -1..=1 {
            for dy in -1..=1 {

                // finding neighbor x,y
                let nx = ((x as isize) + dx) as usize;
                let ny = ((y as isize) + dy) as usize;

                if done[nx][ny] {
                    // if already generated, don't bother
                    continue;
                }

                // steeper if there is not much water flowing through
                let dz = 1.0 / (0.5 + rivers[(x, y)]);
                let nz = z + dz;

                if terrain[(nx, ny)] != 0.0 && terrain[(nx, ny)] < nz {
                    continue;
                }
                // add neighboring point to queue to process further in the correct order
                queue.push(Point { x: nx, y: ny, z: nz });
                terrain[(nx, ny)] = nz;
            }
        }
    }

    terrain
}

#[test]
fn test_point_sorting_order() {
    let mut queue = BinaryHeap::new();
    queue.push(Point{ x: 0, y: 0, z: 1.0});
    queue.push(Point{ x: 0, y: 0, z: 2.0});

    let first = queue.pop().unwrap();
    let second = queue.pop().unwrap();

    assert!(first.z < second.z);
}

