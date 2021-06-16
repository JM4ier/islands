use crate::{draw::*, map::*};
use cgmath::{prelude::*, Vector2};
use rand::prelude::*;

struct Node {
    pos: Vector2<f32>,
    parent: usize,
}

fn attractors(width: usize, height: usize) -> Vec<Node> {
    let mut rng = rand::thread_rng();
    let count = 10_000;
    let mut points = Vec::new();

    let (width, height) = (width as f32, height as f32);

    for _ in 0..count {
        points.push(Node {
            pos: Vector2::new(width * rng.gen::<f32>(), height * rng.gen::<f32>()),
            parent: 0,
        });
    }

    points
}

pub fn generate_valleys(width: usize, height: usize) -> Map {
    // radii squared, as to make it easier computation wise
    let radius_death = 100.0;
    let radius_influence = 400.0;

    let spawn_distance = 2.0;

    let mut attractors = attractors(width, height);

    const NONE: usize = usize::MAX;

    let mut rivers = vec![Node {
        pos: Vector2::new(0.0, 0.0),
        parent: 0,
    }];

    let mut influences = vec![Vector2::zero()];

    // loop until a fixpoint is found
    loop {
        for inf in influences.iter_mut() {
            *inf = Vector2::zero();
        }

        for attr in attractors.iter() {
            if attr.parent == NONE {
                continue;
            }
            let apos = attr.pos;
            let rpos = rivers[attr.parent].pos;
            if apos.distance2(rpos) < radius_influence {
                influences[attr.parent] += (apos - rpos).normalize();
            }
        }

        let mut new_rivers = Vec::new();
        for ((river, influence), idx) in rivers.iter().zip(influences.iter_mut()).zip(0..) {
            if influence.magnitude2() > 0.01 {
                new_rivers.push(Node {
                    pos: river.pos + *influence * spawn_distance,
                    parent: idx,
                });
            }
        }

        if new_rivers.len() == 0 {
            break;
        }

        if new_rivers.len() > 0 {
            for river in new_rivers.into_iter() {
                for attr in attractors.iter_mut() {
                    if attr.parent != NONE
                        && attr.pos.distance2(rivers[attr.parent].pos)
                            > attr.pos.distance2(river.pos)
                    {
                        attr.parent = rivers.len();
                    }
                }

                rivers.push(river);
                influences.push(Vector2::zero());
            }
        }

        for attr in attractors.iter_mut() {
            if attr.parent != NONE && attr.pos.distance2(rivers[attr.parent].pos) < radius_death {
                attr.parent = NONE;
            }
        }
    }

    let mut map = Map::new(width, height);

    let clamp = |v, l| {
        let l = l as isize;
        if v < 0 {
            0
        } else if v >= l {
            l - 1
        } else {
            v
        }
    };

    for river in rivers.iter() {
        let x1 = clamp(river.pos.x as _, width);
        let y1 = clamp(river.pos.y as _, height);
        let x2 = clamp(rivers[river.parent].pos.x as _, width);
        let y2 = clamp(rivers[river.parent].pos.y as _, height);
        draw_line(&mut map, x1, y1, x2, y2, &|_| 1.0);
    }

    map
}
