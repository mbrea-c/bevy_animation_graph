use bevy::{
    asset::Asset,
    math::{Isometry3d, Vec3, primitives::Cuboid},
    platform::collections::HashMap,
    reflect::Reflect,
    utils::default,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::colliders::core::ColliderShape;

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Ragdoll {
    pub bodies: HashMap<BodyId, Body>,
    pub colliders: HashMap<ColliderId, Collider>,
    pub joints: HashMap<JointId, Joint>,
}

impl Ragdoll {
    pub fn get_body(&self, id: BodyId) -> Option<&Body> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: BodyId) -> Option<&mut Body> {
        self.bodies.get_mut(&id)
    }

    ///  Adds new body to the ragdoll. This operation is idempotent; if you try to add a body with
    ///  an ID that already exists, it's ignored.
    pub fn add_body(&mut self, body: Body) {
        if !self.bodies.contains_key(&body.id) {
            self.bodies.insert(body.id, body);
        }
    }

    pub fn get_collider(&self, id: ColliderId) -> Option<&Collider> {
        self.colliders.get(&id)
    }

    pub fn get_collider_mut(&mut self, id: ColliderId) -> Option<&mut Collider> {
        self.colliders.get_mut(&id)
    }

    pub fn get_joint(&self, id: JointId) -> Option<&Joint> {
        self.joints.get(&id)
    }

    pub fn get_joint_mut(&mut self, id: JointId) -> Option<&mut Joint> {
        self.joints.get_mut(&id)
    }

    pub fn iter_bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.values()
    }

    pub fn iter_joints(&self) -> impl Iterator<Item = &Joint> {
        self.joints.values()
    }
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub id: BodyId,
    #[serde(default)]
    pub label: String,
    pub isometry: Isometry3d,
    pub colliders: Vec<ColliderId>,
    pub default_mode: BodyMode,
}

impl Body {
    pub fn new() -> Self {
        Self {
            id: BodyId {
                uuid: Uuid::new_v4(),
            },
            label: "New body".into(),
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

impl Collider {
    pub fn new() -> Self {
        Self {
            id: ColliderId {
                uuid: Uuid::new_v4(),
            },
            local_offset: default(),
            shape: ColliderShape::Cuboid(Cuboid::new(0.2, 0.2, 0.2)),
            layer_membership: 1,
            layer_filter: 1,
            override_layers: false,
            label: "New collider".into(),
        }
    }
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub id: JointId,
    #[serde(default)]
    pub label: String,
    pub variant: JointVariant,
}

impl Joint {
    pub fn new() -> Self {
        Self {
            id: JointId {
                uuid: Uuid::new_v4(),
            },
            label: "New joint".into(),
            variant: JointVariant::Spherical(SphericalJoint::default()),
        }
    }
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JointVariant {
    Spherical(SphericalJoint),
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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

#[derive(Reflect, Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AngleLimit {
    pub min: f32,
    pub max: f32,
}

#[derive(Default, Reflect, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct BodyId {
    uuid: Uuid,
}

impl BodyId {
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self { uuid }
    }
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
