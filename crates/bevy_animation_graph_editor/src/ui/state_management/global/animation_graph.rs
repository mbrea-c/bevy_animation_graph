use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::Event,
    observer::On,
    reflect::AppTypeRegistry,
    system::{Commands, Res, ResMut},
    world::World,
};
use bevy_animation_graph::core::animation_graph::{
    AnimationGraph, serial::AnimationGraphSerializer,
};

use crate::{
    Cli,
    scanner::RescanAssets,
    ui::{
        UiState,
        core::EguiWindow,
        native_windows::{
            NativeEditorWindow, asset_creation::animation_graph::CreateAnimationGraphWindow,
        },
        state_management::global::RegisterStateComponent,
    },
};

#[derive(Debug, Component, Default, Clone)]
pub struct AnimationGraphManager;

impl RegisterStateComponent for AnimationGraphManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(RequestCreateAnimationGraph::observe);
        world.add_observer(CreateAnimationGraph::observe);
    }
}

/// Will open a "create FSM" window popup
#[derive(Event)]
pub struct RequestCreateAnimationGraph;

impl RequestCreateAnimationGraph {
    pub fn observe(
        _: On<RequestCreateAnimationGraph>,
        mut commands: Commands,
        ui_state: ResMut<UiState>,
    ) {
        if let Some(active_view_idx) = ui_state.active_view {
            let view = &ui_state.views[active_view_idx];
            let win = NativeEditorWindow::create_cmd(
                &mut commands,
                view.entity,
                CreateAnimationGraphWindow,
            );

            let UiState { views, .. } = ui_state.into_inner();

            views[active_view_idx]
                .dock_state
                .add_window(vec![EguiWindow::EntityWindow(win)]);
        }
    }
}

#[derive(Event)]
pub struct CreateAnimationGraph {
    pub virtual_path: String,
    pub animation_graph: AnimationGraph,
}

impl CreateAnimationGraph {
    pub fn observe(
        event: On<CreateAnimationGraph>,
        cli: Res<Cli>,
        registry: Res<AppTypeRegistry>,
        mut commands: Commands,
    ) {
        let type_registry = registry.0.read();
        let graph_serial = AnimationGraphSerializer::new(&event.animation_graph, &type_registry);

        let mut final_path = cli.asset_source.clone();
        final_path.push(&event.virtual_path);
        bevy::log::info!("Creating animation graph metadata at {:?}", final_path);
        ron::Options::default()
            .to_io_writer_pretty(
                std::fs::File::create(final_path).unwrap(),
                &graph_serial,
                ron::ser::PrettyConfig::default(),
            )
            .unwrap();

        commands.trigger(RescanAssets);
    }
}
