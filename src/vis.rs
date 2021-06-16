use crate::map::*;
use png::*;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};

#[derive(Copy, Clone)]
struct Color([f32; 3]);

impl Color {
    fn mix(&self, other: &Self, f: f32) -> Self {
        let mut new = [0.0; 3];
        for i in 0..3 {
            new[i] = self.0[i] * (1.0 - f) + other.0[i] * f;
        }
        Self(new)
    }
    fn write_to_rgb_buf(self, buf: &mut [u8]) {
        for i in 0..3 {
            buf[i] = (self.0[i] * 255.0) as u8;
        }
    }
    fn from(rgb: [u8; 3]) -> Self {
        let mut new = [0.0; 3];
        for i in 0..3 {
            new[i] = rgb[i] as f32 / 255.0;
        }
        Self(new)
    }
}

pub fn visualize(map: &Map, ocean: f32, out: impl Write) -> Result<()> {
    let mut map = map.clone();
    map.map(|h| if h < ocean { ocean } else { h });
    let map = &map;

    let (_, peak) = map.minmax();

    let mut buffer = vec![0u8; 3 * map.width() * map.height()];

    let col_ocean = Color::from([77, 77, 140]);
    let col_default = Color::from([200; 3]);
    let col_ambient = Color([0.0; 3]);
    let col_lines = Color([0.0; 3]);

    // inclination angle of the sun
    // the light always comes from -x -y direction
    let angle = 0.2;

    let shade = shade_map(map, angle);

    for x in 0..map.width() {
        for y in 0..map.height() {
            let light = if map[(x, y)] < shade[(x, y)] {
                0.0
            } else {
                let diff = if x > 0 && y > 0 {
                    map[(x, y)] - map[(x - 1, y - 1)]
                } else {
                    0.0
                };

                let p = (2.0f32).sqrt();
                let l = (angle / p).atan();
                let r = (diff / p).atan();

                let target = std::f32::consts::PI - l - r;
                target.sin().powf(0.5)
            };

            // banding
            let light = light - (light % 0.125);

            let shade = 0.5 * (1.0 - light);
            let col = if map[(x, y)] < ocean + 0.001 {
                col_ocean
            } else {
                col_default
            };

            let col = col.mix(&col_ambient, shade);
            let col = col.mix(&col_lines, height_line(map[(x, y)]));
            col.write_to_rgb_buf(&mut buffer[3 * map.flatten_xy(x, y)..]);
        }
    }

    let mut encoder = Encoder::new(out, map.width() as _, map.height() as _);
    encoder.set_color(ColorType::RGB);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer
        .write_image_data(&buffer)
        .map_err(|_| Error::new(ErrorKind::Other, "PNG encoding error"))
}

fn height_line(h: f32) -> f32 {
    let h = (h + 5.0) % 10.0;
    if h < 0.8 {
        0.3
    } else {
        0.0
    }
}

fn shade_map(map: &Map, angle: f32) -> Map {
    let mut shade = Map::new(map.width(), map.height());

    for x in 0..map.width() {
        shade[(x, 0)] = map[(x, 0)];
    }
    for y in 0..map.height() {
        shade[(0, y)] = map[(0, y)];
    }

    for y in 1..map.height() {
        for x in 1..map.height() {
            shade[(x, y)] = map[(x, y)].max(shade[(x - 1, y - 1)] - angle);
        }
    }

    shade
}
