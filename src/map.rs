use crate::{geometry::*, obj::*};
use rayon::prelude::*;
use std::cmp::*;
use std::io::{Error, ErrorKind, Result, Write};
use std::ops::*;

/// Generic matrix type with a fixed width and heigth that holds floats.
///
/// It can be used to store heightmaps, wetness or other kinds of maps of a terrain.
#[derive(Clone)]
pub struct Map {
    width: usize,
    height: usize,
    buffer: Vec<f32>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0.0; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// maps each entry of the map to a new value
    pub fn map<F>(&mut self, fun: F)
    where
        F: Send + Sync + Fn(f32) -> f32,
    {
        self.map_coords(|_, _, h| fun(h))
    }

    /// maps each entry of the map, together with its coordinates to a new value
    pub fn map_coords<F>(&mut self, fun: F)
    where
        F: Send + Sync + Fn(usize, usize, f32) -> f32,
    {
        self.buffer
            .par_chunks_mut(self.width)
            .enumerate()
            .for_each(|(y, chunk)| {
                for (x, h) in chunk.iter_mut().enumerate() {
                    *h = fun(x, y, *h);
                }
            });
    }

    /// Returns the minimum and maximum value of the map
    pub fn minmax(&self) -> (f32, f32) {
        let mut min = std::f32::MAX;
        let mut max = std::f32::MIN;
        for &h in self.buffer.iter() {
            min = min.min(h);
            max = max.max(h);
        }
        (min, max)
    }

    #[inline(always)]
    pub fn flatten_xy(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }
}

impl Map {
    /// exports the heightmap as obj mesh
    #[allow(unused)]
    pub fn export_mesh<W: Write>(&self, obj: &mut ObjWriter<W>) -> Result<()> {
        let vertex = |x, y| Vector::new(x as _, y as _, self[(x, y)]);

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

    /// exports the heightmap as cube mesh
    #[allow(unused)]
    pub fn export_cube_mesh<W: Write>(&self, obj: &mut ObjWriter<W>) -> Result<()> {
        let vertex = |x, y, h| Vector::new(x as _, y as _, h);
        let low = |v: Vector| Vector::new(v.x, v.y, -100.0);

        for x in 0..self.width() - 1 {
            for y in 0..self.height() - 1 {
                // top triangles

                let h = self[(x, y)];
                let a = vertex(x, y, h);
                let b = vertex(x + 1, y, h);
                let c = vertex(x + 1, y + 1, h);
                let d = vertex(x, y + 1, h);

                // top face
                obj.triangle(&Triangle([a, b, c]))?;
                obj.triangle(&Triangle([a, c, d]))?;

                let mut side_face = |a, b| {
                    obj.triangle(&Triangle([low(a), low(b), b]))?;
                    obj.triangle(&Triangle([low(a), b, a]))
                };

                side_face(a, b)?;
                side_face(b, c)?;
                side_face(c, d)?;
                side_face(d, a)?;
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

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.buffer[self.flatten_xy(x, y)]
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let idx = self.flatten_xy(x, y);
        &mut self.buffer[idx]
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
