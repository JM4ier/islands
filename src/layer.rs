use coffee::graphics::Point;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::collections::*;

pub type ChunkCoord = (i64, i64);

macro_rules! layer_world {
    (
        $name:ident {
            $(
                fn $layer_name:ident ($self:ident, $coord_arg:ident) -> $layer_ty:ty {
                    $layer_impl:expr
                }
            )*
        }
    ) => {
        pub type Seed = u64;

        pub struct $name {
            seed: Seed,
            $(
                $layer_name: HashMap<ChunkCoord, $layer_ty>,
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
                pub fn $layer_name(&mut $self, $coord_arg: ChunkCoord) -> &$layer_ty {
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
                    $(
                        $layer_name: HashMap::new(),
                    )*
                }
            }
        }
    };
}

struct Voronoi;
struct Cell;

layer_world!(World {
    fn layer0(self, coords) -> Point {{
        let mut rng = self.chunk_layer_rng(coords, 0);
        Point::new(rng.gen(), rng.gen())
    }}

    fn layer1(self, coords) -> Voronoi {{
        let (x, y) = coords;
        let mut pos = [[Point::new(0.0, 0.0); 3]; 3];

        for dx in 0..3 {
            for dy in 0..3 {
                let rel = Point::new((dx - 1) as f32, (dy - 1) as f32);
                pos[dx][dy] = rel + self.layer0((x + dx as i64 - 1, y + dy as i64 - 1)).clone();
            }
        }

        todo!("Find voronoi shape based on the points")
    }}

    fn layer2(self, coords) -> Cell {{
        todo!("Merge voronoi cells and create a more complex shape based off it")
    }}
});

impl World {
    fn chunk_layer_rng(&self, (x, y): ChunkCoord, layer: u64) -> Pcg64 {
        let seed = self.seed ^ (x as u64) ^ ((y as u64) << 32);
        let seed = seed + layer;
        Pcg64::seed_from_u64(layer)
    }
}
