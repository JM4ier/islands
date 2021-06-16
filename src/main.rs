use std::fs::File;
use std::io::*;

mod draw;
mod flow;
mod geometry;
mod lake;
mod log;
mod map;
mod obj;
mod river;
mod simplex;
mod vis;
mod water_terrain;

fn main() -> std::io::Result<()> {
    let export_wetmap = false;
    let export_heightmap = true;
    let export_obj = false;
    let export_rgb = true;

    let size = 2000;
    let water_range = 6;
    let ocean_height = 20.0;

    println!("\n-- Island Generator --\n");
    println!("Generating a {}x{} island.", size, size);

    let logger = log::Logger::new();

    let map = logger.do_task("Generating Simplex Map", || {
        simplex::simplex_map(size, size)
    });
    let targets = logger.do_task("Finding Flow Targets", || {
        flow::find_targets(&map, water_range)
    });
    let mut river_map = logger.do_task("Generating River Map", || {
        river::create_flow_map(&map, &targets)
    });
    let lake_map = logger.do_task("Generating Lake Map", || {
        lake::lake_map(&map, &river_map, &targets, ocean_height)
    });

    logger.do_task("Adjusting River Map", || river_map.map(|h| h.powf(0.45)));

    let water_terrain = logger.do_task("Generating Terrain", || {
        water_terrain::create_heightmap(&river_map, &lake_map)
    });

    if export_obj {
        logger.do_task("Exporting Terrain OBJ", || {
            let terrain_file = File::create("terrain.obj")?;
            let mut buf_writer = BufWriter::new(terrain_file);
            let mut writer = obj::ObjWriter::new(&mut buf_writer)?;
            water_terrain.export_cube_mesh(&mut writer)
        })?;
    }

    if export_heightmap {
        logger.do_task("Exporting Heightmap", || {
            let heightmap_file = File::create("terrain.png")?;
            water_terrain.export_image(heightmap_file)
        })?;
    }

    if export_wetmap {
        logger.do_task("Exporting River/Lake", || {
            let river_file = File::create("river.png")?;
            river_map.export_image(river_file)?;
            let lake_file = File::create("lake.png")?;
            lake_map.export_image(lake_file)
        })?;
    }

    if export_rgb {
        logger.do_task("Exporting Visualization", || {
            let file = File::create("vis.png")?;
            vis::visualize(&map, ocean_height, file)
        })?;
    }

    logger.finalize();

    Ok(())
}
