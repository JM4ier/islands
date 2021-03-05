use crate::{geometry::*, obj::*};
use std::cmp::*;
use std::io::{Error, ErrorKind, Result, Write};
use std::ops::*;

/// Generic matrix type with a fixed width and heigth that holds floats.
///
/// It can be used to store heightmaps, wetness or other kinds of maps of a terrain.
#[derive(Clone)]
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
        F: Send + Sync + Fn(f32) -> f32,
    {
        self.map_coords(|_, _, h| fun(h))
    }

    /// maps each entry of the map, together with its coordinates to a new value
    pub fn map_coords<F>(&mut self, fun: F)
    where
        F: Send + Sync + Fn(usize, usize, f32) -> f32,
    {
        let fun = &fun;
        let width = self.width();

        let threads = 16;
        let chunk_size = width / threads;

        crossbeam::scope(|s| {
            for (cidx, chunk) in self.0.chunks_mut(chunk_size).enumerate() {
                s.spawn(move |_| {
                    for (coff, vec) in chunk.iter_mut().enumerate() {
                        let x = chunk_size * cidx + coff;
                        for y in 0..vec.len() {
                            vec[y] = fun(x, y, vec[y]);
                        }
                    }
                });
            }
        })
        .expect("Child thread pannicked.");
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
