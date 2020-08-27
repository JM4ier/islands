use crate::map::*;

pub fn next_target(map: &Map, range: usize, x: usize, y: usize) -> (usize, usize) {
    let (mut nx, mut ny) = (x, y);
    let mut lowest = std::f32::MAX;

    for px in (x-range)..=(x+range) {
        for py in (y-range)..=(y+range) {
            if (px-x)*(px-x) + (py-y)*(py-y) > range*range {
                continue;
            }

            let height = map[(px, py)];
            if height < lowest {
                lowest = height;
                nx = px;
                ny = py;
            }
        }
    }

    (nx, ny)
}

pub fn draw_line(map: &mut Map, x1: isize, y1: isize, x2: isize, y2: isize, fun: fn(f32) -> f32) {
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

