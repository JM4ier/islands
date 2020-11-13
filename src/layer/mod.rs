#![allow(unused)]
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::collections::*;

use crate::geometry::{Line2, Vector2};

/// Visualization
pub mod vis;

pub type ChunkCoord = (i64, i64);

macro_rules! layer_world {
    (
        $name:ident {
            $(
                $(#[$layer_meta:meta])*
                fn $layer_name:ident ($self:ident, $coord_arg:ident : $coord_ty:ty) -> $layer_ty:ty {
                    $layer_impl:expr
                }
            )*
        }
    ) => {
        pub type Seed = u64;

        #[derive(Debug, PartialEq)]
        pub struct $name {
            seed: Seed,
            params: WorldParams,
            $(
                $layer_name: HashMap<$coord_ty, $layer_ty>,
            )*
        }

        impl $name {
            pub fn from_seed(seed: Seed) -> Self {
                Self {
                    seed,
                    ..Default::default()
                }
            }

            $(
                $(#[$layer_meta])*
                pub fn $layer_name(&mut $self, $coord_arg: $coord_ty) -> &$layer_ty {
                    if !$self.$layer_name.contains_key(&$coord_arg) {
                        let val = $layer_impl;
                        $self.$layer_name.insert($coord_arg, val);
                    }
                    $self.$layer_name.get(&$coord_arg).unwrap()
                }
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    seed: Seed::default(),
                    params: WorldParams::default(),
                    $(
                        $layer_name: HashMap::new(),
                    )*
                }
            }
        }

        #[test]
        fn world_layer_commutativity() {
            let (width, height) = (20, 20);
            let seed: u64 = 3498398439434;
            let mut indices = Vec::new();
            for x in 0..width {
                for y in 0..height {
                    indices.push((x, y));
                }
            }

            let mut shuf_indices = indices.clone();
            let mut rng = rand::thread_rng();
            shuf_indices.shuffle(&mut rng);

            assert!(indices != shuf_indices);

            $(
                let mut world1 = World::from_seed(seed);
                let mut world2 = World::from_seed(seed);

                for idx in indices.iter() {
                    world1.$layer_name(*idx);
                }
                for idx in shuf_indices.iter() {
                    world2.$layer_name(*idx);
                }

                assert_eq!(world1, world2, "Commutativity of layer {} is violated", stringify!($layer_name));
            )*
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldParams {
    cell_size: usize,
    strong_prob: f32,
    weak_prob: f32,
    reach: usize,
}

impl Eq for WorldParams {}

impl Default for WorldParams {
    fn default() -> Self {
        Self {
            cell_size: 500,
            strong_prob: 0.1,
            weak_prob: 0.3,
            reach: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Voronoi(Vec<Vector2>);

pub type Adjacency = Vec<ChunkCoord>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CellType {
    Strong,
    Weak,
    Ocean,
}

layer_world!(World {

    /// Find the center of each voronoi cell
    fn center(self, coords: ChunkCoord) -> Vector2 {{
        let mut rng = self.chunk_layer_rng(coords, 0);
        let randomness = 0.9;
        let map = |v: f32| randomness * v + (1.0 - randomness) * 0.5;
        Vector2::new(map(rng.gen()), map(rng.gen()))
    }}

    /// Find a voronoi cell for each chunk
    fn voronoi(self, coords: ChunkCoord) -> Voronoi {{
        let (x, y) = coords;

        // center of this voronoi cell
        let center = self.center(coords).clone();

        // centers of surrounding voronoi cells
        let mut surrounding = Vec::with_capacity(8);

        for nx in (x-1)..=(x+1) {
            for ny in (y-1)..=(y+1) {
                let neighbor = (nx, ny);
                if neighbor != coords {
                    let cell_offset = Vector2::new((nx-x) as _, (ny-y) as _);
                    surrounding.push(cell_offset + self.center(neighbor).clone());
                }
            }
        }

        // sort surrounding based on angle around center point
        surrounding.sort_by(|p, q| {
            let angle = |p: &Vector2| (p.x - center.x).atan2(p.y - center.y);
            angle(p).partial_cmp(&angle(q)).unwrap()
        });

        // edge lines of the voronoi cell
        // note that there can be unneeded lines in this list which are covered by other lines
        let mut lines = surrounding.iter().map(|p| Line2::dividing_mid(center.clone(), p)).collect::<Vec<_>>();

        let mut i = 0;
        while i < lines.len() {
            let prev = (i + lines.len() - 1) % lines.len();
            let next = (i + 1) % lines.len();
            if let Some(inter) = lines[prev].intersection(&lines[next]) {
                let a = (center - lines[i].a).normalize();
                let b = (inter - lines[i].a).normalize();
                let c = (inter - center).normalize();

                if a.dot(&b) > 0.0 && a.dot(&c) < 0.0 {
                    // acute angle -> line segment i is not needed
                    lines.remove(i);
                    continue;
                }

            }

            i += 1;
        }

        // find the vertex list as intersections of adjacent lines
        let mut vertices = Vec::with_capacity(lines.len());
        for i in 0..lines.len() {
            let next = (i+1) % lines.len();
            let vertex = lines[i].intersection(&lines[next]).unwrap();
            vertices.push(vertex);
        }

        Voronoi(vertices)
    }}

    /// Adjacent Voronoi cells
    fn adjacency(self, coords: ChunkCoord) -> Adjacency {{
        let (x, y) = coords;
        let Voronoi(shape) = self.voronoi(coords).clone();
        let center = self.center(coords).clone();

        let mut neighbors = Vec::new();

        for nx in (x-1)..=(x+1) {
            for ny in (y-1)..=(y+1) {
                let n = (nx, ny);
                if n == coords {
                    continue;
                }

                let offset = Vector2::new((nx-x) as _, (ny-y) as _);
                let ncenter = offset + self.center(n);

                // if this point lies on an edge of the voronoi shape, the neighbor is adjacent
                let mid = 0.5 * (center + ncenter);

                for idx in 0..shape.len() {
                    let next = (idx+1) % shape.len();

                    let from = shape[idx];
                    let to = shape[next];

                    let a = (to-from).normalize();
                    let b = (mid-from).normalize();

                    if a.dot(&b) > 0.9999 {
                        neighbors.push(n);
                        break;
                    }
                }

            }
        }

        neighbors
    }}

    fn cell_type(self, coords: ChunkCoord) -> CellType {{
        let mut rng = self.chunk_layer_rng(coords, 3);
        let val = rng.gen::<f32>();
        if val < self.params.strong_prob {
            CellType::Strong
        } else if val < self.params.strong_prob + self.params.weak_prob {
            CellType::Weak
        } else {
            CellType::Ocean
        }
    }}

    /// Searches the closest 'strong' Cell
    ///
    /// If it finds one within reach, that strong cell is returned,
    /// otherwise the cell itself is returned
    fn parent(self, coords: ChunkCoord) -> ChunkCoord {{
        let mut strong_cell = coords;

        let mut q = VecDeque::new();
        let mut visited = HashSet::new();
        q.push_back((0, coords));
        visited.insert(coords);

        if *self.cell_type(coords) == CellType::Ocean {
            coords
        } else {
            while let Some((dist, cell)) = q.pop_front() {
                if *self.cell_type(cell) == CellType::Strong {
                    strong_cell = cell;
                    break;
                }
                if dist < self.params.reach {
                    let adj = self.adjacency(cell);
                    for neighbor in adj {
                        if !visited.contains(neighbor) {
                            q.push_back((dist+1, *neighbor));
                            visited.insert(*neighbor);
                        }
                    }
                }
            }
            strong_cell
        }
    }}

    /// list of neighbors that are connected to this cell
    fn connected(self, coords: ChunkCoord) -> Adjacency {{
        let mut connected = Adjacency::new();
        let neighbors = self.adjacency(coords).clone();
        let parent = *self.parent(coords);
        for n in neighbors.iter() {
            if *self.parent(*n) == parent {
                connected.push(*n);
            }
        }
        connected
    }}
});

impl World {
    fn chunk_layer_rng(&self, (x, y): ChunkCoord, layer: u64) -> Pcg64 {
        let lower_mask = 0xFF_FF_FF_FF;
        let x = (x as u64) & lower_mask; // lower 4 bytes of x
        let y = ((y as u64) << 32) & !lower_mask; // lower 4 bytes of y shifted to the upper 4 bytes
        let chunk_mod = x | y;
        let seed = (self.seed ^ chunk_mod) + layer;
        Pcg64::seed_from_u64(seed)
    }
}