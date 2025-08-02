mod egui_fsm;
mod egui_nodes;
mod fsm_show;
mod graph_show;
mod scanner;
mod tree;
mod ui;

use bevy::prelude::*;
use bevy_animation_graph::core::plugin::AnimationGraphPlugin;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_egui};
use clap::Parser;
use fsm_show::FsmIndicesMap;
use graph_show::GraphIndicesMap;
use scanner::ScannerPlugin;
use std::path::PathBuf;
use ui::actions::PendingActions;
use ui::actions::clip_preview::ClipPreviewScenes;
use ui::actions::saving::DirtyAssets;
use ui::egui_inspector_impls::BetterInspectorPlugin;
use ui::{UiState, graph_debug_draw_bone_system};

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
            .add_plugins(EguiPlugin {
                enable_multipass_for_primary_context: false,
            })
            .add_plugins(AnimationGraphPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(BetterInspectorPlugin)
            // .add_plugins(WorldInspectorPlugin::new())
            .add_plugins(ScannerPlugin);

        #[cfg(feature = "physics_avian")]
        app.add_plugins(avian3d::prelude::PhysicsPlugins::default());

        app.insert_resource(UiState::new())
            .insert_resource(PendingActions::default())
            .insert_resource(DirtyAssets::default())
            .insert_resource(GraphIndicesMap::default())
            .insert_resource(FsmIndicesMap::default())
            .insert_resource(ClipPreviewScenes::default())
            .insert_resource(cli)
            .add_systems(
                Update,
                (
                    ui::show_ui_system,
                    ui::actions::process_actions_system,
                    ui::override_scene_animations,
                    ui::render_pose_gizmos,
                    ui::propagate_layers,
                    graph_debug_draw_bone_system,
                )
                    .chain(),
            );
    }
}
