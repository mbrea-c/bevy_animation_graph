mod egui_inspector_impls;
mod egui_nodes;
mod graph_saving;
mod graph_show;
mod graph_update;
mod ui;

use bevy::{asset::LoadedFolder, prelude::*, utils::HashSet};
use bevy_animation_graph::core::{animation_graph::AnimationGraph, plugin::AnimationGraphPlugin};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{bevy_egui, DefaultInspectorConfigPlugin};
use clap::Parser;
use egui_inspector_impls::register_editor_impls;
use graph_saving::{save_graph_system, SaveGraph};
use std::path::PathBuf;
use ui::UiState;

#[derive(Parser, Resource)]
struct Cli {
    #[arg(short, long)]
    asset_source: PathBuf,
}

/// Keeps a handle to the folder so that it does not get unloaded
#[derive(Resource)]
struct GraphHandles {
    #[allow(dead_code)]
    folder: Handle<LoadedFolder>,
    unsaved: HashSet<Handle<AnimationGraph>>,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    app //
        .add_event::<SaveGraph>()
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
        .insert_resource(UiState::new())
        .insert_resource(cli)
        .add_systems(Startup, (load_target_graph, ui::setup_system))
        .add_systems(
            Update,
            (
                ui::show_ui_system,
                ui::graph_save_event_system,
                ui::scene_spawner_system,
                save_graph_system,
            )
                .chain(),
        );

    {
        let type_registry = app.world.resource::<bevy::ecs::prelude::AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        register_editor_impls(&mut type_registry);
    }

    app.run();
}

fn load_target_graph(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GraphHandles {
        folder: asset_server.load_folder(""),
        unsaved: HashSet::default(),
    });
}
