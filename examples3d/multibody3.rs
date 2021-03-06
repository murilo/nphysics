extern crate nalgebra as na;

use na::{Isometry3, Point3, RealField, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::joint::{
    BallJoint, FixedJoint, HelicalJoint, PinSlotJoint, PlanarJoint, PrismaticJoint,
    RectangularJoint, RevoluteJoint, UniversalJoint,
};
use nphysics3d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodySet, DefaultColliderSet, Ground, MultibodyDesc,
};
use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use nphysics_testbed3d::Testbed;
use std::f64::consts::PI;

/*
 * NOTE: The `r` macro is only here to convert from f64 to the `N` scalar type.
 * This simplifies experimentation with various scalar types (f32, fixed-point numbers, etc.)
 */
pub fn init_world<N: RealField>(testbed: &mut Testbed<N>) {
    /*
     * World
     */
    let mechanical_world = DefaultMechanicalWorld::new(Vector3::new(r!(0.0), r!(-9.81), r!(0.0)));
    let geometrical_world = DefaultGeometricalWorld::new();
    let mut bodies = DefaultBodySet::new();
    let mut colliders = DefaultColliderSet::new();
    let joint_constraints = DefaultJointConstraintSet::new();
    let force_generators = DefaultForceGeneratorSet::new();

    /*
     * Shape that will be re-used for several multibody links.
     */
    let rad = r!(0.2);
    let cuboid = ShapeHandle::new(Cuboid::new(Vector3::repeat(rad)));
    let collider_desc = ColliderDesc::new(cuboid.clone()).density(r!(1.0));

    /*
     * Revolute joints.
     */
    let num = 6;
    let revo = RevoluteJoint::new(Vector3::x_axis(), r!(-0.1));
    let body_shift = Vector3::z() * (rad * r!(3.0 + 0.2));

    let mut multibody_desc = MultibodyDesc::new(revo)
        .body_shift(body_shift)
        .parent_shift(Vector3::new(r!(0.0), r!(5.0), r!(11.0)));

    let mut curr = &mut multibody_desc;

    for _ in 0usize..num {
        curr = curr.add_child(revo).set_body_shift(body_shift);
    }

    let multibody = multibody_desc.build();
    let multibody_handle = bodies.insert(multibody);

    // Create one collider for each link.
    for i in 0..num + 1 {
        let co = collider_desc.build(BodyPartHandle(multibody_handle, i));
        colliders.insert(co);
    }

    /*
     * Prismatic joint.
     */
    let mut prism = PrismaticJoint::new(Vector3::y_axis(), r!(0.0));
    prism.enable_min_offset(-rad * r!(2.0)); // Limit the joint so it does not fall indefinitely.
    let mut multibody_desc =
        MultibodyDesc::new(prism).parent_shift(Vector3::new(r!(0.0), r!(5.0), r!(5.0)));

    let mut curr = &mut multibody_desc;

    for _ in 0usize..num {
        curr = curr
            .add_child(prism)
            .set_parent_shift(Vector3::z() * rad * r!(3.0));
    }

    let multibody = multibody_desc.build();
    let multibody_handle = bodies.insert(multibody);

    // Create one collider for each link.
    for i in 0..num + 1 {
        let co = collider_desc.build(BodyPartHandle(multibody_handle, i));
        colliders.insert(co);
    }

    /*
     * Ball joint.
     */
    let spherical = BallJoint::new(na::zero());
    let mut multibody_desc = MultibodyDesc::new(spherical).parent_shift(Vector3::y() * r!(5.0));
    let mut curr = &mut multibody_desc;

    for i in 0usize..num {
        // The multibody links are initialized along a circle.
        let angle = r!(i as f64) * r!(2.0) * r!(PI) / r!(num as f64);
        let shift = rad * r!(5.0);
        let parent_shift = Vector3::zeros();
        let body_shift = Vector3::new(angle.cos(), r!(0.3), angle.sin()) * shift;

        curr = curr
            .add_child(spherical)
            .set_parent_shift(parent_shift)
            .set_body_shift(body_shift)
    }

    let multibody = multibody_desc.build();
    let multibody_handle = bodies.insert(multibody);

    // Create one collider for each link.
    for i in 0..num + 1 {
        let co = collider_desc.build(BodyPartHandle(multibody_handle, i));
        colliders.insert(co);
    }

    /*
     * Universal joint.
     */
    let axis1 = Vector3::x_axis();
    let axis2 = Vector3::z_axis();
    let fixed = FixedJoint::new(Isometry3::identity());
    let mut uni = UniversalJoint::new(axis1, axis2, r!(0.0), r!(0.0));
    uni.enable_angular_motor_2();
    uni.set_desired_angular_motor_velocity_2(r!(5.0));

    let parent_shift = Vector3::new(r!(0.0), r!(3.0), r!(-5.0));
    let body_shift = -Vector3::z();

    let mut multibody_desc = MultibodyDesc::new(fixed).parent_shift(parent_shift);

    multibody_desc.add_child(uni).set_body_shift(body_shift);

    // Remove the default damping so that it balances indefinitely.
    let mut multibody = multibody_desc.build();
    multibody.damping_mut().fill(r!(0.0));

    let multibody_handle = bodies.insert(multibody);
    colliders.insert(collider_desc.build(BodyPartHandle(multibody_handle, 0)));
    colliders.insert(collider_desc.build(BodyPartHandle(multibody_handle, 1)));

    /*
     * Helical joint.
     */
    let axis = Vector3::y_axis();

    let mut hel = HelicalJoint::new(axis, r!(1.0), r!(0.0));
    hel.set_desired_angular_motor_velocity(r!(4.0));

    let parent_shift = Vector3::new(r!(0.0), r!(-2.0), r!(10.0));
    let helical_multibody = MultibodyDesc::new(hel).parent_shift(parent_shift).build();
    let helical_handle = bodies.insert(helical_multibody);
    colliders.insert(collider_desc.build(BodyPartHandle(helical_handle, 0)));

    /*
     * Planar joint.
     */
    let axis1 = Vector3::z_axis();
    let axis2 = Vector3::y_axis();
    let shift = Vector3::new(r!(0.0), r!(-2.0), r!(5.0));
    let width = r!(5.0) * rad * r!(4.0);
    for i in 0..5 {
        for j in 0..5 {
            let mut x = r!(i as f64) * rad * r!(4.0) - width / r!(2.0);
            let y = r!(j as f64) * rad * r!(4.0) - width / r!(2.0);

            if j % 2 == 0 {
                x += rad * r!(2.0);
            }

            let mut planar = PlanarJoint::new(axis1, axis2, x, y, r!(0.0));
            planar.enable_min_offset_1(-width / r!(2.0));
            planar.enable_max_offset_1(width / r!(2.0));
            planar.enable_min_offset_2(r!(-5.0));

            let multibody = MultibodyDesc::new(planar).parent_shift(shift).build();
            let multibody_handle = bodies.insert(multibody);
            colliders.insert(collider_desc.build(BodyPartHandle(multibody_handle, 0)));
        }
    }

    /*
     * Rectangular joint.
     */
    let axis1 = Vector3::z_axis();
    let axis2 = Vector3::y_axis();
    let shift = Vector3::new(r!(0.0), r!(-2.0), r!(0.0));
    let width = r!(5.0) * rad * r!(4.0);

    for i in 0..5 {
        for j in 0..5 {
            let mut x = r!(i as f64) * rad * r!(4.0) - width / r!(2.0);
            let y = r!(j as f64) * rad * r!(4.0) - width / r!(2.0);

            if j % 2 == 0 {
                x += rad * r!(2.0);
            }

            let mut rect = RectangularJoint::new(axis1, axis2, x, y);
            rect.enable_min_offset_1(-width / r!(2.0));
            rect.enable_max_offset_1(width / r!(2.0));
            rect.enable_min_offset_2(r!(-5.0));

            let multibody = MultibodyDesc::new(rect).parent_shift(shift).build();
            let multibody_handle = bodies.insert(multibody);
            colliders.insert(collider_desc.build(BodyPartHandle(multibody_handle, 0)));
        }
    }

    /*
     * Pin-slot joint.
     */
    let cuboid = ShapeHandle::new(Cuboid::new(Vector3::new(rad * r!(5.0), rad, rad * r!(5.0))));
    let collider_desc = ColliderDesc::new(cuboid).density(r!(1.0));
    let axis_v = Vector3::y_axis();
    let axis_w = Vector3::x_axis();
    let shift = Vector3::z() * r!(-1.5);

    let mut pin_slot = PinSlotJoint::new(axis_v, axis_w, r!(-10.0), r!(0.0));
    pin_slot.set_desired_linear_motor_velocity(r!(3.0));

    let pin_slot_multibody = MultibodyDesc::new(pin_slot).parent_shift(shift).build();
    let pin_slot_handle = bodies.insert(pin_slot_multibody);
    colliders.insert(collider_desc.build(BodyPartHandle(pin_slot_handle, 0)));

    /*
     * Set up the testbed.
     */
    testbed.add_callback(move |_, _, bodies, _, _, _| {
        /*
         * Activate the helical joint motor if it is to low.
         */
        // Might be None if the user interactively deleted the helical body.
        let link = bodies
            .multibody_mut(helical_handle)
            .and_then(|mb| mb.link_mut(0));
        if let Some(helical) = link {
            let dof = helical
                .joint_mut()
                .downcast_mut::<HelicalJoint<N>>()
                .unwrap();

            if dof.offset() < r!(-5.0) {
                dof.enable_angular_motor();
            } else if dof.offset() > r!(0.0) {
                dof.disable_angular_motor();
            }
        }
    });

    testbed.add_callback(move |_, _, bodies, _, _, _| {
        /*
         * Activate the pin-slot joint linear motor if it is to low.
         */
        // Might be None if the user interactively deleted the pin-slot body.
        let link = bodies
            .multibody_mut(pin_slot_handle)
            .and_then(|mb| mb.link_mut(0));
        if let Some(pin_slot) = link {
            let dof = pin_slot
                .joint_mut()
                .downcast_mut::<PinSlotJoint<N>>()
                .unwrap();

            if dof.offset() < r!(-10.0) {
                dof.enable_linear_motor();
            } else if dof.offset() > r!(-4.0) {
                dof.disable_linear_motor();
            }
        }
    });

    // NOTE: we add another static body to the scene. It is not necessary for our simulation
    // but this is required so that we can call `testbed.set_ground_handle` which will
    // enable the testbed's feature that lets us grab an object with the mouse.
    let ground_handle = bodies.insert(Ground::new());
    testbed.set_ground_handle(Some(ground_handle));
    testbed.set_world(
        mechanical_world,
        geometrical_world,
        bodies,
        colliders,
        joint_constraints,
        force_generators,
    );
    testbed.look_at(Point3::new(30.0, -2.0, 0.0), Point3::new(0.0, -2.0, 0.0));
}

fn main() {
    let testbed = Testbed::<f32>::from_builders(0, vec![("Multibody", init_world)]);
    testbed.run()
}
