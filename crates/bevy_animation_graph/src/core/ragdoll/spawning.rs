use bevy::{
    ecs::{entity::Entity, system::Commands},
    math::Isometry3d,
    platform::collections::HashMap,
    prelude::default,
    reflect::Reflect,
    transform::components::Transform,
};

use crate::core::{
    colliders::core::ColliderLabel,
    ragdoll::definition::{BodyId, ColliderId, JointId, JointVariant, Ragdoll},
};

#[derive(Reflect)]
pub struct SpawnedRagdoll {
    pub root: Entity,
    pub bodies: HashMap<BodyId, Entity>,
    pub colliders: HashMap<ColliderId, Entity>,
    pub joints: HashMap<JointId, Entity>,
}

impl SpawnedRagdoll {
    pub fn new(root: Entity) -> Self {
        Self {
            root,
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            joints: HashMap::new(),
        }
    }
}

#[cfg(feature = "physics_avian")]
pub fn spawn_ragdoll_avian(
    ragdoll: &Ragdoll,
    root_pos: Isometry3d,
    commands: &mut Commands,
) -> SpawnedRagdoll {
    use avian3d::prelude::{AngleLimit, CollisionLayers, RigidBody, SphericalJoint};

    let root = commands
        .spawn(Transform {
            translation: root_pos.translation.into(),
            rotation: root_pos.rotation,
            ..default()
        })
        .id();

    let mut spawned = SpawnedRagdoll::new(root);

    for body in &ragdoll.bodies {
        let body_entity = commands
            .spawn((
                Transform {
                    translation: body.isometry.translation.into(),
                    rotation: body.isometry.rotation,
                    ..default()
                },
                // TODO: Default should be kinematic :)
                RigidBody::Dynamic,
            ))
            .id();
        commands.entity(root).add_child(body_entity);
        spawned.bodies.insert(body.id, body_entity);

        for collider in &body.colliders {
            let collider_entity = commands
                .spawn((
                    Transform::from_isometry(collider.local_offset),
                    collider.shape.avian_collider(),
                    CollisionLayers {
                        memberships: collider.layer_membership.into(),
                        filters: collider.layer_filter.into(),
                    },
                    ColliderLabel(collider.label.clone()),
                ))
                .id();
            commands.entity(body_entity).add_child(collider_entity);
            spawned.colliders.insert(collider.id, collider_entity);
        }
    }

    for joint in &ragdoll.joints {
        let joint_entity = match &joint.variant {
            JointVariant::Spherical(spherical_joint) => commands
                .spawn(SphericalJoint {
                    entity1: *spawned
                        .bodies
                        .get(&spherical_joint.body1)
                        .expect("Validation should have caught this"),
                    entity2: *spawned
                        .bodies
                        .get(&spherical_joint.body2)
                        .expect("Validation should have caught this"),
                    local_anchor1: spherical_joint.local_anchor1,
                    local_anchor2: spherical_joint.local_anchor2,
                    swing_axis: spherical_joint.swing_axis,
                    twist_axis: spherical_joint.twist_axis,
                    swing_limit: spherical_joint
                        .swing_limit
                        .as_ref()
                        .map(|limit| AngleLimit {
                            min: limit.min,
                            max: limit.max,
                        }),
                    twist_limit: spherical_joint
                        .twist_limit
                        .as_ref()
                        .map(|limit| AngleLimit {
                            min: limit.min,
                            max: limit.max,
                        }),
                    damping_linear: spherical_joint.damping_linear,
                    damping_angular: spherical_joint.damping_angular,
                    position_lagrange: spherical_joint.position_lagrange,
                    swing_lagrange: spherical_joint.swing_lagrange,
                    twist_lagrange: spherical_joint.twist_lagrange,
                    compliance: spherical_joint.compliance,
                    force: spherical_joint.force,
                    swing_torque: spherical_joint.swing_torque,
                    twist_torque: spherical_joint.twist_torque,
                })
                .id(),
        };
        commands.entity(root).add_child(joint_entity);
        spawned.joints.insert(joint.id, joint_entity);
    }

    spawned
}
