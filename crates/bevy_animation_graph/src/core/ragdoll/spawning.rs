use bevy::{ecs::entity::Entity, platform::collections::HashMap, reflect::Reflect};
#[cfg(feature = "physics_avian")]
use bevy::{ecs::system::Commands, math::Isometry3d};

#[cfg(feature = "physics_avian")]
use crate::core::ragdoll::definition::Ragdoll;
use crate::core::ragdoll::definition::{BodyId, ColliderId, JointId};

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
    use bevy::{ecs::name::Name, transform::components::Transform, utils::default};

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
        use crate::core::ragdoll::definition::{JointLabel, JointVariant};

        let joint_entity = match &joint.variant {
            JointVariant::Spherical(spherical_joint) => {
                use avian3d::prelude::JointFrame;

                commands
                    .spawn((
                        Name::new("Ragdoll joint - spherical"),
                        SphericalJoint {
                            body1: *spawned
                                .bodies
                                .get(&spherical_joint.body1)
                                .expect("Validation should have caught this"),
                            body2: *spawned
                                .bodies
                                .get(&spherical_joint.body2)
                                .expect("Validation should have caught this"),
                            frame1: JointFrame::global(spherical_joint.position),
                            frame2: JointFrame::global(spherical_joint.position),
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
                            point_compliance: spherical_joint.point_compliance,
                            swing_compliance: spherical_joint.swing_compliance,
                            twist_compliance: spherical_joint.twist_compliance,
                        },
                    ))
                    .id()
            }
            JointVariant::Revolute(revolute_joint) => {
                use avian3d::prelude::JointFrame;

                commands
                    .spawn((
                        Name::new("Ragdoll joint - revolute"),
                        RevoluteJoint {
                            body1: *spawned
                                .bodies
                                .get(&revolute_joint.body1)
                                .expect("Validation should have caught this"),
                            body2: *spawned
                                .bodies
                                .get(&revolute_joint.body2)
                                .expect("Validation should have caught this"),
                            frame1: JointFrame::global(revolute_joint.position),
                            frame2: JointFrame::global(revolute_joint.position),
                            hinge_axis: revolute_joint.hinge_axis,
                            angle_limit: revolute_joint.angle_limit.as_ref().map(|limit| {
                                AngleLimit {
                                    min: limit.min,
                                    max: limit.max,
                                }
                            }),
                            point_compliance: revolute_joint.point_compliance,
                            align_compliance: revolute_joint.align_compliance,
                            limit_compliance: revolute_joint.limit_compliance,
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
