use crate::{map::*, flow::*};

pub fn create_flow_map(map: &Map, range: usize) -> Map {
    let width = map.width();
    let height = map.height();
    let mut flow_map = Map::new(width, height);

    let circle = circle(range);

    let mut targets = vec![vec![0isize; height]; width];
    let mut volume = vec![vec![1.0f32; height]; width];

    for x in range..(width-range) {
        for y in range..(height-range) {
            let (nx, ny) = next_target(map, x, y, &circle);
            if (nx, ny) != (x, y) {
                targets[nx][ny] += 1;
            }
        }
    }

    for x in range..(width-range) {
        for y in range..(height-range) {

            let (mut x, mut y) = (x, y);
            if targets[x][y] == 0 {
                loop {
                    targets[x][y] = -1;
                    if x < range || x >= width-range || y < range || y >= height-range {
                        break;
                    }

                    let (nx, ny) = next_target(map, x, y, &circle);
                    if (nx, ny) == (x, y) {
                        break;
                    }

                    draw_line(&mut flow_map, x as _, y as _, nx as _, ny as _, &|h| h + volume[x][y]);
                    volume[nx][ny] += volume[x][y];
                    targets[nx][ny] -= 1;

                    if targets[nx][ny] == 0 {
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

#[test]
fn points_are_correctly_sorted() {
    let mut points = vec![Point{ x: 0, y: 0, z: 1.0 }, Point{ x: 4, y: 6, z: 7.5 }];
    points.sort();
    assert!(points[0].z > points[1].z);
}

