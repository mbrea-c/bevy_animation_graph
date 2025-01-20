mod asset_saving;
mod egui_fsm;
mod egui_inspector_impls;
mod egui_nodes;
mod fsm_show;
mod graph_show;
mod graph_update;
mod scanner;
mod tree;
mod ui;

use asset_saving::AssetSavingPlugin;
use bevy::prelude::*;
use bevy_animation_graph::core::plugin::AnimationGraphPlugin;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{bevy_egui, DefaultInspectorConfigPlugin};
use clap::Parser;
use egui_inspector_impls::BetterInspectorPlugin;
use scanner::ScannerPlugin;
use std::path::PathBuf;
use ui::{graph_debug_draw_bone_system, UiState};

#[derive(Parser, Resource)]
struct Cli {
    #[arg(short, long)]
    asset_source: PathBuf,
}

/// Will start up an animation graph asset editor. This plugin will add all other required
/// plugins: if you use it, do not add [`DefaultPlugins`], [`EguiPlugin`] or others manually (see
/// plugin source code for full up-to-date list).
///
/// This plugin also takes care of command-line parsing.
///
/// You only need to add plugins necessary to register your custom animation node types, if you
/// have any.
pub struct AnimationGraphEditorPlugin;

impl Plugin for AnimationGraphEditorPlugin {
    fn build(&self, app: &mut App) {
        let cli = Cli::parse();

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
            .add_plugins(ScannerPlugin)
            .insert_resource(UiState::new())
            .insert_resource(cli)
            .add_systems(Startup, ui::setup_system)
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
    }
}
