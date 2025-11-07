use std::f32::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

use crate::tree::{TreeInternal, TreeResult};
use crate::ui::SubSceneSyncAction;
use bevy::asset::UntypedAssetId;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::Extent3d;
use bevy_animation_graph::core::animation_graph::{AnimationGraph, NodeId, PinMap};
use bevy_animation_graph::core::context::SpecContext;
use bevy_animation_graph::core::state_machine::high_level::StateMachine;
use bevy_animation_graph::prelude::{AnimatedSceneInstance, AnimationGraphPlayer, DataSpec};
use bevy_inspector_egui::bevy_egui::EguiUserTextures;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};

use super::{PartOfSubScene, PreviewScene, SubSceneConfig, provide_texture_for_scene};

pub mod collapsing;
pub mod popup;

pub fn asset_sort_key<T: Asset>(asset_id: AssetId<T>, asset_server: &AssetServer) -> String {
    format!(
        "{} {}",
        asset_server
            .get_path(asset_id)
            .map_or("zzzzz".to_string(), |p| p.path().to_string_lossy().into()),
        asset_id
    )
}

pub(crate) fn get_node_output_data_pins(
    world: &mut World,
    graph_id: AssetId<AnimationGraph>,
    node_id: &NodeId,
) -> Option<PinMap<DataSpec>> {
    world.resource_scope::<Assets<AnimationGraph>, _>(|world, graph_assets| {
        world.resource_scope::<Assets<StateMachine>, _>(|_, fsm_assets| {
            let graph = graph_assets.get(graph_id).unwrap();
            let spec_context = SpecContext {
                graph_assets: &graph_assets,
                fsm_assets: &fsm_assets,
            };
            let node = graph.nodes.get(node_id)?;
            Some(node.inner.data_output_spec(spec_context))
        })
    })
}

pub(crate) fn select_from_branches<I, L>(
    ui: &mut egui::Ui,
    branches: Vec<TreeInternal<I, L>>,
) -> TreeResult<I, L> {
    let mut res = TreeResult::None;

    for branch in branches {
        res = res.or(select_from_tree_internal(ui, branch));
    }

    res
}

pub(crate) fn select_from_tree_internal<I, L>(
    ui: &mut egui::Ui,
    tree: TreeInternal<I, L>,
) -> TreeResult<I, L> {
    match tree {
        TreeInternal::Leaf(name, val) => {
            if ui.selectable_label(false, name).clicked() {
                TreeResult::Leaf(val, ())
            } else {
                TreeResult::None
            }
        }
        TreeInternal::Node(name, val, subtree) => {
            let res = ui.collapsing(name, |ui| select_from_branches(ui, subtree));
            if res.header_response.clicked() {
                TreeResult::Node(val, ())
            } else {
                TreeResult::None
            }
            .or(res.body_returned.unwrap_or(TreeResult::None))
            //.body_returned
            //.flatten(),
        }
    }
}

pub(crate) fn get_specific_animation_graph_player(
    world: &World,
    entity: Entity,
) -> Option<&AnimationGraphPlayer> {
    let mut query = world.try_query::<&AnimationGraphPlayer>()?;
    query.get(world, entity).ok()
}

pub(crate) fn iter_animation_graph_players(world: &World) -> Vec<(Entity, &AnimationGraphPlayer)> {
    let mut scene_query = world
        .try_query::<(&AnimatedSceneInstance, &PreviewScene)>()
        .unwrap();
    let mut player_query = world
        .try_query::<(Entity, &AnimationGraphPlayer)>()
        .unwrap();

    scene_query
        .iter(world)
        .filter_map(|(instance, _)| {
            let entity = instance.player_entity();
            player_query.get(world, entity).ok()
        })
        .collect()
}

pub(crate) fn get_animation_graph_player_mut(
    world: &mut World,
) -> Option<&mut AnimationGraphPlayer> {
    let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
    let Ok((instance, _)) = query.single(world) else {
        return None;
    };
    let entity = instance.player_entity();
    let mut query = world.query::<&mut AnimationGraphPlayer>();
    query
        .get_mut(world, entity)
        .ok()
        .map(|player| player.into_inner())
}

pub fn handle_path(handle: UntypedAssetId, asset_server: &AssetServer) -> PathBuf {
    asset_server
        .get_path(handle)
        .map_or("Unsaved Asset".into(), |p| p.path().to_owned())
}

pub fn render_image(ui: &mut egui::Ui, world: &mut World, image: &Handle<Image>) -> egui::Response {
    let texture_id =
        world.resource_scope::<EguiUserTextures, egui::TextureId>(|_, user_textures| {
            user_textures.image_id(image).unwrap()
        });

    let available_size = ui.available_size();

    let e3d_size = Extent3d {
        width: available_size.x as u32,
        height: available_size.y as u32,
        ..default()
    };
    world.resource_scope::<Assets<Image>, ()>(|_, mut images| {
        let image = images.get_mut(image).unwrap();
        image.texture_descriptor.size = e3d_size;
        image.resize(e3d_size);
    });

    let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

    // Ensure interaction
    let response = response.interact(egui::Sense::drag());

    ui.put(rect, |ui: &mut egui::Ui| {
        ui.image(egui::load::SizedTexture::new(texture_id, available_size))
    });

    response
}

pub fn using_inspector_env<T>(world: &mut World, expr: impl FnOnce(InspectorUi) -> T) -> T {
    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };

    let env = InspectorUi::for_bevy(&type_registry, &mut cx);

    expr(env)
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrbitView {
    pub angles: Vec2,
    pub distance: f32,
}

impl Default for OrbitView {
    fn default() -> Self {
        Self {
            distance: 3.,
            angles: Vec2::ZERO,
        }
    }
}

#[derive(Clone)]
pub struct OrbitCameraSceneConfig<T> {
    pub view: OrbitView,
    pub inner: T,
}

impl<T: SubSceneConfig> SubSceneConfig for OrbitCameraSceneConfig<T> {
    fn spawn(&self, builder: &mut ChildSpawnerCommands, render_target: Handle<Image>) {
        builder.spawn((
            Camera3d::default(),
            Camera {
                // render before the "main pass" camera
                order: -1,
                clear_color: ClearColorConfig::Custom(LinearRgba::new(1.0, 1.0, 1.0, 0.0).into()),
                target: RenderTarget::Image(render_target.clone().into()),
                ..default()
            },
            // Position based on orbit camera parameters
            orbit_camera_transform(&self.view),
        ));

        self.inner.spawn(builder, render_target);
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        let outer = if self.view == new_config.view {
            SubSceneSyncAction::Nothing
        } else {
            SubSceneSyncAction::Update
        };

        outer | self.inner.sync_action(&new_config.inner)
    }

    fn update(&self, id: egui::Id, world: &mut World) {
        world
            .run_system_cached_with(orbit_camera_update, (id, self.view.clone()))
            .unwrap();
        self.inner.update(id, world);
    }
}

pub fn orbit_camera_scene_show<T: SubSceneConfig>(
    config: &T,
    ui: &mut egui::Ui,
    world: &mut World,
    id: egui::Id,
) {
    let orbit_id = ui.id().with("orbit");
    let mut orbit =
        ui.memory_mut(|mem| mem.data.get_temp::<OrbitView>(orbit_id).unwrap_or_default());

    let orbit_config = OrbitCameraSceneConfig {
        view: orbit.clone(),
        inner: config.clone(),
    };

    // First we need to make sure the subscene is created and get the camera image handle
    let texture = provide_texture_for_scene(world, id, orbit_config.clone());
    let response = render_image(ui, world, &texture);
    let motion = response.drag_motion() * 0.01;

    orbit.angles = Vec2::new(
        (orbit.angles.x + motion.x).rem_euclid(2. * PI),
        (orbit.angles.y + motion.y).clamp(-FRAC_PI_2, FRAC_PI_2),
    );

    if ui.rect_contains_pointer(response.interact_rect) {
        let mut zoom = 0.;
        ui.ctx().input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { unit, delta, .. } = event {
                    zoom += delta.y
                        * match unit {
                            egui::MouseWheelUnit::Point => 0.1,
                            egui::MouseWheelUnit::Line => 1.,
                            egui::MouseWheelUnit::Page => 10.,
                        };
                }
            }
        });
        orbit.distance = (orbit.distance - zoom).max(0.001);
    }

    ui.memory_mut(|mem| mem.data.insert_temp::<OrbitView>(orbit_id, orbit));
}

pub fn orbit_camera_transform(view: &OrbitView) -> Transform {
    let mut cam_transform = Transform::from_translation(
        view.distance
            * Vec3::new(
                view.angles.y.cos() * view.angles.x.cos(),
                view.angles.y.sin(),
                view.angles.y.cos() * view.angles.x.sin(),
            ),
    )
    .looking_at(Vec3::ZERO, Vec3::Y);

    cam_transform.translation += Vec3::Y * view.distance.sqrt() * 0.5;
    cam_transform
}

pub fn orbit_camera_update(
    In((target_id, view)): In<(egui::Id, OrbitView)>,
    mut query: Query<(&mut Transform, &PartOfSubScene), With<Camera3d>>,
) {
    for (mut cam_transform, PartOfSubScene(id)) in &mut query {
        if target_id == *id {
            *cam_transform = orbit_camera_transform(&view);
            break;
        }
    }
}

pub fn with_assets<A: Asset, T, const N: usize>(
    world: &mut World,
    assets: [AssetId<A>; N],
    f: impl FnOnce(&mut World, [Option<&A>; N]) -> T,
) -> T {
    world.resource_scope::<Assets<A>, T>(move |world, a_assets| {
        let mut asset_array = [None; N];
        for i in 0..N {
            let a = a_assets.get(assets[i]);
            asset_array[i] = a;
        }
        f(world, asset_array)
    })
}

pub fn with_assets_all<A: Asset, T, const N: usize>(
    world: &mut World,
    assets: [AssetId<A>; N],
    f: impl FnOnce(&mut World, [&A; N]) -> T,
) -> Option<T> {
    with_assets(world, assets, |world, maybe_assets| {
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            maybe_assets[i]?;
        }
        let all_assets =
            std::array::from_fn(|i| maybe_assets[i].expect("Already checked it's not None"));
        Some(f(world, all_assets))
    })
}
