use crate::map::*;

/// returns relative coordinates of adjacent fields within a certain range
fn circle(range: usize) -> Vec<(isize, isize)> {
    let mut positions = Vec::with_capacity(range * range);

    let range = range as isize;

    for x in (-range)..=(range) {
        for y in (-range)..=(range) {
            if x * x + y * y <= range * range {
                positions.push((x, y));
            }
        }
    }

    positions
}

/// finds the flow target for each position on the map and returns them in a `Vec`
pub fn find_targets(map: &Map, range: usize) -> Vec<Vec<(usize, usize)>> {
    let (width, height) = (map.width(), map.height());
    let mut targets = vec![vec![(0, 0); height]; width];
    let at_border = |x, y| x < range || x >= width - range || y < range || y >= height - range;
    let circle = circle(range);

    for x in 0..width {
        for y in 0..height {
            if at_border(x, y) {
                targets[x][y] = (x, y);
            } else {
                targets[x][y] = next_target(map, x, y, &circle);
            }
        }
    }
    targets
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
) -> (usize, usize) {
    let (mut nx, mut ny) = (x, y); // next x,y
    let mut lowest = std::f32::MAX; // lowest z coordinate of neighbors

    let (x, y) = (x as isize, y as isize);

    for (dx, dy) in neighbor_positions.iter() {
        // absolute x,y coordinate of neighbors
        let px = (x + dx) as usize;
        let py = (y + dy) as usize;

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

/// Draws a line on the `map` from `(x1, y1)` to `(x2, y2)`.
///
/// The `fun` parameter is given to implement custom strokes, i.e. how much the values are
/// increased given a base value. To simply draw a line with no respect to the previous values of
/// the map, something like this can be passed as `fun`: `&|_| 1.0`.
///
/// The drawing is done using a generalized version of
/// [Bresenhams' Line Algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm).
#[inline]
pub fn draw_line(
    map: &mut Map,
    x1: isize,
    y1: isize,
    x2: isize,
    y2: isize,
    fun: &dyn Fn(f32) -> f32,
) {
    let sign = |x| if x > 0 { 1 } else { -1 };

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = sign(x2 - x1);
    let sy = sign(y2 - y1);

    let (dx, dy, swap) = if dy <= dx {
        (dx, dy, false)
    } else {
        (dy, dx, true)
    };

    let mut d = 2 * dy - dx;

    let (mut x, mut y) = (x1, y1);

    for _ in 0..dx {
        map[(x as _, y as _)] = fun(map[(x as _, y as _)]);

        while d >= 0 {
            if swap {
                x += sx;
            } else {
                y += sy;
            }
            d -= 2 * dx;
        }

        if swap {
            y += sy;
        } else {
            x += sx;
        }
        d += 2 * dy;
    }

    map[(x as _, y as _)] = fun(map[(x as _, y as _)]);
}
