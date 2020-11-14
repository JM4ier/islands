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
            cell_size: 100,
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

        let neighbors = self.indirect_neighbors(coords);

        let mut min = Vector2::new(x as _, y as _);
        let mut max = min;
        let mut shapes = Vec::new();
        let mut oceans = Vec::new();

        for cell in neighbors {
            let (nx, ny) = cell;

            // find adjacent oceans
            let adj = self.adjacent(cell).clone();
            for a in adj {
                if *self.cell_type(a) == CellType::Ocean {
                    oceans.push(a);
                }
            }

            // clone shape
            let Voronoi(shape) = self.voronoi(cell);
            let offset = Vector2::new(nx as _, ny as _);

            shapes.push(shape.iter().map(|v| v + offset).collect::<Vec<_>>());

            // find min/max of landmass
            for vertex in shape {
                let vertex = vertex + offset;
                min.x = min.x.min(vertex.x);
                min.y = min.y.min(vertex.y);
                max.x = max.x.max(vertex.x);
                max.y = max.y.max(vertex.y);
            }
        }

        // remove duplicates of ocean cells
        oceans.sort();
        oceans.dedup();

        let chunk_width = max.x - min.x;
        let chunk_height = max.y - min.y;

        let scale = self.params.cell_size;
        let fscale = scale as f32;
        let real_width = ((chunk_width + 1.0) * fscale) as usize;
        let real_height = ((chunk_height + 1.0) * fscale) as usize;

        let mapping = |v: Vector2| (v - min) * fscale;

        let shapes = shapes
            .into_iter()
            .map(
                |shape|
                shape.into_iter().map(mapping).collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        // remove oceans outside bounding box
        let oceans = oceans.into_iter()
            .map(|(x, y)| {
                let v = Vector2::new(x as _, y as _) + self.cell_center((x, y));
                mapping(v)
            })
            .filter(|&v| v.x >= 0.0 && v.y >= 0.0 && v.x < (real_width as _) && v.y < (real_height as _))
            .map(|v| (v.x as usize, v.y as usize))
            .collect::<Vec<_>>();

        let mut rng = self.chunk_layer_rng(coords, 4);
        let seed = vec![0; 10].into_iter().map(|_| rng.gen()).collect();
        let mut noise = crate::simplex::simplex_map(real_width, real_height, (min.x, min.y), 0.01, seed);

        let edge_scaling = polygon_scaling(real_width, real_height, shapes, oceans);
        noise.scale(&edge_scaling);

        noise
    }}
});

fn polygon_scaling(
    width: usize,
    height: usize,
    shapes: Vec<Vec<Vector2>>,
    oceans: Vec<(usize, usize)>,
) -> Map {
    use crate::map::Point;

    let slope = 0.03;
    let mut map = Map::new(width, height);

    let ocean_height = 1e-6;
    let mut queue = BinaryHeap::new();

    for x in 0..width {
        queue.push(Point {
            x,
            y: 0,
            z: ocean_height,
        });
        queue.push(Point {
            x,
            y: height - 1,
            z: ocean_height,
        });
    }

    for y in 1..(height - 1) {
        queue.push(Point {
            x: 0,
            y,
            z: ocean_height,
        });
        queue.push(Point {
            x: width - 1,
            y,
            z: ocean_height,
        });
    }

    for &(x, y) in oceans.iter() {
        queue.push(Point {
            x,
            y,
            z: ocean_height,
        });
    }

    let inside = |a: Vector2, b: Vector2, c: Vector2| {
        let dir = (b - a).normalize();
        let perp_dir = Vector2::new(-dir.y, dir.x);
        perp_dir.dot(&(c - a)) > 0.0
    };

    let in_polygon = |x, y| {
        let c = Vector2::new(x as _, y as _);
        for poly in shapes.iter() {
            let mut outside = false;
            for i in 0..poly.len() {
                let a = poly[i];
                let b = poly[(i + 1) % poly.len()];

                if !inside(a, b, c) {
                    outside = true;
                    break;
                }
            }
            if !outside {
                return true;
            }
        }
        false
    };

    while let Some(point) = queue.pop() {
        let Point { x, y, z } = point;
        if map[(x, y)] >= ocean_height {
            // already edited at that point
            continue;
        }

        map[(x, y)] = z;

        let inc_z = (z + slope).min(1.0);

        let from = |x| if x > 0 { x - 1 } else { x };
        let to = |x, bound| if x < bound - 1 { x + 1 } else { x };

        for dx in from(x)..=to(x, width) {
            for dy in from(y)..=to(y, height) {
                if map[(dx, dy)] <= 0.0 {
                    let z = if in_polygon(dx, dy) { inc_z } else { z };
                    queue.push(Point { x: dx, y: dy, z });
                }
            }
        }
    }

    map.map(|v| (((v - 0.5) * std::f32::consts::PI).sin() + 1.0) * 0.5);

    map
}

impl World {
    fn chunk_layer_rng(&self, (x, y): ChunkCoord, layer: u64) -> Pcg64 {
        let lower_mask = 0xFF_FF_FF_FF;
        let x = (x as u64) & lower_mask; // lower 4 bytes of x
        let y = ((y as u64) << 32) & !lower_mask; // lower 4 bytes of y shifted to the upper 4 bytes
        let chunk_mod = x | y;
        let seed = (self.seed ^ chunk_mod) + layer;
        Pcg64::seed_from_u64(seed)
    }

    fn indirect_neighbors(&mut self, coords: ChunkCoord) -> Vec<ChunkCoord> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(coords);
        queue.push_back(coords);

        while let Some(cell) = queue.pop_front() {
            for n in self.neighbors(cell) {
                if !visited.contains(n) {
                    queue.push_back(*n);
                    visited.insert(*n);
                }
            }
        }
        visited.into_iter().collect()
    }
}
