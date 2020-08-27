use crate::map::*;

/// returns a simplex based height map with a central raise and edges scaled down
pub fn simplex_map(width: usize, height: usize, enable_edge_scaling: bool) -> Map {
    let iter = 4;
    let persistence = 0.3;
    let scale = 5.02 / width as f32;

    let simplex = fuss::Simplex::new();
    let mut map = Map::new(width, height);

    let scaling = |h: f32| (h + 1.06) * 16.0;

    let edge_scaling = |x: usize, y: usize| {
        let pow = 8.5;

        let x = x as f32;
        let y = y as f32;
        let width = width as f32;
        let height = height as f32;

        let dx = x - width / 2.0;
        let dy = y - height / 2.0;

        let dist = dx.abs().powf(pow) + dy.abs().powf(pow);
        let midp = (0.5 * width.min(height) as f32).powf(pow);

        let fade0 = midp;
        let fade1 = (0.8f32).powf(pow) * midp;

        let v: f32 = (dist - fade0) / (fade1 - fade0);
        let v = v.max(0.0).min(1.0);

        (((v - 0.5) * std::f32::consts::PI).sin() + 1.0) * 0.5
    };

    let mountain = |x: usize, y: usize| {
        let x = x as f32 - (width as f32) * 0.5;
        let y = y as f32 - (height as f32) * 0.5;

        let s = 4.5 / (width as f32);

        let x = x * s;
        let y = y * s;

        6.0 / (x*x + y*y + 1.0)
    };

    for x in 0..width {
        for y in 0..height {
            let height = simplex.sum_octave_2d(iter, x as _, y as _, persistence, scale);
            map[(x, y)] = scaling(height) * mountain(x, y);
            if enable_edge_scaling {
                map[(x, y)] *= edge_scaling(x, y);
            }
        }
    }

    map
}

