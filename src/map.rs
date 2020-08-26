use std::ops::*;
use std::io::{Result, Error, ErrorKind, Write};
use crate::{geometry::*, obj::*};

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

    pub fn map(&mut self, fun: fn(f32) -> f32) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                self[(x, y)] = fun(self[(x, y)]);
            }
        }
    }

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

impl Map{
    /// exports the heightmap as obj mesh
    pub fn export_mesh<W: Write>(&self, obj: &mut ObjWriter<W>) -> Result<()> {
        let vertex = |x, y| Vector::new(x as _, y as _, self[(x, y)]);

        for x in 0..self.width() - 1 {
            for y in 0..self.height() - 1 {
                obj.triangle(&Triangle([vertex(x, y), vertex(x+1, y), vertex(x, y+1)]))?;
                obj.triangle(&Triangle([vertex(x+1, y), vertex(x+1, y+1), vertex(x, y+1)]))?;
            }
        }
        Ok(())
    }

    /// exports the heightmap as png image
    pub fn export_image<W: Write>(&self, writer: W) -> Result<()>{
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

        println!("min: {}, max: {}", min, max);

        writer.write_image_data(&buffer)
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

