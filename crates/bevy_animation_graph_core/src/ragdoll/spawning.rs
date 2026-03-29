#[cfg(feature = "physics_avian")]
use bevy::{ecs::system::Commands, math::Isometry3d};
use bevy::{
    ecs::{entity::Entity, event::EntityEvent},
    platform::collections::HashMap,
    reflect::Reflect,
};

#[cfg(feature = "physics_avian")]
use crate::ragdoll::definition::Ragdoll;
use crate::ragdoll::definition::{BodyId, ColliderId, JointId};

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

#[derive(EntityEvent, Reflect)]
pub struct RagdollSpawned {
    #[event_target]
    pub ragdoll: Entity,
    pub animation_player: Entity,
}

#[derive(EntityEvent, Reflect)]
pub struct RagdollColliderSpawned {
    #[event_target]
    pub collider: Entity,
    pub ragdoll: Entity,
    pub animation_player: Entity,
}

#[cfg(feature = "physics_avian")]
pub fn spawn_ragdoll_avian(
    player_entity: Entity,
    ragdoll: &Ragdoll,
    root_pos: Isometry3d,
    simulated_parent: Option<Entity>,
    commands: &mut Commands,
) -> SpawnedRagdoll {
    use avian3d::prelude::{AngleLimit, CollisionLayers, RevoluteJoint, RigidBody, SphericalJoint};
    use bevy::{ecs::name::Name, math::Vec3, transform::components::Transform, utils::default};

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

    let mut total_shared_mass_volume = 0.;

    for body in ragdoll.bodies.values() {
        for collider_id in body.colliders.iter() {
            use crate::ragdoll::definition::ColliderMassMode;

            let Some(collider) = ragdoll.colliders.get(collider_id) else {
                continue;
            };

            if let ColliderMassMode::ByVolume = &collider.mass_mode {
                total_shared_mass_volume += collider.volume();
            }
        }
    }

    for body in ragdoll.bodies.values() {
        use crate::ragdoll::{
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
            use avian3d::prelude::Mass;

            use crate::ragdoll::definition::{ColliderLabel, ColliderMassMode};

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
                    Mass(match &collider.mass_mode {
                        ColliderMassMode::Override(val) => *val,
                        ColliderMassMode::ByVolume => {
                            ragdoll.total_mass * collider.volume() / total_shared_mass_volume
                        }
                    }),
                ))
                .id();
            commands.entity(body_entity).add_child(collider_entity);
            spawned.colliders.insert(collider.id, collider_entity);

            commands.trigger(RagdollColliderSpawned {
                collider: collider_entity,
                ragdoll: root,
                animation_player: player_entity,
            });
        }
    }

    let make_offset = |body_id: BodyId, joint_pos: Vec3| {
        use avian3d::prelude::JointFrame;

        let body = ragdoll
            .bodies
            .get(&body_id)
            .expect("Validation should have caught this");

        JointFrame::local(joint_pos - body.offset)
    };

    for joint in ragdoll.joints.values() {
        use avian3d::prelude::JointCollisionDisabled;

        use crate::ragdoll::definition::{JointLabel, JointVariant};

        let joint_entity = match &joint.variant {
            JointVariant::Spherical(spherical_joint) => {
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
                            frame1: make_offset(spherical_joint.body1, spherical_joint.position),
                            frame2: make_offset(spherical_joint.body2, spherical_joint.position),
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
            JointVariant::Revolute(revolute_joint) => commands
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
                        frame1: make_offset(revolute_joint.body1, revolute_joint.position),
                        frame2: make_offset(revolute_joint.body2, revolute_joint.position),
                        hinge_axis: revolute_joint.hinge_axis,
                        angle_limit: revolute_joint.angle_limit.as_ref().map(|limit| AngleLimit {
                            min: limit.min,
                            max: limit.max,
                        }),
                        point_compliance: revolute_joint.point_compliance,
                        align_compliance: revolute_joint.align_compliance,
                        limit_compliance: revolute_joint.limit_compliance,
                    },
                ))
                .id(),
        };
        commands
            .entity(joint_entity)
            .insert((JointLabel(joint.label.clone()), JointCollisionDisabled));
        commands.entity(root).add_child(joint_entity);
        spawned.joints.insert(joint.id, joint_entity);

        commands.trigger(RagdollSpawned {
            ragdoll: root,
            animation_player: player_entity,
        });
    }

    spawned
}
