use crate::geometry::*;
use std::io::Write;

pub struct ObjWriter<'w, W> {
    writer: &'w mut W,
    vertex_count: usize,
}

impl<'w, W: Write> ObjWriter<'w, W> 
{
    pub fn new(writer: &'w mut W) -> std::io::Result<Self> {
        writer.write_all(b"# Terrain Export from _WorldGenTM_\n")?;
        Ok(Self {
            writer,
            vertex_count: 0,
        })
    }
}

impl<'w, W: Write> ObjWriter<'w, W> 
{
    fn vertex(&mut self, vector: &Vector) -> std::io::Result<()> {
        let line = format!("v {} {} {}\n", vector.x, vector.y, vector.z);
        self.writer.write_all(line.as_bytes())
    }

    #[allow(unused)]
    pub fn triangle(&mut self, triangle: &Triangle) -> std::io::Result<()> {
        for i in 0..3 {
            self.vertex(&triangle.0[i])?;
        }
        let base = self.vertex_count;
        let line = format!("f {} {} {}\n", base+1, base+2, base+3);
        self.vertex_count += 3;
        self.writer.write_all(line.as_bytes())
    }

    #[allow(unused)]
    pub fn triangles(&mut self, triangles: &[Triangle]) -> std::io::Result<()> {
        for triangle in triangles.iter() {
            self.triangle(triangle)?;
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn line(&mut self, vertices: &[Vector]) -> std::io::Result<()> {
        let mut line = String::from("l");
        for vertex in vertices.iter() {
            self.vertex(vertex)?;
            self.vertex_count += 1;
            line += &format!(" {}", self.vertex_count);
        }
        line += "\n";
        self.writer.write_all(line.as_bytes())
    }
}

