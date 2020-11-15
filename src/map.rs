use crate::{geometry::*, obj::*};
use std::cmp::*;
use std::io::{Error, ErrorKind, Result, Write};
use std::ops::*;

/// Generic matrix type with a fixed width and heigth that holds floats.
///
/// It can be used to store heightmaps, wetness or other kinds of maps of a terrain.
#[derive(Clone, PartialEq, Debug)]
pub struct Map(Vec<Vec<f32>>);

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self(vec![vec![0.0; height]; width])
    }

    pub fn width(&self) -> usize {
        self.0.len()
    }

    pub fn height(&self) -> usize {
        match self.0.last() {
            None => 0,
            Some(v) => v.len(),
        }
    }

    /// maps each entry of the map to a new value
    pub fn map<F>(&mut self, fun: F)
    where
        F: Fn(f32) -> f32,
    {
        for x in 0..self.width() {
            for y in 0..self.height() {
                self[(x, y)] = fun(self[(x, y)]);
            }
        }
    }

    /// applies a binary operation between this map and the other map
    pub fn binary_op<F: FnMut(f32, f32) -> f32>(&mut self, other: &Map, mut op: F) {
        let w = self.width().min(other.width());
        let h = self.height().min(other.height());

        for x in 0..w {
            for y in 0..h {
                self[(x, y)] = op(self[(x, y)], other[(x, y)]);
            }
        }
    }

    /// applies a binary operation to this map at a specified offset
    ///
    /// `same` and `other` don't need to have the same dimensions
    pub fn binary_op_at<F: FnMut(f32, f32) -> f32>(
        &mut self,
        other: &Map,
        mut op: F,
        from: (usize, usize),
        to: (usize, usize),
    ) {
        let (from_x, from_y) = from;
        let (to_x, to_y) = to;
        let to_x = to_x.min((from_x + other.width()).min(self.width()));
        let to_y = to_y.min((from_y + other.height()).min(self.height()));

        println!(
            "Applying binary op from ({}, {}) to ({}, {})",
            from_x, from_y, to_x, to_y
        );

        for x in from_x..to_x {
            for y in from_y..to_y {
                self[(x, y)] = op(self[(x, y)], other[(x - from_x, y - from_y)]);
            }
        }
    }

    /// Returns the minimum and maximum value of the map
    pub fn minmax(&self) -> (f32, f32) {
        let mut min = std::f32::MAX;
        let mut max = std::f32::MIN;
        for x in 0..self.width() {
            for y in 0..self.height() {
                min = min.min(self[(x, y)]);
                max = max.max(self[(x, y)]);
            }
        }
        (min, max)
    }

    pub fn gaussian_blur(&self, mask: &Self, radius: usize) -> Self {
        assert!(radius >= 1);
        let mut mat = vec![vec![0.0f32; radius]; radius];

        let frad2 = (radius * radius) as f32;
        for x in 0..radius {
            for y in 0..radius {
                let (fx, fy) = (x as f32, y as f32);
                mat[x][y] =
                    0.5 / std::f32::consts::PI / frad2 * (-(fx * fx + fy * fy) / 2.0 / frad2).exp();
            }
        }

        let mut map = Self::new(self.width(), self.height());
        let (w, h) = (self.width() as isize, self.height() as isize);

        for x in 0..self.width() {
            for y in 0..self.height() {
                if mask[(x, y)] <= 1e-5 {
                    continue;
                }
                let r = radius as isize - 1;
                let (x, y) = (x as isize, y as isize);
                for nx in (0.max(x - r))..(w.min(x + r)) {
                    for ny in (0.max(y - r))..(h.min(y + r)) {
                        let ex = (nx - x).abs() as usize;
                        let ey = (ny - y).abs() as usize;

                        map[(x as _, y as _)] += mat[ex][ey] * self[(nx as _, ny as _)];
                    }
                }
            }
        }

        map
    }
}

impl Map {
    /// exports the heightmap as obj mesh
    #[allow(unused)]
    pub fn export_mesh<W: Write>(&self, obj: &mut ObjWriter<W>) -> Result<()> {
        let vertex = |x, y| Vector3::new(x as _, y as _, self[(x, y)]);

        for x in 0..self.width() - 1 {
            for y in 0..self.height() - 1 {
                obj.triangle(&Triangle([
                    vertex(x, y),
                    vertex(x + 1, y),
                    vertex(x, y + 1),
                ]))?;
                obj.triangle(&Triangle([
                    vertex(x + 1, y),
                    vertex(x + 1, y + 1),
                    vertex(x, y + 1),
                ]))?;
            }
        }
        Ok(())
    }

    /// exports the heightmap as png image
    #[allow(unused)]
    pub fn export_image<W: Write>(&self, writer: W) -> Result<()> {
        let mut encoder = png::Encoder::new(writer, self.width() as _, self.height() as _);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;

        let mut buffer = vec![0u8; self.width() * self.height()];

        let (min, max) = self.minmax();

        for x in 0..self.width() {
            for y in 0..self.height() {
                let idx = y * self.width() + x;
                let val = self[(x, y)];
                let val01 = (val - min) / (max - min);
                buffer[idx] = (255.0 * val01) as _;
            }
        }

        writer
            .write_image_data(&buffer)
            .map_err(|_| Error::new(ErrorKind::Other, "PNG encoding error"))
    }
}

impl Index<(usize, usize)> for Map {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

/// Single point of the map, storing its height
#[derive(PartialEq, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
    pub z: f32,
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.z.partial_cmp(&self.z)
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
