use crate::layer::*;
use coffee::graphics::*;
use coffee::load::Task;
use coffee::*;

pub fn run_visualization() -> Result<()> {
    Visualization::run(WindowSettings {
        title: String::from("Layer Visualization"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}

struct Visualization;

impl Game for Visualization {
    type Input = ();
    type LoadingScreen = ();

    fn load(_window: &Window) -> Task<Self> {
        Task::succeed(|| Self)
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(Color::WHITE);

        let scale = 50.0;
        let width = 20;
        let height = 20;
        let center_size = 0.1;

        let strong_cell_color = Color::new(0.0, 0.7, 0.0, 1.0);
        let weak_cell_color = Color::new(0.4, 1.0, 0.0, 1.0);
        let ocean_cell_color = Color::new(0.0, 0.0, 0.5, 1.0);
        let center_color = Color::new(1.0, 0.0, 0.0, 1.0);
        let connection_color = Color::new(1.0, 0.0, 0.0, 1.0);

        let new_mesh = || Mesh::new_with_tolerance(0.001);
        let p = |v: Vector2| Point::new(v.x, v.y);

        let mut voronoi_mesh = new_mesh();
        let mut connection_voronoi_mesh = new_mesh();
        let mut center_voronoi_mesh = new_mesh();

        let mut world = World::from_seed(929);

        for x in 0..width {
            for y in 0..height {
                let coord = (x as i64, y as i64);
                let offset = Vector2::new(x as _, y as _);

                let cell_type = *world.cell_type(coord);
                let center = *world.center(coord);

                // draw center point of each cell
                center_voronoi_mesh.fill(
                    Shape::Rectangle(Rectangle {
                        x: x as f32 + center.x - center_size * 0.5,
                        y: y as f32 + center.y - center_size * 0.5,
                        width: center_size,
                        height: center_size,
                    }),
                    center_color,
                );

                let mut poly_color = match cell_type {
                    CellType::Strong => strong_cell_color,
                    CellType::Weak => weak_cell_color,
                    _ => ocean_cell_color,
                };

                // find voronoi shape, and process it
                let Voronoi(poly) = world.voronoi(coord);
                let mut polyline = poly
                    .iter()
                    .map(|v| v + 0.08 * (center - v).normalize())
                    .map(|v| v + offset)
                    .map(p)
                    .collect::<Vec<_>>();

                // fill the shape transparently
                poly_color.a = 0.6;
                voronoi_mesh.fill(
                    Shape::Polyline {
                        points: polyline.clone(),
                    },
                    poly_color,
                );

                // draw the voronoi cells border without transparency
                polyline.push(polyline[0]); // add first vertex on the end to draw the last edge
                poly_color.a = 1.0;
                voronoi_mesh.stroke(Shape::Polyline { points: polyline }, poly_color, 0.05);

                // draw connections to adjacent voronoi cells
                let connections = world.connected(coord).clone();
                for neighbor in connections {
                    let ncenter = *world.center(neighbor);

                    let from = center + offset;
                    let to = ncenter + Vector2::new(neighbor.0 as _, neighbor.1 as _);

                    connection_voronoi_mesh.stroke(
                        Shape::Polyline {
                            points: vec![p(from), p(to), p(from)],
                        },
                        connection_color,
                        0.05,
                    );
                }
            }
        }

        let mut target = frame.as_target();
        let mut target = target.transform(Transformation::scale(scale));

        voronoi_mesh.draw(&mut target);
        connection_voronoi_mesh.draw(&mut target);
        center_voronoi_mesh.draw(&mut target);
    }
}
