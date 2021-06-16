use crate::map::*;
use rayon::prelude::*;

/// finds the flow target for each position on the map and returns them in a `Vec`
pub fn find_targets(map: &Map, range: usize) -> Vec<Vec<(usize, usize)>> {
    assert!(range >= 1);
    assert!(range < map.width());
    assert!(range < map.height());

    let (width, height) = (map.width(), map.height());
    let mut targets = vec![vec![(0, 0); height]; width];

    let circle = |x, y| x * x + y * y < (range * range) as isize;
    let within_range = |x, y, (ox, oy)| circle(x as isize - ox as isize, y as isize - oy as isize);

    // points to check that neighbors didn't check with 4 different cases
    let rest_points = [
        points(range, &circle),
        points(range, &|x, y| circle(x, y) && !circle(x + 1, y)),
        points(range, &|x, y| circle(x, y) && !circle(x, y + 1)),
        points(range, &|x, y| {
            circle(x, y) && !circle(x + 1, y) && !circle(x, y + 1)
        }),
    ];

    // finding targets of the upper and lower border positions
    for x in 0..width {
        for y in (0..range).chain((height - range)..height) {
            targets[x][y] = next_target(map, x, y, &rest_points[0], true);
        }
    }

    let threads = if width < 512 { 1 } else { num_cpus::get() };
    let threads = 1;
    let chunk_size = width / threads;

    targets
        .par_iter_mut()
        .chunks(chunk_size)
        .enumerate()
        .for_each(|(chunk_id, mut chunk)| {
            let chunk_offset = chunk_id * chunk_size;

            // calculate the border areas naively
            for x in 0..range {
                for y in 0..height {
                    chunk[x][y] = next_target(map, x + chunk_offset, y, &rest_points[0], true);
                }
            }

            let x_end;

            if chunk_id == threads - 1 {
                for x in chunk.len() - range..chunk.len() {
                    for y in 0..height {
                        chunk[x][y] = next_target(map, x + chunk_offset, y, &rest_points[0], true);
                    }
                }
                x_end = chunk.len() - range;
            } else {
                x_end = chunk.len();
            }

            for local_x in range..x_end {
                let x = local_x + chunk_offset;

                for y in range..(height - range) {
                    let mut lowest = f32::MAX;
                    let mut target = (x, y);
                    let mut rest_idx = 0;

                    // check if target of x neighbor can be used
                    let tx = chunk[local_x - 1][y];
                    if within_range(x, y, tx) {
                        rest_idx += 1;
                        lowest = map[tx];
                        target = tx;
                    }

                    // check if target of y neighbor can be used
                    let ty = chunk[local_x][y - 1];
                    if within_range(x, y, ty) {
                        rest_idx += 2;
                        let h = map[ty];
                        if h < lowest {
                            lowest = h;
                            target = ty;
                        }
                    }

                    // check if there is a better minimum in the points
                    // the previous neighbors didn't reach but this point does
                    let t = next_target(map, x, y, &rest_points[rest_idx], false);
                    let h = map[t];
                    if h < lowest {
                        target = t;
                    }

                    chunk[local_x][y] = target;
                }
            }
        });

    targets
}

/// Finds points within a range that satisfy a property
fn points(range: usize, inside: &dyn Fn(isize, isize) -> bool) -> Vec<(isize, isize)> {
    let mut positions = Vec::with_capacity(range * range);
    let range = range as isize;

    for x in (-range)..=(range) {
        for y in (-range)..=(range) {
            if inside(x, y) {
                positions.push((x, y));
            }
        }
    }

    positions
}

/// Finds the lowest neighbor around a point on the map using the relative coordinates of all
/// neighbors.
/// Relative coordinates of all neighbors can be found once using the [`circle`](./fn.circle.html) function.
#[inline]
fn next_target(
    map: &Map,
    x: usize,
    y: usize,
    neighbor_positions: &[(isize, isize)],
    check_coordinates: bool,
) -> (usize, usize) {
    let (mut nx, mut ny) = (x, y); // next x,y
    let mut lowest = f32::MAX; // lowest z coordinate of neighbors

    let (x, y) = (x as isize, y as isize);

    for (dx, dy) in neighbor_positions.iter() {
        // absolute x,y coordinate of neighbors
        let px = x + dx;
        let py = y + dy;

        if check_coordinates
            && (px < 0 || px >= map.width() as _ || py < 0 || py >= map.height() as _)
        {
            continue;
        }

        let (px, py) = (px as usize, py as usize);

        // find lowest neighbor
        let height = map[(px, py)];
        if height < lowest {
            lowest = height;
            nx = px;
            ny = py;
        }
    }

    (nx, ny)
}

/// Check if the efficient DP solution does find the local minimum of each point correctly
#[test]
fn check_dp_solution() {
    use crate::simplex::*;

    let size = 128;
    let map = simplex_map(size, size);
    let range = 6;
    let targets = find_targets(&map, range);
    let circle = |x, y| x * x + y * y < (range * range) as isize;
    let points = points(range, &circle);

    for x in 0..size {
        for y in 0..size {
            let target = next_target(&map, x, y, &points, true);

            // Check if found _a_ minimum, not the specific minimum the target function would have
            // found. If the height is the same, but the coordinates are different, there are
            // multiple valid solutions.
            assert_eq!(map[target], map[targets[x][y]]);
        }
    }
}
