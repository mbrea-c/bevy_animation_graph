use std::sync::Arc;

use bevy::{
    asset::Handle,
    color::{Color, LinearRgba, palettes::css},
    core_pipeline::core_3d::Camera3d,
    ecs::{hierarchy::ChildSpawnerCommands, world::World},
    gizmos::primitives::dim3::GizmoPrimitive3d,
    image::Image,
    math::{Isometry3d, Vec3},
    pbr::PointLight,
    platform::collections::HashMap,
    render::camera::{Camera, ClearColorConfig, RenderTarget},
    transform::components::Transform,
    utils::default,
};
use bevy_animation_graph::{
    core::{
        colliders::core::ColliderShape,
        ragdoll::definition::{
            Body, BodyId, Collider, ColliderId, Joint, JointId, JointVariant, Ragdoll,
        },
    },
    prelude::{AnimatedScene, AnimatedSceneHandle, AnimationGraphPlayer, AnimationSource},
};

use crate::ui::{
    PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
    core::EditorWindowContext,
    utils::{
        OrbitView, orbit_camera_scene_show, orbit_camera_transform, orbit_camera_update,
        with_assets_all,
    },
};

pub struct RagdollPreview<'a, 'b> {
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
    pub orbit_view: &'a mut OrbitView,
    pub ragdoll: Handle<Ragdoll>,
    pub base_scene: Handle<AnimatedScene>,
    pub body_buffers: HashMap<BodyId, Body>,
    pub collider_buffers: HashMap<ColliderId, Collider>,
    pub joint_buffers: HashMap<JointId, Joint>,
}

impl RagdollPreview<'_, '_> {
    pub fn draw(self, ui: &mut egui::Ui) {
        let config = RagdollPreviewConfig {
            animated_scene: self.base_scene.clone(),
            view: self.orbit_view.clone(),
            gizmo_overlays: vec![
                Arc::new(RagdollBodies {
                    ragdoll: self.ragdoll.clone(),
                    body_buffers: self.body_buffers.clone(),
                }),
                Arc::new(RagdollColliders {
                    ragdoll: self.ragdoll.clone(),
                    body_buffers: self.body_buffers.clone(),
                    collider_buffers: self.collider_buffers,
                }),
                Arc::new(RagdollJoints {
                    ragdoll: self.ragdoll.clone(),
                    body_buffers: self.body_buffers,
                    joint_buffers: self.joint_buffers,
                }),
            ],
        };

        let ui_texture_id = ui.id().with("clip preview texture");
        orbit_camera_scene_show(&config, self.orbit_view, ui, self.world, ui_texture_id);
    }
}

#[derive(Clone)]
pub struct RagdollPreviewConfig {
    pub animated_scene: Handle<AnimatedScene>,
    pub view: OrbitView,
    pub gizmo_overlays: Vec<Arc<dyn GizmoOverlay>>,
}

impl SubSceneConfig for RagdollPreviewConfig {
    fn spawn(&self, builder: &mut ChildSpawnerCommands, render_target: Handle<Image>) {
        builder.spawn((
            PointLight::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ));

        builder.spawn((
            Camera3d::default(),
            Camera {
                // render before the "main pass" camera
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::from(LinearRgba::new(0., 0., 0., 1.))),
                target: RenderTarget::Image(render_target.into()),
                ..default()
            },
            orbit_camera_transform(&self.view),
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
        world
            .run_system_cached_with(orbit_camera_update, (id, self.view.clone()))
            .unwrap();
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
    let query_world = unsafe { world_cell.clone().world_mut() };
    let overlay_world = unsafe { world_cell.clone().world_mut() };
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

pub struct RagdollBodies {
    pub ragdoll: Handle<Ragdoll>,
    pub body_buffers: HashMap<BodyId, Body>,
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
                player.gizmo_relative_to_root(move |root_transform, gizmos| {
                    gizmos.axes(root_transform * Transform::from_translation(offset), 0.1);
                });
            }
        });
    }
}

pub struct RagdollColliders {
    pub ragdoll: Handle<Ragdoll>,
    pub body_buffers: HashMap<BodyId, Body>,
    pub collider_buffers: HashMap<ColliderId, Collider>,
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

                    player.gizmo_relative_to_root(move |root_transform, gizmos| match shape {
                        ColliderShape::Sphere(sphere) => {
                            gizmos.primitive_3d(
                                &sphere,
                                root_transform.to_isometry() * isometry,
                                css::ORANGE,
                            );
                        }
                        ColliderShape::Capsule(capsule3d) => {
                            gizmos.primitive_3d(
                                &capsule3d,
                                root_transform.to_isometry() * isometry,
                                css::ORANGE,
                            );
                        }
                        ColliderShape::Cuboid(cuboid) => {
                            gizmos.primitive_3d(
                                &cuboid,
                                root_transform.to_isometry() * isometry,
                                css::ORANGE,
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
                        if let Some(body1) = self
                            .body_buffers
                            .get(&spherical_joint.body1)
                            .or(ragdoll.get_body(spherical_joint.body1))
                            && let Some(body2) = self
                                .body_buffers
                                .get(&spherical_joint.body2)
                                .or(ragdoll.get_body(spherical_joint.body2))
                        {
                            let char_anchor_1 = body1.offset + spherical_joint.local_anchor1;
                            let char_anchor_2 = body2.offset + spherical_joint.local_anchor2;

                            player.gizmo_relative_to_root(move |root_transform, gizmos| {
                                gizmos.line(
                                    root_transform * char_anchor_1,
                                    root_transform * char_anchor_2,
                                    css::PURPLE,
                                );
                            });
                        }
                    }
                }
            }
        });
    }
}
