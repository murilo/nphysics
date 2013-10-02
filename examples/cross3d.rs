#[link(name     = "cross3d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "e0e11c1b-2909-420a-9c17-563d6e9d6f6c")];
#[crate_type = "bin"];
#[warn(non_camel_case_types)]

extern mod std;
extern mod extra;
extern mod kiss3d;
extern mod graphics3d;
extern mod nphysics;
extern mod nalgebra;
extern mod ncollide;

use std::num::One;
use kiss3d::window::Window;
use nalgebra::mat::Translation;
use nalgebra::vec::Vec3;
use ncollide::geom::{Geom, Box, Plane, CompoundAABB};
use nphysics::world::BodyWorld;
use nphysics::aliases::dim3;
use nphysics::object::{RigidBody, Static, Dynamic, RB};
use graphics3d::engine::GraphicsManager;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    GraphicsManager::simulate(cross3d)
}

pub fn cross3d(window: &mut Window, graphics: &mut GraphicsManager) -> dim3::BodyWorld3d<f64> {
    /*
     * World
     */
    let mut world = BodyWorld::new();
    world.set_gravity(Vec3::new(0.0f64, -9.81, 0.0));

    /*
     * Planes
     */
    let rb   = RigidBody::new(Geom::new_plane(Plane::new(Vec3::y())), 0.0f64, Static, 0.3, 0.6);
    let body = @mut RB(rb);

    world.add_body(body);
    graphics.add(window, body);

    /*
     * Cross shaped geometry
     */
    let box1 = Box::new(Vec3::new(5.0f64, 0.25, 0.25));
    let box2 = Box::new(Vec3::new(0.25f64, 5.0, 0.25));
    let box3 = Box::new(Vec3::new(0.25f64, 0.25, 5.0));

    let mut cross_geoms = ~[];
    cross_geoms.push((One::one(), Geom::new_box(box1)));
    cross_geoms.push((One::one(), Geom::new_box(box2)));
    cross_geoms.push((One::one(), Geom::new_box(box3)));

    let cross = @CompoundAABB::new(cross_geoms);

    /*
     * Create the crosses 
     */
    let num     = 6;
    let rad     = 5.0;
    let shift   = (rad + 0.08) * 2.0;
    let centerx = shift * (num as f64) / 2.0;
    let centery = 30.0 + shift / 2.0;
    let centerz = shift * (num as f64) / 2.0;

    for i in range(0u, num) {
        for j in range(0u, num) {
            for k in range(0u, num) {
                let x = i as f64 * shift - centerx;
                let y = j as f64 * shift + centery;
                let z = k as f64 * shift - centerz;

                let mut rb = RigidBody::new(Geom::new_compound(cross), 1.0f64, Dynamic, 0.3, 0.5);

                rb.translate_by(&Vec3::new(x, y, z));

                let body = @mut RB(rb);

                world.add_body(body);
                graphics.add(window, body);
            }
        }
    }

    /*
     * Set up the camera and that is it!
     */
    graphics.look_at(Vec3::new(-30.0, 30.0, -30.0), Vec3::new(0.0, 0.0, 0.0));

    world
}