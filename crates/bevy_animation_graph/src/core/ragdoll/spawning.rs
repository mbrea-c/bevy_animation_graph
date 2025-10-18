use bevy::{
    ecs::{entity::Entity, system::Commands},
    math::Isometry3d,
    platform::collections::HashMap,
    prelude::default,
    reflect::Reflect,
    transform::components::Transform,
};

use crate::core::ragdoll::definition::{BodyId, ColliderId, JointId, JointVariant, Ragdoll};

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
    simulated_parent: Option<Entity>,
    commands: &mut Commands,
) -> SpawnedRagdoll {
    use avian3d::prelude::{AngleLimit, CollisionLayers, RevoluteJoint, RigidBody, SphericalJoint};
    use bevy::ecs::name::Name;

    let root = commands
        .spawn((
            Transform {
                translation: root_pos.translation.into(),
                rotation: root_pos.rotation,
                ..default()
            },
            Name::new("Ragdoll"),
        ))
        .id();

    let mut spawned = SpawnedRagdoll::new(root);

    for body in ragdoll.bodies.values() {
        use crate::core::ragdoll::{
            definition::{BodyLabel, BodyMode},
            relative_kinematic_body::RelativeKinematicBodyPositionBased,
        };

        let body_entity = commands
            .spawn((
                Name::new("Ragdoll body"),
                Transform {
                    translation: body.offset,
                    ..default()
                },
                match body.default_mode {
                    BodyMode::Kinematic => RigidBody::Kinematic,
                    BodyMode::Dynamic => RigidBody::Dynamic,
                },
                RelativeKinematicBodyPositionBased {
                    relative_to: simulated_parent,
                    ..default()
                },
                BodyLabel(body.label.clone()),
            ))
            .id();

        commands.entity(root).add_child(body_entity);
        spawned.bodies.insert(body.id, body_entity);

        for collider_id in &body.colliders {
            use crate::core::ragdoll::definition::ColliderLabel;

            let Some(collider) = ragdoll.get_collider(*collider_id) else {
                continue;
            };
            let collider_entity = commands
                .spawn((
                    Name::new("Ragdoll collider"),
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

    for joint in ragdoll.joints.values() {
        use crate::core::ragdoll::definition::JointLabel;

        let joint_entity = match &joint.variant {
            JointVariant::Spherical(spherical_joint) => {
                let Some(body1) = ragdoll.get_body(spherical_joint.body1) else {
                    continue;
                };

                let Some(body2) = ragdoll.get_body(spherical_joint.body2) else {
                    continue;
                };

                let local_anchor1 = spherical_joint.position - body1.offset;
                let local_anchor2 = spherical_joint.position - body2.offset;

                commands
                    .spawn((
                        Name::new("Ragdoll joint - spherical"),
                        SphericalJoint {
                            entity1: *spawned
                                .bodies
                                .get(&spherical_joint.body1)
                                .expect("Validation should have caught this"),
                            entity2: *spawned
                                .bodies
                                .get(&spherical_joint.body2)
                                .expect("Validation should have caught this"),
                            local_anchor1,
                            local_anchor2,
                            swing_axis: spherical_joint.swing_axis,
                            twist_axis: spherical_joint.twist_axis,
                            swing_limit: spherical_joint.swing_limit.as_ref().map(|limit| {
                                AngleLimit {
                                    min: limit.min,
                                    max: limit.max,
                                }
                            }),
                            twist_limit: spherical_joint.twist_limit.as_ref().map(|limit| {
                                AngleLimit {
                                    min: limit.min,
                                    max: limit.max,
                                }
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
                        },
                    ))
                    .id()
            }
            JointVariant::Revolute(revolute_joint) => {
                let Some(body1) = ragdoll.get_body(revolute_joint.body1) else {
                    continue;
                };

                let Some(body2) = ragdoll.get_body(revolute_joint.body2) else {
                    continue;
                };

                let local_anchor1 = revolute_joint.position - body1.offset;
                let local_anchor2 = revolute_joint.position - body2.offset;

                commands
                    .spawn((
                        Name::new("Ragdoll joint - revolute"),
                        RevoluteJoint {
                            entity1: *spawned
                                .bodies
                                .get(&revolute_joint.body1)
                                .expect("Validation should have caught this"),
                            entity2: *spawned
                                .bodies
                                .get(&revolute_joint.body2)
                                .expect("Validation should have caught this"),
                            local_anchor1,
                            local_anchor2,
                            aligned_axis: revolute_joint.aligned_axis,
                            angle_limit: revolute_joint.angle_limit.as_ref().map(|limit| {
                                AngleLimit {
                                    min: limit.min,
                                    max: limit.max,
                                }
                            }),
                            align_lagrange: revolute_joint.align_lagrange,
                            angle_limit_lagrange: revolute_joint.angle_limit_lagrange,
                            align_torque: revolute_joint.align_torque,
                            angle_limit_torque: revolute_joint.angle_limit_torque,
                            damping_linear: revolute_joint.damping_linear,
                            damping_angular: revolute_joint.damping_angular,
                            position_lagrange: revolute_joint.position_lagrange,
                            compliance: revolute_joint.compliance,
                            force: revolute_joint.force,
                        },
                    ))
                    .id()
            }
        };
        commands
            .entity(joint_entity)
            .insert(JointLabel(joint.label.clone()));
        commands.entity(root).add_child(joint_entity);
        spawned.joints.insert(joint.id, joint_entity);
    }

    spawned
}
