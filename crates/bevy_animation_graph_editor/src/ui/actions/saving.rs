use crate::{
    Cli,
    ui::{
        UiState,
        core::EguiWindow,
        editor_windows::saving::{SaveWindow, SaveWindowAssetMeta},
        global_state::{ClearGlobalState, active_fsm::ActiveFsm, active_graph::ActiveGraph},
    },
};
use bevy::{asset::UntypedAssetId, platform::collections::HashMap, prelude::*};
use bevy_animation_graph::{
    core::{
        animation_clip::loader::GraphClipSerial,
        animation_graph::{AnimationGraph, serial::AnimationGraphSerializer},
        ragdoll::{
            bone_mapping::RagdollBoneMap, bone_mapping_loader::RagdollBoneMapSerial,
            definition::Ragdoll,
        },
        state_machine::high_level::{StateMachine, serial::StateMachineSerial},
    },
    prelude::GraphClip,
};
use std::path::PathBuf;

/// Assets that have been modified in the editor but not yet saved
#[derive(Resource, Default)]
pub struct DirtyAssets {
    // The hashmap structure is just so that we can index/remove without having a handle
    pub assets: HashMap<UntypedAssetId, UntypedHandle>,
}

impl DirtyAssets {
    pub fn add(&mut self, asset: impl Into<UntypedHandle>) {
        let handle = asset.into();
        self.assets.insert(handle.id(), handle);
    }
}

pub enum SaveAction {
    RequestMultiple,
    Multiple(SaveMultiple),
}

pub struct SaveGraph {
    pub asset_id: AssetId<AnimationGraph>,
    pub virtual_path: PathBuf,
}

pub struct SaveFsm {
    pub asset_id: AssetId<StateMachine>,
    pub virtual_path: PathBuf,
}

pub struct SaveClip {
    pub asset_id: AssetId<GraphClip>,
    pub virtual_path: PathBuf,
}

pub struct SaveRagdoll {
    pub asset_id: AssetId<Ragdoll>,
    pub virtual_path: PathBuf,
}

pub struct SaveRagdollBoneMap {
    pub asset_id: AssetId<RagdollBoneMap>,
    pub virtual_path: PathBuf,
}

pub struct SaveMultiple {
    /// Map from asset ids to the path where they should be saved (relative to asset source root)
    pub assets: HashMap<UntypedAssetId, PathBuf>,
}

pub fn handle_save_action(world: &mut World, action: SaveAction) {
    match action {
        SaveAction::Multiple(action) => {
            if let Err(err) = world.run_system_cached_with(handle_save_multiple, action) {
                error!("Failed to apply save action: {:?}", err);
            }
        }
        SaveAction::RequestMultiple => {
            if let Err(err) = world.run_system_cached(handle_request_save_multiple) {
                error!("Failed to apply save action: {:?}", err);
            }
        }
    }
}

pub fn handle_save_graph(
    In(save_graph): In<SaveGraph>,
    asset_server: Res<AssetServer>,
    graph_assets: Res<Assets<AnimationGraph>>,
    cli: Res<Cli>,
    registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    let type_registry = registry.0.read();
    let graph = graph_assets.get(save_graph.asset_id).unwrap();
    let graph_serial = AnimationGraphSerializer::new(graph, &type_registry);
    let mut final_path = cli.asset_source.clone();
    final_path.push(&save_graph.virtual_path);
    info!(
        "Saving graph with id {:?} to {:?}",
        save_graph.asset_id, final_path
    );
    ron::Options::default()
        .to_io_writer_pretty(
            std::fs::File::create(final_path).unwrap(),
            &graph_serial,
            ron::ser::PrettyConfig::default(),
        )
        .unwrap();

    // If we just saved a newly created graph, unload the in-memory asset from the
    // editor selection.
    // Also delete the temporary asset
    if asset_server.get_path(save_graph.asset_id).is_none() {
        commands.trigger(ClearGlobalState::<ActiveGraph>::default());
    }
}

pub fn handle_save_fsm(
    In(save_fsm): In<SaveFsm>,
    asset_server: Res<AssetServer>,
    graph_assets: Res<Assets<StateMachine>>,
    cli: Res<Cli>,
    mut commands: Commands,
) {
    let fsm = graph_assets.get(save_fsm.asset_id).unwrap();
    let graph_serial = StateMachineSerial::from(fsm);
    let mut final_path = cli.asset_source.clone();
    final_path.push(&save_fsm.virtual_path);
    info!(
        "Saving FSM with id {:?} to {:?}",
        save_fsm.asset_id, final_path
    );
    ron::Options::default()
        .to_io_writer_pretty(
            std::fs::File::create(final_path).unwrap(),
            &graph_serial,
            ron::ser::PrettyConfig::default(),
        )
        .unwrap();

    // If we just saved a newly created graph, unload the in-memory asset from the
    // editor selection.
    // Also delete the temporary asset
    if asset_server.get_path(save_fsm.asset_id).is_none() {
        commands.trigger(ClearGlobalState::<ActiveFsm>::default());
    }
}

pub fn handle_save_animation_clip(
    In(save_fsm): In<SaveClip>,
    clip_assets: Res<Assets<GraphClip>>,
    cli: Res<Cli>,
) {
    let clip = clip_assets.get(save_fsm.asset_id).unwrap();
    let Ok(clip_serial) = GraphClipSerial::try_from(clip) else {
        error!("Could not save graph clip asset");
        return;
    };
    let mut final_path = cli.asset_source.clone();
    final_path.push(&save_fsm.virtual_path);
    info!(
        "Saving animation clip with id {:?} to {:?}",
        save_fsm.asset_id, final_path
    );
    ron::Options::default()
        .to_io_writer_pretty(
            std::fs::File::create(final_path).unwrap(),
            &clip_serial,
            ron::ser::PrettyConfig::default(),
        )
        .unwrap();
}

pub fn handle_save_ragdoll(
    In(input): In<SaveRagdoll>,
    ragdoll_assets: Res<Assets<Ragdoll>>,
    cli: Res<Cli>,
) {
    let ragdoll = ragdoll_assets.get(input.asset_id).unwrap();
    let mut final_path = cli.asset_source.clone();
    final_path.push(&input.virtual_path);
    info!(
        "Saving Ragdoll with id {:?} to {:?}",
        input.asset_id, final_path
    );
    ron::Options::default()
        .to_io_writer_pretty(
            std::fs::File::create(final_path).unwrap(),
            &ragdoll,
            ron::ser::PrettyConfig::default(),
        )
        .unwrap();
}

pub fn handle_save_ragdoll_bone_map(
    In(input): In<SaveRagdollBoneMap>,
    ragdoll_bone_map_assets: Res<Assets<RagdollBoneMap>>,
    cli: Res<Cli>,
) {
    let ragdoll_bone_map = ragdoll_bone_map_assets.get(input.asset_id).unwrap();
    let mut final_path = cli.asset_source.clone();
    final_path.push(&input.virtual_path);
    info!(
        "Saving Ragdoll bone mapping with id {:?} to {:?}",
        input.asset_id, final_path
    );

    let Some(ragdoll_bone_map_serial) = RagdollBoneMapSerial::from_value(ragdoll_bone_map) else {
        error!(
            "Could not serialize ragdoll bone mapping with id {:?}",
            input.asset_id
        );
        return;
    };

    ron::Options::default()
        .to_io_writer_pretty(
            std::fs::File::create(final_path).unwrap(),
            &ragdoll_bone_map_serial,
            ron::ser::PrettyConfig::default(),
        )
        .unwrap();
}

pub fn handle_save_multiple(
    In(action): In<SaveMultiple>,
    mut commands: Commands,
    mut dirty_assets: ResMut<DirtyAssets>,
) {
    for (asset_id, virtual_path) in action.assets.into_iter() {
        // TODO: Do we care if saving succeeded? We won't know until later
        dirty_assets.assets.remove(&asset_id);

        if let Ok(asset_id) = asset_id.try_typed::<AnimationGraph>() {
            commands.run_system_cached_with(
                handle_save_graph,
                SaveGraph {
                    asset_id,
                    virtual_path,
                },
            );
        } else if let Ok(asset_id) = asset_id.try_typed::<StateMachine>() {
            commands.run_system_cached_with(
                handle_save_fsm,
                SaveFsm {
                    asset_id,
                    virtual_path,
                },
            );
        } else if let Ok(asset_id) = asset_id.try_typed::<GraphClip>() {
            commands.run_system_cached_with(
                handle_save_animation_clip,
                SaveClip {
                    asset_id,
                    virtual_path,
                },
            );
        } else if let Ok(asset_id) = asset_id.try_typed::<Ragdoll>() {
            commands.run_system_cached_with(
                handle_save_ragdoll,
                SaveRagdoll {
                    asset_id,
                    virtual_path,
                },
            );
        } else if let Ok(asset_id) = asset_id.try_typed::<RagdollBoneMap>() {
            commands.run_system_cached_with(
                handle_save_ragdoll_bone_map,
                SaveRagdollBoneMap {
                    asset_id,
                    virtual_path,
                },
            );
        }
    }
}

pub fn handle_request_save_multiple(
    mut ui_state: ResMut<UiState>,
    asset_server: Res<AssetServer>,
    dirty_assets: Res<DirtyAssets>,
) {
    if let Some(active_view_idx) = ui_state.active_view {
        let metas = dirty_assets
            .assets
            .keys()
            .copied()
            .map(|id| {
                asset_server
                    .get_path(id)
                    .map(|path| SaveWindowAssetMeta {
                        id,
                        should_save: false,
                        should_rename: false,
                        virtual_path: path.clone_owned().path().to_path_buf(),
                        current_path: Some(path.into_owned()),
                    })
                    .unwrap_or_else(|| SaveWindowAssetMeta {
                        id,
                        should_save: false,
                        should_rename: true,
                        virtual_path: PathBuf::default(),
                        current_path: None,
                    })
            })
            .collect();

        let window_id = ui_state.windows.open(SaveWindow::new(metas));
        let window = EguiWindow::DynWindow(window_id);

        let UiState { views, .. } = ui_state.into_inner();

        views[active_view_idx].dock_state.add_window(vec![window]);
    }
}
