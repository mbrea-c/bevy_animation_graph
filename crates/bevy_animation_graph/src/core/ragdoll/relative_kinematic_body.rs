use bevy::{
    ecs::{component::Component, entity::Entity},
    math::Vec3,
    reflect::Reflect,
};

/// A physics-simulated body that has two components of motion:
/// * A component that is relative to a dynamic body
/// * A component that is kinematically-driven
#[derive(Component, Debug, Clone, Reflect)]
pub struct RelativeKinematicBody {
    /// This entity should have `LinearVelocity` and `AngularVelocity`
    pub relative_to: Option<Entity>,
    pub kinematic_linear_velocity: Vec3,
    pub kinematic_angular_velocity: Vec3,
}
