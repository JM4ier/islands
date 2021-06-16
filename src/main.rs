use std::fs::File;
use std::io::*;

mod draw;
mod flow;
mod geometry;
mod lake;
mod map;
mod obj;
mod river;
mod simplex;
mod water_terrain;

fn main() -> std::io::Result<()> {
    let export_wetmap = false;
    let export_heightmap = true;
    let export_obj = false;

    let size = 2000;
    let water_range = 6;
    let ocean_height = 20.0;

    println!("\n-- Island Generator --\n");
    println!("Generating a {}x{} island.", size, size);
    let start_time = std::time::SystemTime::now();

    let map = log("Generating Simplex Map", || {
        simplex::simplex_map(size, size)
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
            let terrain_file = File::create("terrain.obj")?;
            let mut buf_writer = BufWriter::new(terrain_file);
            let mut writer = obj::ObjWriter::new(&mut buf_writer)?;
            water_terrain.export_cube_mesh(&mut writer)
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
    std::io::stdout().lock().flush().ok();
    let start = std::time::SystemTime::now();
    let result = fun();
    match start.elapsed() {
        Ok(elapsed) => println!("({:.3} s)", elapsed.as_secs_f32()),
        Err(e) => println!("(Timing error: {:?})", e),
    }
    result
}
