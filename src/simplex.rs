use crate::map::Map;

/// returns a simplex based height map with a central raise and edges scaled down
pub fn simplex_map(width: usize, height: usize) -> Map {
    // simplex noise parameters
    let iter = 4;
    let persistence = 0.3;
    let scale = 5.02 / width as f32;

    // power to give edge scaling a shape
    // a value of 2.0 would give the edge the shape of a circle
    let edge_pow = 9.0;

    let simplex = fuss::Simplex::new();
    let mut map = Map::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let height_scaling = {
                let (x, y, width, height) = (x as f32, y as f32, width as f32, height as f32);

                let (dx, dy) = (x - width * 0.5, y - height * 0.5);

                let edge_scaling = {
                    let dist = dx.abs().powf(edge_pow) + dy.abs().powf(edge_pow);

                    let fade0 = (0.5 * width.min(height)).powf(edge_pow);
                    let fade1 = (0.8f32).powf(edge_pow) * fade0;

                    let v = (dist - fade0) / (fade1 - fade0);
                    let v = v.max(0.0).min(1.0);

                    // sin curve mapping 0,1 to 0,1 to have a smooth gradient
                    (((v - 0.5) * std::f32::consts::PI).sin() + 1.0) * 0.5
                };

                let mountain_scaling = {
                    let scale = 4.5 / width;
                    let (dx, dy) = (dx * scale, dy * scale);
                    6.0 / (dx * dx + dy * dy + 1.0)
                };

                edge_scaling * mountain_scaling
            };

            let height = simplex.sum_octave_2d(iter, x as _, y as _, persistence, scale);
            let scaled_height = height_scaling * (16.0 * height + 17.0);
            map[(x, y)] = scaled_height;
        }
    }

    map
}
