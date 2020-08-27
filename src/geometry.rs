use cgmath::Vector3;
use cgmath::prelude::InnerSpace;

/// Three dimensional floating point vector type 
pub type Vector = Vector3<f32>;

#[derive(Debug, Clone)]
pub struct Triangle(pub [Vector; 3]);

impl Triangle {
    #[allow(unused)]
    /// Normal vector of the triangle pointing perpendicular to its surface
    pub fn normal(&self) -> Vector {
        let vec = self.0;
        (vec[1] - vec[0]).cross(vec[2] - vec[0]).normalize()
    }
}

