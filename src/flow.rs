use crate::map::*;

pub fn circle(range: usize) -> Vec<(isize, isize)> {
    let mut positions = Vec::with_capacity(range*range);

    let range = range as isize;

    for x in (-range)..=(range) {
        for y in (-range)..=(range) {
            if x*x + y*y <= range*range {
                positions.push((x, y));
            }
        }
    }

    positions
}

#[inline]
pub fn next_target(map: &Map,  x: usize, y: usize, positions: &[(isize, isize)]) -> (usize, usize) {
    let (mut nx, mut ny) = (x, y);
    let mut lowest = std::f32::MAX;

    let (x, y) = (x as isize, y as isize);

    for (dx, dy) in positions.iter() {
        let px = (x + dx) as usize;
        let py = (y + dy) as usize;

        let height = map[(px, py)];
        if height < lowest {
            lowest = height;
            nx = px;
            ny = py;
        }
    }

    (nx, ny)
}

#[inline]
pub fn draw_line(map: &mut Map, x1: isize, y1: isize, x2: isize, y2: isize, fun: &dyn Fn(f32) -> f32) {
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

