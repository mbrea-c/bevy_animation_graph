use crate::egui_inspector_impls::handle_name;
use crate::egui_nodes::lib::NodesContext;
use crate::graph_show::{make_graph_indices, GraphIndices, GraphReprSpec};
use crate::graph_update::{convert_graph_change, update_graph, Change, GraphChange};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::window::PrimaryWindow;
use bevy_animation_graph::core::animated_scene::{AnimatedScene, AnimatedSceneBundle};
use bevy_animation_graph::core::animation_graph::{AnimationGraph, NodeId};
use bevy_animation_graph::core::animation_node::AnimationNode;
use bevy_animation_graph::core::context::SpecContext;
use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_egui::EguiUserTextures;
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use bevy_inspector_egui::{bevy_egui, egui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

#[derive(Component)]
struct MainCamera;

pub fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

pub struct GraphSelection {
    pub graph: AssetId<AnimationGraph>,
    pub graph_indices: GraphIndices,
    pub nodes_context: NodesContext,
}

pub struct NodeSelection {
    graph: AssetId<AnimationGraph>,
    node: NodeId,
    name_buf: String,
}

pub struct SceneSelection {
    scene: Handle<AnimatedScene>,
    respawn: bool,
}

#[derive(Default)]
pub struct NodeCreation {
    node: AnimationNode,
}

#[derive(Default)]
pub struct InspectorSelection {
    pub graph_editor: Option<GraphSelection>,
    node: Option<NodeSelection>,
    scene: Option<SceneSelection>,
    node_creation: NodeCreation,
}

#[derive(Resource)]
pub struct UiState {
    state: DockState<EguiWindow>,
    pub selection: InspectorSelection,
    graph_changes: Vec<GraphChange>,
    preview_image: Handle<Image>,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GraphEditor]);
        let tree = state.main_surface_mut();
        let [graph_editor, inspectors] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![
                EguiWindow::NodeInspector,
                EguiWindow::GraphInspector,
                EguiWindow::NodeCreate,
            ],
        );
        let [_graph_editor, graph_selector] =
            tree.split_left(graph_editor, 0.2, vec![EguiWindow::GraphSelector]);
        let [_graph_selector, _scene_selector] =
            tree.split_below(graph_selector, 0.2, vec![EguiWindow::SceneSelector]);
        let [_node_inspector, _preview] =
            tree.split_above(inspectors, 0.2, vec![EguiWindow::Preview]);
        //let [_game, _bottom] =
        //tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selection: InspectorSelection::default(),
            graph_changes: vec![],
            preview_image: Handle::default(),
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            selection: &mut self.selection,
            graph_changes: &mut self.graph_changes,
            preview_image: &self.preview_image,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
enum EguiWindow {
    GraphEditor,
    GraphSelector,
    SceneSelector,
    NodeInspector,
    GraphInspector,
    NodeCreate,
    Preview,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selection: &'a mut InspectorSelection,
    graph_changes: &'a mut Vec<GraphChange>,
    preview_image: &'a Handle<Image>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        match window {
            EguiWindow::GraphSelector => graph_selector(self.world, ui, self.selection),
            EguiWindow::SceneSelector => scene_selector(self.world, ui, self.selection),
            EguiWindow::GraphEditor => {
                graph_editor(self.world, ui, self.selection, self.graph_changes)
            }
            EguiWindow::NodeInspector => {
                node_inspector(self.world, ui, self.selection, self.graph_changes)
            }
            EguiWindow::GraphInspector => {
                graph_inspector(self.world, ui, self.selection, self.graph_changes)
            }
            EguiWindow::Preview => {
                animated_scene_preview(self.world, ui, self.preview_image, self.selection)
            }
            EguiWindow::NodeCreate => {
                node_creator(self.world, ui, self.selection, self.graph_changes)
            }
        }

        if !self.graph_changes.is_empty() {
            let must_regen_indices =
                self.world
                    .resource_scope::<Assets<AnimationGraph>, bool>(|_, mut graph_assets| {
                        update_graph(self.graph_changes.clone(), &mut graph_assets)
                    });
            self.graph_changes.clear();
            if must_regen_indices {
                if let Some(graph_selection) = self.selection.graph_editor.as_mut() {
                    graph_selection.graph_indices =
                        graph_indices(self.world, graph_selection.graph);
                }
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

fn graph_editor(
    world: &mut World,
    ui: &mut egui::Ui,
    selection: &mut InspectorSelection,
    graph_changes: &mut Vec<GraphChange>,
) {
    let Some(graph_selection) = &mut selection.graph_editor else {
        ui.centered_and_justified(|ui| ui.label("Select a graph to edit!"));
        return;
    };

    world.resource_scope::<Assets<AnimationGraph>, ()>(|_, graph_assets| {
        if !graph_assets.contains(graph_selection.graph) {
            return;
        }

        let changes = {
            let graph = graph_assets.get(graph_selection.graph).unwrap();
            let spec_context = SpecContext {
                graph_assets: &graph_assets,
            };
            let nodes =
                GraphReprSpec::from_graph(graph, &graph_selection.graph_indices, spec_context);
            graph_selection
                .nodes_context
                .show(nodes.nodes, nodes.edges, ui);
            graph_selection.nodes_context.get_changes().clone()
        }
        .into_iter()
        .map(|c| convert_graph_change(c, &graph_selection.graph_indices, graph_selection.graph));
        graph_changes.extend(changes);

        if let Some(selected_node) = graph_selection.nodes_context.get_selected_nodes().last() {
            if *selected_node > 1 {
                let node_name = graph_selection
                    .graph_indices
                    .node_indices
                    .name(*selected_node)
                    .unwrap();
                if let Some(node_selection) = &mut selection.node {
                    if &node_selection.node != node_name
                        || node_selection.graph != graph_selection.graph
                    {
                        node_selection.node = node_name.clone();
                        node_selection.name_buf = node_name.clone();
                        node_selection.graph = graph_selection.graph;
                    }
                } else {
                    selection.node = Some(NodeSelection {
                        graph: graph_selection.graph,
                        node: node_name.clone(),
                        name_buf: node_name.clone(),
                    });
                }
            }
        }
    });
}

/// Display all assets of the specified asset type `A`
pub fn graph_selector(world: &mut World, ui: &mut egui::Ui, selection: &mut InspectorSelection) {
    let mut queue = CommandQueue::default();

    let mut chosen_id: Option<AssetId<AnimationGraph>> = None;

    world.resource_scope::<AssetServer, ()>(|world, asset_server| {
        // create a context with access to the world except for the `R` resource
        world.resource_scope::<Assets<AnimationGraph>, ()>(|_, assets| {
            let mut assets: Vec<_> = assets.ids().collect();
            assets.sort_by(|a, b| a.cmp(b));
            for handle_id in assets {
                let response =
                    ui.selectable_label(false, handle_name(handle_id.untyped(), &asset_server));

                if response.double_clicked() {
                    chosen_id = Some(handle_id);
                }
            }
        });
    });
    queue.apply(world);
    if let Some(chosen_id) = chosen_id {
        selection.graph_editor = Some(GraphSelection {
            graph: chosen_id,
            graph_indices: graph_indices(world, chosen_id),
            nodes_context: NodesContext::default(),
        });
    }
}

pub fn scene_selector(world: &mut World, ui: &mut egui::Ui, selection: &mut InspectorSelection) {
    let mut queue = CommandQueue::default();

    let mut chosen_handle: Option<Handle<AnimatedScene>> = None;

    world.resource_scope::<AssetServer, ()>(|world, asset_server| {
        // create a context with access to the world except for the `R` resource
        world.resource_scope::<Assets<AnimatedScene>, ()>(|_, assets| {
            let mut assets: Vec<_> = assets.ids().collect();
            assets.sort_by(|a, b| a.cmp(b));
            for handle_id in assets {
                let path = asset_server.get_path(handle_id).unwrap();
                let response =
                    ui.selectable_label(false, handle_name(handle_id.untyped(), &asset_server));

                if response.double_clicked() {
                    chosen_handle = Some(asset_server.get_handle(path).unwrap());
                }
            }
        });
    });
    queue.apply(world);

    if let Some(chosen_handle) = chosen_handle {
        selection.scene = Some(SceneSelection {
            scene: chosen_handle,
            respawn: true,
        });
    }
}

#[derive(Component)]
pub struct PreviewScene;

pub fn scene_spawner(
    mut commands: Commands,
    mut query: Query<(Entity, &Handle<AnimatedScene>), With<PreviewScene>>,
    mut ui_state: ResMut<UiState>,
) {
    if let Ok((entity, scene_handle)) = query.get_single_mut() {
        if let Some(scene_selection) = &mut ui_state.selection.scene {
            if scene_selection.respawn || &scene_selection.scene != scene_handle {
                commands.entity(entity).despawn_recursive();
                commands
                    .spawn(AnimatedSceneBundle {
                        animated_scene: scene_selection.scene.clone(),
                        ..default()
                    })
                    .insert(PreviewScene);
                scene_selection.respawn = false;
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    } else if let Some(scene_selection) = &mut ui_state.selection.scene {
        commands
            .spawn(AnimatedSceneBundle {
                animated_scene: scene_selection.scene.clone(),
                ..default()
            })
            .insert(PreviewScene);
        scene_selection.respawn = false;
    }
}

fn graph_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    selection: &mut InspectorSelection,
    graph_changes: &mut Vec<GraphChange>,
) {
    let mut changes = Vec::new();

    let Some(graph_selection) = &mut selection.graph_editor else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let mut graph_assets = unsafe {
        unsafe_world
            .get_resource_mut::<Assets<AnimationGraph>>()
            .unwrap()
    };
    let graph = graph_assets.get_mut(graph_selection.graph).unwrap();

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let changed = env.ui_for_reflect(graph, ui);

    if changed {
        changes.push(GraphChange {
            change: Change::GraphValidate,
            graph: graph_selection.graph,
        });
    }

    graph_changes.extend(changes);

    queue.apply(world);
}

fn node_creator(
    world: &mut World,
    ui: &mut egui::Ui,
    selection: &mut InspectorSelection,
    graph_changes: &mut Vec<GraphChange>,
) {
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
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    env.ui_for_reflect(&mut selection.node_creation.node, ui);
    let submit_response = ui.button("Create node");

    if submit_response.clicked() && selection.graph_editor.is_some() {
        let graph_selection = selection.graph_editor.as_ref().unwrap();
        graph_changes.push(GraphChange {
            change: Change::NodeCreated(selection.node_creation.node.clone()),
            graph: graph_selection.graph,
        });
    }

    queue.apply(world);
}

fn node_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    selection: &mut InspectorSelection,
    graph_changes: &mut Vec<GraphChange>,
) {
    let mut changes = Vec::new();

    let Some(node_selection) = &mut selection.node else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let mut graph_assets = unsafe {
        unsafe_world
            .get_resource_mut::<Assets<AnimationGraph>>()
            .unwrap()
    };
    let graph = graph_assets.get_mut(node_selection.graph).unwrap();
    let Some(node) = graph.nodes.get_mut(&node_selection.node) else {
        selection.node = None;
        return;
    };

    let response = ui.text_edit_singleline(&mut node_selection.name_buf);
    if response.lost_focus() {
        changes.push(GraphChange {
            change: Change::NodeRenamed(
                node_selection.node.clone(),
                node_selection.name_buf.clone(),
            ),
            graph: node_selection.graph,
        });
    }

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let inner = node.node.inner_reflect();
    let changed = env.ui_for_reflect(inner, ui);

    if changed {
        changes.push(GraphChange {
            change: Change::GraphValidate,
            graph: node_selection.graph,
        });
    }

    graph_changes.extend(changes);

    queue.apply(world);
}

fn graph_indices(world: &mut World, graph_id: AssetId<AnimationGraph>) -> GraphIndices {
    world.resource_scope::<Assets<AnimationGraph>, GraphIndices>(|_, graph_assets| {
        let graph = graph_assets.get(graph_id).unwrap();
        let spec_context = SpecContext {
            graph_assets: &graph_assets,
        };
        let idx = make_graph_indices(graph, spec_context.clone());
        idx
    })
}

fn animated_scene_preview(
    world: &mut World,
    ui: &mut egui::Ui,
    preview_image: &Handle<Image>,
    selection: &mut InspectorSelection,
) {
    if ui.button("Close Preview").clicked() {
        selection.scene = None;
    }

    let cube_preview_texture_id =
        world.resource_scope::<EguiUserTextures, egui::TextureId>(|_, user_textures| {
            user_textures.image_id(&preview_image).unwrap()
        });

    let available_size = ui.available_size();
    let e3d_size = Extent3d {
        width: available_size.x as u32,
        height: available_size.y as u32,
        ..default()
    };
    world.resource_scope::<Assets<Image>, ()>(|_, mut images| {
        let image = images.get_mut(preview_image).unwrap();
        image.texture_descriptor.size = e3d_size;
        image.resize(e3d_size);
    });
    ui.image(egui::load::SizedTexture::new(
        cube_preview_texture_id,
        available_size,
    ));
}

pub fn setup(
    mut egui_user_textures: ResMut<bevy_egui::EguiUserTextures>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    egui_user_textures.add_image(image_handle.clone());
    ui_state.preview_image = image_handle.clone();

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::rgba(1.0, 1.0, 1.0, 0.0)),
            ..default()
        },
        camera: Camera {
            // render before the "main pass" camera
            order: -1,
            target: RenderTarget::Image(image_handle),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 2.0, 3.0))
            .looking_at(Vec3::Y, Vec3::Y),
        ..default()
    });
}
