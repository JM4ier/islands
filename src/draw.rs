use crate::map::Map;

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
    fun: impl Fn(f32) -> f32,
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
