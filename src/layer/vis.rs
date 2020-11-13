use crate::layer::*;
use coffee::graphics::*;
use coffee::load::Task;
use coffee::*;
use rand;
use rand::prelude::*;

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
        let mut linemesh = Mesh::new_with_tolerance(0.001);
        let mut centermesh = Mesh::new_with_tolerance(0.001);

        let mut world = World::from_seed(420);

        let p = |v: Vector2| Point::new(v.x, v.y);

        let mut rng = rand::thread_rng();

        for x in 0..width {
            for y in 0..height {
                let coord = (x as i64, y as i64);
                let cell_type = *world.cell_type(coord);

                let mut color = match cell_type {
                    CellType::Strong => Color::new(0.5, 0.0, 0.0, 1.0),
                    CellType::Weak => Color::new(0.0, 0.5, 0.0, 1.0),
                    _ => Color::new(0.0, 0.0, 0.5, 1.0),
                };

                let center = *world.center(coord);

                centermesh.fill(
                    Shape::Rectangle(Rectangle {
                        x: x as f32 + center.x,
                        y: y as f32 + center.y,
                        width: 0.1,
                        height: 0.1,
                    }),
                    Color::new(1.0, 0.0, 1.0, 1.0),
                );

                let Voronoi(poly) = world.voronoi(coord);
                let mut polyline = poly
                    .iter()
                    .map(|v| 0.9 * v + 0.1 * center)
                    .map(|v| Point::new(v.x + x as f32, v.y + y as f32))
                    .collect::<Vec<_>>();
                polyline.push(polyline[0]);

                color.a = 1.0;
                mesh.stroke(
                    Shape::Polyline {
                        points: polyline.clone(),
                    },
                    color,
                    0.05,
                );
                color.a = 0.2;
                polyline.remove(polyline.len() - 1);
                mesh.fill(Shape::Polyline { points: polyline }, color);

                let connections = world.connected(coord).clone();
                for neighbor in connections {
                    let ncenter = *world.center(neighbor);

                    let from = center + Vector2::new(x as _, y as _);
                    let to = ncenter + Vector2::new(neighbor.0 as _, neighbor.1 as _);

                    linemesh.stroke(
                        Shape::Polyline {
                            points: vec![p(from), p(to)],
                        },
                        Color::new(1.0, 1.0, 0.0, 0.1),
                        0.05,
                    );
                }
            }
        }

        let mut target = frame.as_target();
        let mut target = target.transform(Transformation::scale(scale));

        mesh.draw(&mut target);
        linemesh.draw(&mut target);
        centermesh.draw(&mut target);

        // Draw your game here. Check out the `graphics` module!
    }
}
