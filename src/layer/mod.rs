use crate::map::Map;
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
                        #[allow(unused_mut)]
                        let mut implementation = || { $layer_impl };
                        let val = implementation();
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

    /// Find the center of each the voronoi cell at a chunk
    fn cell_center(self, coords: ChunkCoord) -> Vector2 {{
        let mut rng = self.chunk_layer_rng(coords, 0);

        // so that edges of voronoi cells don't go over their neighbors bounds
        let randomness = 0.97 / (2.0f32).sqrt();
        let map = |v: f32| randomness * v + (1.0 - randomness) * 0.5;
        Vector2::new(map(rng.gen()), map(rng.gen()))
    }}

    /// Find a voronoi cell for each chunk
    fn voronoi(self, coords: ChunkCoord) -> Voronoi {{
        let (x, y) = coords;

        // center of this voronoi cell
        let center = *self.cell_center(coords);

        // centers of surrounding voronoi cells relative to this cells origin (not center)
        let mut surrounding = Vec::with_capacity(8);

        for nx in (x-1)..=(x+1) {
            for ny in (y-1)..=(y+1) {
                let neighbor = (nx, ny);
                if neighbor != coords {
                    let cell_offset = Vector2::new((nx-x) as _, (ny-y) as _);
                    surrounding.push(cell_offset + self.cell_center(neighbor).clone());
                }
            }
        }

        // lines in the middle between this cells center and the neighbors cells center
        let mut lines = Vec::with_capacity(8);
        for i in 0..surrounding.len() {
            lines.push(Line2::dividing_mid(&center, &surrounding[i]));
        }

        // find closest side that is surely on the polygon
        let mut closest = 0;
        for i in 0..surrounding.len() {
            if (surrounding[i] - center).magnitude() < (surrounding[closest] - center).magnitude() {
                closest = i;
            }
        }

        // find vertices of voronoi polygon, by starting with a point on the edge of the polygon,
        // and then tracing along the polygon counterclockwise, while always choosing the closest
        // intersecting line
        let mut vertices = Vec::new();

        let mut tracer = 0.5 * (center + surrounding[closest]);
        let dir = surrounding[closest] - center;
        // rotate 90 degrees counterclockwise to point in the direction of the polygon side
        let mut dir = Vector2::new(-dir.y, dir.x).normalize();
        let mut idx = closest;
        let mut prev_idx = surrounding.len();

        loop {
            let trace_line = Line2::from_base_dir(tracer, dir);

            // find next edge point
            let mut next = None;
            let mut next_idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if i == idx || i == prev_idx {
                    continue;
                }
                if let Some(inter) = trace_line.intersection(&line) {
                    if dir.dot(&(inter - tracer)) < 0.0 {
                        // the intersection is behind us
                        continue;
                    }

                    match next {
                        None => {
                            next = Some(inter);
                            next_idx = i;
                        },
                        Some(n) => {
                            if (n - tracer).magnitude() > (inter - tracer).magnitude() {
                                next = Some(inter);
                                next_idx = i;
                            }
                        }
                    }
                }
            }

            let next = next.unwrap();
            if vertices.is_empty() {
                vertices.push(next);
            } else if (vertices[0] - next).magnitude() > 0.0001 {
                vertices.push(next);
            } else {
                break;
            }

            tracer = next;
            let perp_dir = Vector2::new(-dir.y, dir.x);
            let mut next_dir = (lines[next_idx].b - lines[next_idx].a).normalize();
            if perp_dir.dot(&next_dir) < 0.0 {
                next_dir = -next_dir;
            }
            dir = next_dir;
            prev_idx = idx;
            idx = next_idx;
        }

        Voronoi(vertices)
    }}

    /// Adjacent Voronoi cells, regardless what type they have
    fn adjacent(self, coords: ChunkCoord) -> Adjacency {{
        let (x, y) = coords;
        let Voronoi(shape) = self.voronoi(coords).clone();

        let mut neighbors = Vec::new();

        for nx in (x-1)..=(x+1) {
            for ny in (y-1)..=(y+1) {
                let n = (nx, ny);
                if n == coords {
                    continue;
                }

                let offset = Vector2::new((nx-x) as _, (ny-y) as _);

                let Voronoi(nshape) = self.voronoi(n).clone();

                let mut corners = 0;
                for p in shape.iter() {
                    for np in nshape.iter() {
                        let np = np + offset;
                        if (np-p).magnitude() < 0.001 {
                            // common corner
                            corners += 1;
                        }
                    }
                }

                if corners > 0 {
                    neighbors.push(n);
                }
            }
        }

        neighbors
    }}

    /// Cell type of the chunk, based off the probabilities in the params
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

    /// Searches the closest 'strong' Cell over land connections
    ///
    /// If it finds one within reach, that strong cell is returned,
    /// otherwise the cell itself is returned
    fn parent(self, coords: ChunkCoord) -> ChunkCoord {{
        if *self.cell_type(coords) == CellType::Ocean {
            return coords;
        }

        let mut strong_cell = coords;

        let mut q = VecDeque::new();
        let mut visited = HashSet::new();
        q.push_back((0, coords));
        visited.insert(coords);

        while let Some((dist, cell)) = q.pop_front() {
            if *self.cell_type(cell) == CellType::Strong {
                strong_cell = cell;
                break;
            }
            if dist < self.params.reach {
                let adj = self.adjacent(cell).clone();
                for neighbor in adj {
                    if !visited.contains(&neighbor) && *self.cell_type(neighbor) != CellType::Ocean {
                        q.push_back((dist+1, neighbor));
                        visited.insert(neighbor);
                    }

                }
            }
        }
        strong_cell
    }}

    /// list of land neighbors that are connected to this cell
    ///
    /// If the cell is ocean, itself is returned
    fn neighbors(self, coords: ChunkCoord) -> Adjacency {{
        let mut connected = Vec::new();
        let neighbors = self.adjacent(coords).clone();
        let parent = *self.parent(coords);
        for n in neighbors.iter() {
            if *self.parent(*n) == parent {
                connected.push(*n);
            }
        }
        connected
    }}

    fn heightmap(self, coords: ChunkCoord) -> Map {{
        assert_eq!(*self.cell_type(coords), CellType::Strong);
        let (x, y) = coords;

        let (min, max, shapes) = self.island_geometry(coords);
        let chunk_width = max.x - min.x;
        let chunk_height = max.y - min.y;

        let scale = self.params.cell_size;
        let real_width = (chunk_width + 1.0) as usize * scale;
        let real_height = (chunk_height + 1.0) as usize * scale;

        let shapes = shapes
            .into_iter()
            .map(
                |Voronoi(shape)|
                shape.into_iter().map(|v| (v - min) * scale as f32).collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        // TODO find enclosed oceans

        todo!()
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

    fn island_geometry(&mut self, coords: ChunkCoord) -> (Vector2, Vector2, Vec<Voronoi>) {
        let (x, y) = coords;

        let mut min = Vector2::new(x as _, y as _);
        let mut max = min;
        let mut shapes = Vec::new();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(coords);
        queue.push_back(coords);

        while let Some(cell) = queue.pop_front() {
            let (nx, ny) = cell;
            let Voronoi(shape) = self.voronoi(cell);
            shapes.push(Voronoi(shape.clone()));
            let offset = Vector2::new(nx as _, ny as _);

            for vertex in shape {
                let vertex = vertex + offset;
                min.x = min.x.min(vertex.x);
                min.y = min.y.min(vertex.y);
                max.x = max.x.max(vertex.x);
                max.y = max.y.max(vertex.y);
            }

            for n in self.neighbors(cell) {
                if !visited.contains(n) {
                    queue.push_back(*n);
                    visited.insert(*n);
                }
            }
        }

        (min, max, shapes)
    }
}
