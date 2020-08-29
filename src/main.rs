use std::fs::File;

mod simplex;
mod geometry;
mod obj;
mod map;
mod river;
mod flow;
mod lake;
mod water_terrain;

fn main() -> std::io::Result<()> {
    let export_wetmap = true;
    let export_heightmap = true;
    let export_obj = false;

    let size = 800;
    let water_range = 6;
    let ocean_height = 20.0;

    println!("-- Island Generator --\n");

    let map = log("Generating Simplex Map", || simplex::simplex_map(size, size, true));
    let mut river_map = log("Generating River Map", || river::create_flow_map(&map, water_range));
    let lake_map = log("Generating Lake Map", || lake::lake_map(&map, &river_map, ocean_height, water_range));

    log("Adjusting River Map", || river_map.map(|h| h.powf(0.45)));

    let water_terrain = log("Generating Terrain", || water_terrain::create_heightmap(&river_map, &lake_map));

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

    println!();

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

