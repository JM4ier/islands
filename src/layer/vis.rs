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
        frame.clear(Color::BLACK);

        let scale = 50.0;
        let width = 20;
        let height = 20;

        let mut mesh = Mesh::new_with_tolerance(0.001);

        let mut world = World::from_seed(42);

        let p = |v: Vector2| Point::new(v.x, v.y);

        let mut linemesh = Mesh::new_with_tolerance(0.0001);

        for x in 0..width {
            for y in 0..height {
                let coord = (x as i64, y as i64);
                let cell_type = *world.cell_type(coord);

                let color = match cell_type {
                    CellType::Strong => Color::new(0.5, 0.0, 0.0, 1.0),
                    CellType::Weak => Color::new(0.0, 0.5, 0.0, 1.0),
                    _ => Color::new(0.0, 0.0, 0.5, 1.0),
                };

                let center = *world.center(coord);
                let Voronoi(poly) = world.voronoi(coord);
                let polyline = poly
                    .iter()
                    .map(|v| 0.9 * v + 0.1 * center)
                    .map(|v| Point::new(v.x + x as f32, v.y + y as f32))
                    .collect::<Vec<_>>();
                mesh.fill(Shape::Polyline { points: polyline }, color);

                let connections = world.connected(coord).clone();
                for neighbor in connections {
                    let ncenter = *world.center(neighbor);

                    let from = center + Vector2::new(x as _, y as _);
                    let to = ncenter + Vector2::new(neighbor.0 as _, neighbor.1 as _);

                    let iter = 10;
                    for i in 0..=iter {
                        let t = (i as f32) / (iter as f32);
                        let pos = from * (1.0 - t) + t * to;
                        let size = 0.05;

                        linemesh.fill(
                            Shape::Rectangle(Rectangle {
                                x: pos.x,
                                y: pos.y,
                                width: size,
                                height: size,
                            }),
                            Color::new(1.0, 1.0, 0.0, 0.1),
                        );
                    }
                }
            }
        }

        let mut target = frame.as_target();
        let mut target = target.transform(Transformation::scale(scale));

        mesh.draw(&mut target);
        linemesh.draw(&mut target);

        // Draw your game here. Check out the `graphics` module!
    }
}
