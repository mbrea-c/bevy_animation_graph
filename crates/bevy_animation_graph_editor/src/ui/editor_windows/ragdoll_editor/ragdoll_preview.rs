use std::sync::Arc;

use bevy::{
    asset::Handle,
    color::{Alpha, Hsla, palettes::css},
    ecs::{hierarchy::ChildSpawnerCommands, world::World},
    gizmos::primitives::dim3::GizmoPrimitive3d,
    image::Image,
    light::PointLight,
    math::{Isometry3d, Quat, Vec3},
    platform::collections::HashMap,
    transform::components::Transform,
};
use bevy_animation_graph::{
    core::{
        id::BoneId,
        ragdoll::definition::{
            Body, BodyId, Collider, ColliderId, ColliderShape, Joint, JointId, JointVariant,
            Ragdoll,
        },
        skeleton::Skeleton,
    },
    prelude::{AnimatedScene, AnimatedSceneHandle, AnimationGraphPlayer, AnimationSource},
};

use crate::ui::{
    PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
    editor_windows::ragdoll_editor::SelectedItem,
    utils::{orbit_camera_scene_show, with_assets_all},
};

pub struct RagdollPreview<'a> {
    pub world: &'a mut World,

    pub ragdoll: Handle<Ragdoll>,
    pub base_scene: Handle<AnimatedScene>,

    pub body_buffers: HashMap<BodyId, Body>,
    pub collider_buffers: HashMap<ColliderId, Collider>,
    pub joint_buffers: HashMap<JointId, Joint>,

    pub hovered_item: Option<SelectedItem>,
    pub selected_item: Option<SelectedItem>,
}

impl RagdollPreview<'_> {
    pub fn draw(self, ui: &mut egui::Ui) {
        let config = RagdollPreviewConfig {
            animated_scene: self.base_scene.clone(),
            gizmo_overlays: vec![
                Arc::new(RagdollBodies {
                    ragdoll: self.ragdoll.clone(),
                    body_buffers: self.body_buffers.clone(),
                    hover: self.hovered_item.clone().and_then(|i| i.body()),
                    selected: self.selected_item.clone().and_then(|i| i.body()),
                }),
                Arc::new(RagdollColliders {
                    ragdoll: self.ragdoll.clone(),
                    body_buffers: self.body_buffers.clone(),
                    collider_buffers: self.collider_buffers,
                    hovered: self.hovered_item.clone().and_then(|i| i.collider()),
                    selected: self.selected_item.clone().and_then(|i| i.collider()),
                }),
                Arc::new(RagdollJoints {
                    ragdoll: self.ragdoll.clone(),
                    body_buffers: self.body_buffers,
                    joint_buffers: self.joint_buffers,
                    hovered: self.hovered_item.clone().and_then(|i| i.joint()),
                    selected: self.selected_item.clone().and_then(|i| i.joint()),
                }),
                Arc::new(SkeletonBones {
                    skeleton: with_assets_all(
                        self.world,
                        [self.base_scene.id()],
                        |_, [base_scene]| base_scene.skeleton.clone(),
                    )
                    .unwrap(),
                    hovered: self.hovered_item.clone().and_then(|i| i.bone()),
                    selected: self.selected_item.clone().and_then(|i| i.bone()),
                }),
            ],
        };

        let ui_texture_id = ui.id().with("clip preview texture");
        orbit_camera_scene_show(&config, ui, self.world, ui_texture_id);
    }
}

#[derive(Clone)]
pub struct RagdollPreviewConfig {
    pub animated_scene: Handle<AnimatedScene>,
    pub gizmo_overlays: Vec<Arc<dyn GizmoOverlay>>,
}

impl SubSceneConfig for RagdollPreviewConfig {
    fn spawn(&self, builder: &mut ChildSpawnerCommands, _: Handle<Image>) {
        builder.spawn((
            PointLight::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ));

        builder.spawn((
            AnimatedSceneHandle {
                handle: self.animated_scene.clone(),
                override_source: Some(AnimationSource::None),
            },
            PreviewScene,
        ));
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        match self.animated_scene == new_config.animated_scene {
            true => SubSceneSyncAction::Update,
            false => SubSceneSyncAction::Respawn,
        }
    }

    fn update(&self, id: egui::Id, world: &mut World) {
        for overlay in &self.gizmo_overlays {
            draw_gizmo_overlay(id, overlay, world);
        }
    }
}

fn draw_gizmo_overlay(id: egui::Id, input: &Arc<dyn GizmoOverlay>, world: &mut World) {
    let world_cell = world.as_unsafe_world_cell();
    // SAFETY: Only safe as long as the overlay does not access anything conflicting with the query
    // we're getting here. Unfortunately I could not think of a safer way to solve this atm.
    // I'm only letting this through because this is not library code, just editor code.
    let query_world = unsafe { world_cell.world_mut() };
    let overlay_world = unsafe { world_cell.world_mut() };
    let mut query = query_world.query::<(&mut AnimationGraphPlayer, &PartOfSubScene)>();
    for (mut player, PartOfSubScene(target_id)) in query.iter_mut(query_world) {
        if id != *target_id {
            continue;
        }

        input.draw(overlay_world, player.as_mut());
    }
}

pub trait GizmoOverlay: Send + Sync + 'static {
    fn draw(&self, world: &mut World, player: &mut AnimationGraphPlayer);
}

const SYMMETRY_ALPHA: f32 = 0.2;

pub struct RagdollBodies {
    pub ragdoll: Handle<Ragdoll>,
    pub body_buffers: HashMap<BodyId, Body>,
    pub hover: Option<BodyId>,
    #[allow(dead_code)]
    pub selected: Option<BodyId>,
}

impl GizmoOverlay for RagdollBodies {
    fn draw(&self, world: &mut World, player: &mut AnimationGraphPlayer) {
        with_assets_all(world, [self.ragdoll.id()], |_, [ragdoll]| {
            for ragdoll_body in ragdoll.bodies.values() {
                let body = self
                    .body_buffers
                    .get(&ragdoll_body.id)
                    .unwrap_or(ragdoll_body);
                let offset = body.offset;

                let alpha = if body.created_from.is_none() {
                    1.
                } else {
                    SYMMETRY_ALPHA
                };

                let length = if self.hover.is_some_and(|id| id == body.id) {
                    0.1
                } else {
                    0.05
                };

                player.gizmo_relative_to_root(move |root_transform, gizmos| {
                    let t = root_transform * Transform::from_translation(offset);
                    let x = t * (Vec3::X * length);
                    let y = t * (Vec3::Y * length);
                    let z = t * (Vec3::Z * length);
                    gizmos.line(t.translation, x, css::RED.with_alpha(alpha));
                    gizmos.line(t.translation, y, css::GREEN.with_alpha(alpha));
                    gizmos.line(t.translation, z, css::BLUE.with_alpha(alpha));
                });
            }
        });
    }
}

pub struct RagdollColliders {
    pub ragdoll: Handle<Ragdoll>,
    pub body_buffers: HashMap<BodyId, Body>,
    pub collider_buffers: HashMap<ColliderId, Collider>,
    pub hovered: Option<ColliderId>,
    pub selected: Option<ColliderId>,
}

impl GizmoOverlay for RagdollColliders {
    fn draw(&self, world: &mut World, player: &mut AnimationGraphPlayer) {
        with_assets_all(world, [self.ragdoll.id()], |_, [ragdoll]| {
            for ragdoll_body in ragdoll.iter_bodies() {
                let body = self
                    .body_buffers
                    .get(&ragdoll_body.id)
                    .unwrap_or(ragdoll_body);
                for collider_id in ragdoll_body.colliders.iter() {
                    let Some(collider) = self
                        .collider_buffers
                        .get(collider_id)
                        .or(ragdoll.get_collider(*collider_id))
                    else {
                        continue;
                    };

                    let isometry =
                        Isometry3d::from_translation(body.offset) * collider.local_offset;
                    let shape = collider.shape.clone();

                    let is_hovered = self.hovered.is_some_and(|h| h == *collider_id);
                    let is_selected = self.selected.is_some_and(|h| h == *collider_id);

                    let hsla_orange = Hsla::from(css::ORANGE);

                    let base_color = if is_hovered {
                        hsla_orange
                            .with_lightness((hsla_orange.lightness + 0.2).min(1.))
                            .into()
                    } else if is_selected {
                        css::ORANGE
                    } else {
                        hsla_orange
                            .with_lightness((hsla_orange.lightness - 0.1).max(0.))
                            .into()
                    };

                    let color = if collider.created_from.is_none() {
                        base_color
                    } else {
                        base_color.with_alpha(SYMMETRY_ALPHA)
                    };

                    player.gizmo_relative_to_root(move |root_transform, gizmos| match shape {
                        ColliderShape::Sphere(sphere) => {
                            gizmos.primitive_3d(
                                &sphere,
                                root_transform.to_isometry() * isometry,
                                color,
                            );
                        }
                        ColliderShape::Capsule(capsule3d) => {
                            gizmos.primitive_3d(
                                &capsule3d,
                                root_transform.to_isometry() * isometry,
                                color,
                            );
                        }
                        ColliderShape::Cuboid(cuboid) => {
                            gizmos.primitive_3d(
                                &cuboid,
                                root_transform.to_isometry() * isometry,
                                color,
                            );
                        }
                    });
                }
            }
        });
    }
}

pub struct RagdollJoints {
    pub ragdoll: Handle<Ragdoll>,
    pub body_buffers: HashMap<BodyId, Body>,
    pub joint_buffers: HashMap<JointId, Joint>,
    pub hovered: Option<JointId>,
    pub selected: Option<JointId>,
}

impl GizmoOverlay for RagdollJoints {
    fn draw(&self, world: &mut World, player: &mut AnimationGraphPlayer) {
        with_assets_all(world, [self.ragdoll.id()], |_, [ragdoll]| {
            for ragdoll_joint in ragdoll.iter_joints() {
                let joint = self
                    .joint_buffers
                    .get(&ragdoll_joint.id)
                    .unwrap_or(ragdoll_joint);
                match &joint.variant {
                    JointVariant::Spherical(spherical_joint) => {
                        if let Some(_) = self
                            .body_buffers
                            .get(&spherical_joint.body1)
                            .or(ragdoll.get_body(spherical_joint.body1))
                            && let Some(_) = self
                                .body_buffers
                                .get(&spherical_joint.body2)
                                .or(ragdoll.get_body(spherical_joint.body2))
                        {
                            let twist_axis = spherical_joint.twist_axis;
                            let jointpos = spherical_joint.position;

                            let is_hovered = self.hovered.is_some_and(|h| h == joint.id);
                            let is_selected = self.selected.is_some_and(|h| h == joint.id);

                            let hsla_purple = Hsla::from(css::PURPLE);

                            let base_color = if is_hovered {
                                hsla_purple
                                    .with_lightness((hsla_purple.lightness + 0.2).min(1.))
                                    .into()
                            } else if is_selected {
                                css::PURPLE
                            } else {
                                hsla_purple
                                    .with_lightness((hsla_purple.lightness - 0.1).max(0.))
                                    .into()
                            };

                            let color = if joint.created_from.is_none() {
                                base_color
                            } else {
                                base_color.with_alpha(SYMMETRY_ALPHA)
                            };

                            player.gizmo_relative_to_root(move |root_transform, gizmos| {
                                gizmos.arrow(
                                    root_transform * jointpos,
                                    root_transform * jointpos
                                        + twist_axis.normalize_or_zero() * 0.1,
                                    color,
                                );
                                gizmos.circle(
                                    Isometry3d {
                                        rotation: Quat::from_rotation_arc(Vec3::Z, twist_axis),
                                        translation: (root_transform * jointpos).into(),
                                    },
                                    0.05,
                                    color,
                                );
                            });
                        }
                    }
                    JointVariant::Revolute(revolute_joint) => {
                        if let Some(_) = self
                            .body_buffers
                            .get(&revolute_joint.body1)
                            .or(ragdoll.get_body(revolute_joint.body1))
                            && let Some(_) = self
                                .body_buffers
                                .get(&revolute_joint.body2)
                                .or(ragdoll.get_body(revolute_joint.body2))
                        {
                            let hinge_axis = revolute_joint.hinge_axis;
                            let jointpos = revolute_joint.position;

                            let is_hovered = self.hovered.is_some_and(|h| h == joint.id);
                            let is_selected = self.selected.is_some_and(|h| h == joint.id);

                            let hsla_purple = Hsla::from(css::PURPLE);

                            let base_color = if is_hovered {
                                hsla_purple
                                    .with_lightness((hsla_purple.lightness + 0.2).min(1.))
                                    .into()
                            } else if is_selected {
                                css::PURPLE
                            } else {
                                hsla_purple
                                    .with_lightness((hsla_purple.lightness - 0.1).max(0.))
                                    .into()
                            };

                            let color = if joint.created_from.is_none() {
                                base_color
                            } else {
                                base_color.with_alpha(SYMMETRY_ALPHA)
                            };

                            player.gizmo_relative_to_root(move |root_transform, gizmos| {
                                gizmos.arrow(
                                    root_transform * jointpos,
                                    root_transform * jointpos
                                        + hinge_axis.normalize_or_zero() * 0.1,
                                    color,
                                );
                                gizmos.circle(
                                    Isometry3d {
                                        rotation: Quat::from_rotation_arc(Vec3::Z, hinge_axis),
                                        translation: (root_transform * jointpos).into(),
                                    },
                                    0.05,
                                    color,
                                );
                            });
                        }
                    }
                }
            }
        });
    }
}

pub struct SkeletonBones {
    pub skeleton: Handle<Skeleton>,
    pub hovered: Option<BoneId>,
    pub selected: Option<BoneId>,
}

impl GizmoOverlay for SkeletonBones {
    fn draw(&self, world: &mut World, player: &mut AnimationGraphPlayer) {
        with_assets_all(world, [self.skeleton.id()], |_, [skeleton]| {
            for bone_id in skeleton.iter_bones() {
                let is_hovered = self.hovered.is_some_and(|h| h == bone_id);
                let is_selected = self.selected.is_some_and(|h| h == bone_id);

                let color = if is_hovered {
                    css::LIGHT_SKY_BLUE
                } else if is_selected {
                    css::DODGER_BLUE
                } else {
                    css::GRAY
                };

                player.gizmo_for_bones_with_color([(bone_id, color.into(), false)]);
            }
        });
    }
}
