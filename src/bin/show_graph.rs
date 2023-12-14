use bevy::{app::AppExit, prelude::*};
use bevy_animation_graph::{core::animation_graph::ToDot, prelude::*};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: show_graph <PATH_TO_TARGET_GRAPH>");
    }

    App::new()
        .add_plugins((
            // TODO: Figure out the minimal set of plugins needed
            // to make this work
            DefaultPlugins,
            // LogPlugin::default(),
            // TimePlugin::default(),
            // TaskPoolPlugin::default(),
            // AssetPlugin::default(),
            // GltfPlugin::default(),
            AnimationGraphPlugin,
        ))
        .insert_resource(TargetGraph {
            name: args[1].clone(),
            handle: None,
        })
        .add_systems(Startup, load_graph)
        .add_systems(Update, show_graph)
        .run()
}

#[derive(Resource)]
struct TargetGraph {
    name: String,
    handle: Option<Handle<AnimationGraph>>,
}

fn load_graph(mut target_graph: ResMut<TargetGraph>, asset_server: Res<AssetServer>) {
    let handle: Handle<AnimationGraph> = asset_server.load(&target_graph.name);
    target_graph.handle = Some(handle);
}

fn show_graph(
    target_graph: Res<TargetGraph>,
    animation_graph_assets: Res<Assets<AnimationGraph>>,
    graph_clip_assets: Res<Assets<GraphClip>>,
    asset_server: Res<AssetServer>,
    mut exit: EventWriter<AppExit>,
) {
    match asset_server.recursive_dependency_load_state(target_graph.handle.as_ref().unwrap()) {
        bevy::asset::RecursiveDependencyLoadState::NotLoaded => {}
        bevy::asset::RecursiveDependencyLoadState::Loading => {}
        bevy::asset::RecursiveDependencyLoadState::Loaded => {
            info!("Graph {} loaded", target_graph.name);
            let graph = animation_graph_assets
                .get(target_graph.handle.as_ref().unwrap())
                .unwrap();

            let context_tmp = GraphContextTmp {
                graph_clip_assets: &graph_clip_assets,
                animation_graph_assets: &animation_graph_assets,
            };

            graph.dot_to_tmp_file_and_open(None, context_tmp).unwrap();

            exit.send(AppExit);
        }
        bevy::asset::RecursiveDependencyLoadState::Failed => {
            panic!("Failed to load graph {}", target_graph.name)
        }
    };
}
