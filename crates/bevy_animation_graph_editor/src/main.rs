mod asset_saving;
mod egui_fsm;
mod egui_inspector_impls;
mod egui_nodes;
mod fsm_show;
mod graph_show;
mod graph_update;
mod tree;
mod ui;

use asset_saving::{save_graph_system, AssetSavingPlugin, SaveGraph};
use bevy::{asset::LoadedFolder, prelude::*, utils::HashSet};
use bevy_animation_graph::core::{
    animation_graph::AnimationGraph, plugin::AnimationGraphPlugin,
    state_machine::high_level::StateMachine,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{bevy_egui, DefaultInspectorConfigPlugin};
use clap::Parser;
use egui_inspector_impls::BetterInspectorPlugin;
use std::path::PathBuf;
use ui::{graph_debug_draw_bone_system, UiState};

#[derive(Parser, Resource)]
struct Cli {
    #[arg(short, long)]
    asset_source: PathBuf,
}

/// Keeps a handle to the folder so that it does not get unloaded
#[derive(Resource)]
struct PersistedAssetHandles {
    #[allow(dead_code)]
    folder: Handle<LoadedFolder>,
    unsaved_graphs: HashSet<Handle<AnimationGraph>>,
    unsaved_fsms: HashSet<Handle<StateMachine>>,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    app //
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                file_path: std::fs::canonicalize(&cli.asset_source)
                    .unwrap()
                    .to_string_lossy()
                    .into(),
                ..Default::default()
            }),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(AnimationGraphPlugin)
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(BetterInspectorPlugin)
        .add_plugins(AssetSavingPlugin)
        .insert_resource(UiState::new())
        .insert_resource(cli)
        .add_systems(Startup, (core_setup, ui::setup_system))
        .add_systems(
            Update,
            (
                ui::show_ui_system,
                ui::asset_save_event_system,
                ui::scene_spawner_system,
                graph_debug_draw_bone_system,
            )
                .chain(),
        );

    app.run();
}

fn core_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut gizmo_config: ResMut<GizmoConfigStore>,
) {
    commands.insert_resource(PersistedAssetHandles {
        folder: asset_server.load_folder(""),
        unsaved_graphs: HashSet::default(),
        unsaved_fsms: HashSet::default(),
    });

    let config = gizmo_config.config_mut::<DefaultGizmoConfigGroup>().0;
    config.depth_bias = -1.;
}
