use rand;
use rand::prelude::*;
use std::fs::File;

mod flow;
mod geometry;
mod lake;
mod layer;
mod map;
mod obj;
mod river;
mod simplex;
mod water_terrain;

fn main() {
    let seed = rand::thread_rng().gen::<u64>();
    println!("Using seed {}", seed);
    let mut world = layer::World::from_seed(seed);

    let (mut minx, mut maxx, mut miny, mut maxy) = (0, 0, 0, 0);
    let cells = 8;

    for x in 0..cells {
        for y in 0..cells {
            if *world.cell_type((x, y)) != layer::CellType::Ocean {
                let parent = *world.parent((x, y));
                let ((x1, y1), heightmap) = world.heightmap(parent);

                let x2 = x1 + heightmap.width() as i64;
                let y2 = y1 + heightmap.height() as i64;

                minx = minx.min(*x1);
                miny = miny.min(*y1);
                maxx = maxx.max(x2);
                maxy = maxy.max(y2);
            }
        }
    }

    let width = (maxx - minx) as usize;
    let height = (maxy - miny) as usize;

    let mut worldmap = map::Map::new(width, height);

    for x in 0..cells {
        for y in 0..cells {
            if *world.parent((x, y)) == (x, y) && *world.cell_type((x, y)) != layer::CellType::Ocean
            {
                let ((offset_x, offset_y), heightmap) = world.heightmap((x, y));
                let offset_x = (offset_x - minx) as usize;
                let offset_y = (offset_y - miny) as usize;

                worldmap.binary_op_at(
                    heightmap,
                    |a, b| a + b,
                    (offset_x, offset_y),
                    (offset_x + heightmap.width(), offset_y + heightmap.height()),
                );
            }
        }
    }

    let heightmap_file = File::create("heightmap.png").unwrap();
    worldmap.export_image(heightmap_file).unwrap();
}

#[allow(unused)]
fn old_main() -> std::io::Result<()> {
    let export_wetmap = true;
    let export_heightmap = true;
    let export_obj = false;

    let size = 800;
    let water_range = 6;
    let ocean_height = 20.0;

    println!("\n-- Island Generator --\n");
    println!("Generating a {}x{} island.", size, size);
    let start_time = std::time::SystemTime::now();

    let map = log("Generating Simplex Map", || {
        simplex::scaled_simplex_map(size, size)
    });
    let targets = log("Finding Flow Targets", || {
        flow::find_targets(&map, water_range)
    });
    let mut river_map = log("Generating River Map", || {
        river::create_flow_map(&map, &targets)
    });
    let lake_map = log("Generating Lake Map", || {
        lake::lake_map(&map, &river_map, &targets, ocean_height)
    });

    log("Adjusting River Map", || river_map.map(|h| h.powf(0.45)));

    let water_terrain = log("Generating Terrain", || {
        water_terrain::create_heightmap(&river_map, &lake_map)
    });

    if export_obj {
        log("Exporting Terrain OBJ", || {
            let mut terrain_file = File::create("terrain.obj")?;
            let mut writer = obj::ObjWriter::new(&mut terrain_file)?;
            water_terrain.export_mesh(&mut writer)
        })?;
    }

    if export_heightmap {
        log("Exporting Heightmap", || {
            let heightmap_file = File::create("terrain.png")?;
            water_terrain.export_image(heightmap_file)
        })?;
    }

    if export_wetmap {
        log("Exporting River/Lake", || {
            let river_file = File::create("river.png")?;
            river_map.export_image(river_file)?;
            let lake_file = File::create("lake.png")?;
            lake_map.export_image(lake_file)
        })?;
    }

    let elapsed = start_time.elapsed().expect("Timing Error.");
    println!("Total\t\t\t[{:.3} s]\n", elapsed.as_secs_f32());

    Ok(())
}

fn log<T, F: FnMut() -> T>(name: &'static str, mut fun: F) -> T {
    print!("{}\t", name);
    use std::io::Write;
    std::io::stdout().lock().flush().ok();
    let start = std::time::SystemTime::now();
    let result = fun();
    match start.elapsed() {
        Ok(elapsed) => println!("({:.3} s)", elapsed.as_secs_f32()),
        Err(e) => println!("(Timing error: {:?})", e),
    }
    result
}
