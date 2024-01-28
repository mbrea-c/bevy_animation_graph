mod egui_inspector_impls;
mod egui_nodes;
mod graph_show;
mod graph_update;
mod ui;

use std::path::PathBuf;

use bevy::{
    asset::{
        io::{file::FileAssetReader, AssetReader, AssetSource},
        LoadedFolder,
    },
    prelude::*,
};
use bevy_animation_graph::core::{
    animation_graph::{serial::AnimationGraphSerial, AnimationGraph},
    plugin::AnimationGraphPlugin,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{bevy_egui, DefaultInspectorConfigPlugin};
use clap::Parser;
use egui_inspector_impls::register_editor_impls;
use ui::UiState;

#[derive(Parser, Resource)]
struct Cli {
    #[arg(short, long)]
    asset_source: PathBuf,
}

/// Keeps a handle to the folder so that it does not get unloaded
#[derive(Resource)]
struct GraphsFolder {
    #[allow(dead_code)]
    folder: Handle<LoadedFolder>,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    // app.register_asset_source("graphs", {
    //     let source = cli.asset_source.clone();
    //     AssetSource::build().with_reader(move || {
    //         let s = source.clone();
    //         Box::new(FileAssetReader::new(s))
    //     })
    // })
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: cli.asset_source.to_string_lossy().into(),
        ..Default::default()
    }))
    .add_plugins(EguiPlugin)
    .add_plugins(AnimationGraphPlugin)
    .add_plugins(DefaultInspectorConfigPlugin)
    .insert_resource(UiState::new())
    .insert_resource(cli)
    .add_systems(Startup, (load_target_graph, ui::setup))
    .add_systems(
        Update,
        (ui::show_ui_system, save_graph_system, ui::scene_spawner).chain(),
    );

    {
        let type_registry = app.world.resource::<bevy::ecs::prelude::AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        register_editor_impls(&mut type_registry);
    }

    app.run();
}

fn load_target_graph(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GraphsFolder {
        folder: asset_server.load_folder(""),
    });
}

fn save_graph_system(world: &mut World) {
    if !world.contains_resource::<UiState>() {
        return;
    }

    world.resource_scope::<UiState, ()>(|world, ui_state| {
        let Some(graph_selection) = &ui_state.selection.graph_editor else {
            return;
        };

        world.resource_scope::<Assets<AnimationGraph>, ()>(|world, graph_assets| {
            world.resource_scope::<AssetServer, ()>(|world, asset_server| {
                let input = world.resource::<Input<KeyCode>>();
                if input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::S) {
                    let graph = graph_assets.get(graph_selection.graph).unwrap();
                    let graph_serial = AnimationGraphSerial::from(graph);
                    let path = asset_server.get_path(graph_selection.graph).unwrap();
                    let source = asset_server.get_source(path.source()).unwrap();
                    let reader = source.reader();
                    // HACK: Ideally we would not be doing this, but using bevy's dyanmic asset
                    // saving API. However, this does not exist yet, so we're force to find the
                    // file path via "other means".
                    // unsafe downcast reader to FileAssetReader, since as_any is not implemented
                    // pray that nothing goes wrong
                    let reader =
                        unsafe { &*((reader as *const dyn AssetReader) as *const FileAssetReader) };
                    let mut final_path = reader.root_path().clone();
                    final_path.push(path.path());
                    info!("Saving graph to {:?}", final_path);
                    ron::ser::to_writer_pretty(
                        std::fs::File::create(final_path).unwrap(),
                        &graph_serial,
                        ron::ser::PrettyConfig::default(),
                    )
                    .unwrap();
                }
            });
        });
    });
}
