mod egui_fsm;
mod egui_nodes;
mod fsm_show;
mod graph_show;
mod icons;
mod scanner;
mod tree;
mod ui;

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_animation_graph::core::plugin::AnimationGraphPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_egui};
use clap::Parser;
use fsm_show::FsmIndicesMap;
use graph_show::GraphIndicesMap;
use scanner::ScannerPlugin;
use std::path::PathBuf;
use ui::UiState;
use ui::actions::PendingActions;
use ui::actions::clip_preview::ClipPreviewScenes;
use ui::actions::saving::DirtyAssets;
use ui::egui_inspector_impls::BetterInspectorPlugin;

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
            .add_plugins(EguiPlugin::default())
            .add_plugins(AnimationGraphPlugin::from_physics_schedule(FixedPostUpdate))
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(BetterInspectorPlugin)
            // .add_plugins(WorldInspectorPlugin::new())
            .add_plugins(ScannerPlugin);

        #[cfg(feature = "physics_avian")]
        app.add_plugins(avian3d::prelude::PhysicsPlugins::new(FixedPostUpdate));

        UiState::init(app.world_mut());

        app.insert_resource(PendingActions::default())
            .insert_resource(DirtyAssets::default())
            .insert_resource(GraphIndicesMap::default())
            .insert_resource(FsmIndicesMap::default())
            .insert_resource(ClipPreviewScenes::default())
            .insert_resource(cli);

        app.add_systems(Startup, setup);

        app.add_systems(
            EguiPrimaryContextPass,
            (
                ui::setup_ui,
                ui::show_ui_system,
                ui::actions::process_actions_system,
                ui::override_scene_animations,
                ui::render_pose_gizmos,
                ui::propagate_layers,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
struct UiCamera;
pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        UiCamera,
        bevy_egui::PrimaryEguiContext,
        RenderLayers::none(),
    ));
}
