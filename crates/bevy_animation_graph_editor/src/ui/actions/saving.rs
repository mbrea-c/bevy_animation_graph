use crate::{
    scanner::PersistedAssetHandles,
    ui::{core::EguiWindow, UiState},
    Cli,
};
use bevy::prelude::*;
use bevy_animation_graph::core::{
    animation_graph::{serial::AnimationGraphSerializer, AnimationGraph},
    state_machine::high_level::{serial::StateMachineSerial, StateMachine},
};
use std::path::PathBuf;

pub enum SaveAction {
    RequestFsm(RequestSaveFsm),
    RequestGraph(RequestSaveGraph),
    Fsm(SaveFsm),
    Graph(SaveGraph),
}

pub struct RequestSaveGraph {
    pub graph: AssetId<AnimationGraph>,
}

pub struct RequestSaveFsm {
    pub fsm: AssetId<StateMachine>,
}

pub struct SaveGraph {
    pub graph: AssetId<AnimationGraph>,
    pub virtual_path: PathBuf,
}

pub struct SaveFsm {
    pub fsm: AssetId<StateMachine>,
    pub virtual_path: PathBuf,
}

pub fn handle_save_action(world: &mut World, action: SaveAction) {
    match action {
        SaveAction::RequestFsm(request_save_fsm) => {
            if let Err(err) =
                world.run_system_cached_with(handle_request_save_fsm, request_save_fsm)
            {
                error!("Failed to apply save action: {:?}", err);
            }
        }
        SaveAction::RequestGraph(request_save_graph) => {
            if let Err(err) =
                world.run_system_cached_with(handle_request_save_graph, request_save_graph)
            {
                error!("Failed to apply save action: {:?}", err);
            }
        }
        SaveAction::Fsm(save_fsm) => {
            if let Err(err) = world.run_system_cached_with(handle_save_fsm, save_fsm) {
                error!("Failed to apply save action: {:?}", err);
            }
        }
        SaveAction::Graph(save_graph) => {
            if let Err(err) = world.run_system_cached_with(handle_save_graph, save_graph) {
                error!("Failed to apply save action: {:?}", err);
            }
        }
    }
}

pub fn handle_request_save_graph(
    In(save_request): In<RequestSaveGraph>,
    mut ui_state: ResMut<UiState>,
    asset_server: Res<AssetServer>,
) {
    if let Some(active_view_idx) = ui_state.active_view {
        let path = asset_server
            .get_path(save_request.graph)
            .map_or("".into(), |p| p.path().to_string_lossy().into());
        let window = EguiWindow::GraphSaver(save_request.graph, path, false);
        ui_state.views[active_view_idx]
            .dock_state
            .add_window(vec![window]);
    }
}

pub fn handle_request_save_fsm(
    In(save_request): In<RequestSaveFsm>,
    mut ui_state: ResMut<UiState>,
    asset_server: Res<AssetServer>,
) {
    if let Some(active_view_idx) = ui_state.active_view {
        let path = asset_server
            .get_path(save_request.fsm)
            .map_or("".into(), |p| p.path().to_string_lossy().into());
        let window = EguiWindow::FsmSaver(save_request.fsm, path, false);
        ui_state.views[active_view_idx]
            .dock_state
            .add_window(vec![window]);
    }
}

pub fn handle_save_graph(
    In(save_graph): In<SaveGraph>,
    asset_server: Res<AssetServer>,
    graph_assets: Res<Assets<AnimationGraph>>,
    mut ui_state: ResMut<UiState>,
    mut graph_handles: ResMut<PersistedAssetHandles>,
    cli: Res<Cli>,
    registry: Res<AppTypeRegistry>,
) {
    let type_registry = registry.0.read();
    let graph = graph_assets.get(save_graph.graph).unwrap();
    let graph_serial = AnimationGraphSerializer::new(graph, &type_registry);
    let mut final_path = cli.asset_source.clone();
    final_path.push(&save_graph.virtual_path);
    info!(
        "Saving graph with id {:?} to {:?}",
        save_graph.graph, final_path
    );
    ron::ser::to_writer_pretty(
        std::fs::File::create(final_path).unwrap(),
        &graph_serial,
        ron::ser::PrettyConfig::default(),
    )
    .unwrap();

    // If we just saved a newly created graph, unload the in-memory asset from the
    // editor selection.
    // Also delete the temporary asset
    if asset_server.get_path(save_graph.graph).is_none() {
        ui_state.global_state.graph_editor = None;
        graph_handles
            .unsaved_graphs
            .retain(|h| h.id() != save_graph.graph);
    }
}

pub fn handle_save_fsm(
    In(save_fsm): In<SaveFsm>,
    asset_server: Res<AssetServer>,
    graph_assets: Res<Assets<StateMachine>>,
    mut ui_state: ResMut<UiState>,
    mut persisted_handles: ResMut<PersistedAssetHandles>,
    cli: Res<Cli>,
) {
    let fsm = graph_assets.get(save_fsm.fsm).unwrap();
    let graph_serial = StateMachineSerial::from(fsm);
    let mut final_path = cli.asset_source.clone();
    final_path.push(&save_fsm.virtual_path);
    info!("Saving FSM with id {:?} to {:?}", save_fsm.fsm, final_path);
    ron::ser::to_writer_pretty(
        std::fs::File::create(final_path).unwrap(),
        &graph_serial,
        ron::ser::PrettyConfig::default(),
    )
    .unwrap();

    // If we just saved a newly created graph, unload the in-memory asset from the
    // editor selection.
    // Also delete the temporary asset
    if asset_server.get_path(save_fsm.fsm).is_none() {
        ui_state.global_state.fsm_editor = None;
        persisted_handles
            .unsaved_fsms
            .retain(|h| h.id() != save_fsm.fsm);
    }
}
