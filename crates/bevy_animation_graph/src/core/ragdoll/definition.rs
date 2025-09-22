use bevy::{
    asset::Asset,
    math::{Isometry3d, Vec3},
    reflect::Reflect,
    utils::default,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::colliders::core::ColliderShape;

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Ragdoll {
    pub bodies: Vec<Body>,
    pub joints: Vec<Joint>,
}

impl Ragdoll {
    pub fn get_body(&self, id: BodyId) -> Option<&Body> {
        self.bodies.iter().find(|b| b.id == id)
    }

    pub fn get_collider(&self, id: ColliderId) -> Option<&Collider> {
        self.bodies
            .iter()
            .flat_map(|b| b.colliders.iter())
            .find(|c| c.id == id)
    }

    pub fn get_joint(&self, id: JointId) -> Option<&Joint> {
        self.joints.iter().find(|j| j.id == id)
    }
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub id: BodyId,
    pub isometry: Isometry3d,
    pub colliders: Vec<Collider>,
    pub default_mode: BodyMode,
}

impl Body {
    pub fn new() -> Self {
        Self {
            id: BodyId {
                uuid: Uuid::new_v4(),
            },
            isometry: default(),
            colliders: default(),
            default_mode: BodyMode::Kinematic,
        }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyMode {
    Kinematic,
    Dynamic,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub id: ColliderId,
    /// Local offset w.r.t. the rigidbody it's attached to
    pub local_offset: Isometry3d,
    pub shape: ColliderShape,
    pub layer_membership: u32,
    pub layer_filter: u32,
    pub override_layers: bool,
    /// Label that will be attached to the created collider in a [`ColliderLabel`] component.
    pub label: String,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub id: JointId,
    pub variant: JointVariant,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum JointVariant {
    Spherical(SphericalJoint),
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct SphericalJoint {
    pub body1: BodyId,
    pub body2: BodyId,
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
    pub swing_axis: Vec3,
    pub twist_axis: Vec3,
    pub swing_limit: Option<AngleLimit>,
    pub twist_limit: Option<AngleLimit>,
    pub damping_linear: f32,
    pub damping_angular: f32,
    pub position_lagrange: f32,
    pub swing_lagrange: f32,
    pub twist_lagrange: f32,
    pub compliance: f32,
    pub force: Vec3,
    pub swing_torque: Vec3,
    pub twist_torque: Vec3,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct AngleLimit {
    pub min: f32,
    pub max: f32,
}

#[derive(Reflect, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct BodyId {
    uuid: Uuid,
}

impl std::fmt::Debug for BodyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

#[derive(Reflect, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ColliderId {
    uuid: Uuid,
}

impl std::fmt::Debug for ColliderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

#[derive(Reflect, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct JointId {
    uuid: Uuid,
}

impl std::fmt::Debug for JointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}
