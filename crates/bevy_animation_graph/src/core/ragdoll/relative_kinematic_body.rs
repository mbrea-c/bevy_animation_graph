use avian3d::prelude::RigidBody;
use bevy::{
    ecs::{component::Component, entity::Entity},
    math::{Isometry3d, Vec3},
    reflect::Reflect,
};

/// A physics-simulated body that has two components of motion:
/// * A component that is relative to a dynamic body
/// * A component that is kinematically-driven
#[derive(Component, Default, Debug, Clone, Reflect)]
#[require(RigidBody = RigidBody::Kinematic)]
pub struct RelativeKinematicBody {
    /// This entity should have `LinearVelocity` and `AngularVelocity`
    pub relative_to: Option<Entity>,
    pub kinematic_linear_velocity: Vec3,
    pub kinematic_angular_velocity: Vec3,
}

/// A physics-simulated body that has two components of motion:
/// * A component that is relative to a dynamic body
/// * A component that is kinematically-driven
#[derive(Component, Default, Debug, Clone, Reflect)]
#[require(RelativeKinematicBody)]
pub struct RelativeKinematicBodyPositionBased {
    /// This entity should have `LinearVelocity` and `AngularVelocity`
    pub relative_to: Option<Entity>,
    pub relative_target: Isometry3d,
}
