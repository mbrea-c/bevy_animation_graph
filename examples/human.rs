use bevy::{
    asset::RecursiveDependencyLoadState, gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*,
};
use bevy_animation_graph::animation::{
    AnimationGraph, AnimationPlayer, AnimationPlugin, InterpolationMode, WrapEnd,
};
use bevy_animation_graph::nodes::{
    chain_node::ChainNode, clip_node::ClipNode, flip_lr_node::FlipLRNode, loop_node::LoopNode,
    speed_node::SpeedNode,
};
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AnimationPlugin)
        .add_plugins(bevy_egui_editor::EguiEditorPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(ProcessedGraphs(vec![]))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_graphs,
                setup_scene_once_loaded,
                keyboard_animation_control,
            ),
        )
        .run();
}

#[derive(Resource)]
struct RootGltf(Handle<Gltf>);

#[derive(Resource)]
struct ProcessedGraphs(Vec<Handle<AnimationGraph>>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Insert a resource with the current scene information
    commands.insert_resource(RootGltf(asset_server.load("models/character_rigged.gltf")));

    // Camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.)
                .looking_at(Vec3::new(0.0, 2.5, 0.0), Vec3::Y),
            ..default()
        })
        .insert(bevy_egui_editor::EditorCamera);

    // Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(500000.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // Fox
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/character_rigged.gltf#Scene0"),
        transform: Transform::from_xyz(0.0, 2.4, 0.0),
        ..default()
    });
}

fn process_graphs(
    mut processed_graphs: ResMut<ProcessedGraphs>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    root_gltf: Res<RootGltf>,
    animation_clips: Res<Assets<bevy::animation::AnimationClip>>,
    gltf_assets: Res<Assets<Gltf>>,
    asset_server: Res<AssetServer>,
) {
    if asset_server.recursive_dependency_load_state(root_gltf.0.clone())
        != RecursiveDependencyLoadState::Loaded
    {
        return;
    }

    let gltf = gltf_assets.get(root_gltf.0.clone()).unwrap();

    let h_run_clip = gltf.named_animations.get("ActionBaked").unwrap();
    let h_walk_clip = gltf.named_animations.get("WalkBaked2").unwrap();

    let walk_clip = animation_clips.get(h_walk_clip).unwrap().clone();
    let run_clip = animation_clips.get(h_run_clip).unwrap().clone();

    let mut graph = AnimationGraph::new();
    graph.add_node(
        "AnimClip".into(),
        ClipNode::new(
            walk_clip.into(),
            WrapEnd::Loop,
            Some(1.),
            Some(vec![0., 0.5]),
        )
        .wrapped(),
        Some(ClipNode::OUTPUT.into()),
    );
    graph.set_interpolation(InterpolationMode::Linear);
    graph.add_node("Flip LR".into(), FlipLRNode::new().wrapped(), None);
    graph.add_node("Chain".into(), ChainNode::new().wrapped(), None);
    graph.add_node("Speed".into(), SpeedNode::new(1.5).wrapped(), None);
    graph.add_node(
        "Loop".into(),
        LoopNode::new().wrapped(),
        Some(LoopNode::OUTPUT.into()),
    );

    graph.add_edge(
        "AnimClip".into(),
        ClipNode::OUTPUT.into(),
        "Flip LR".into(),
        FlipLRNode::INPUT.into(),
    );
    graph.add_edge(
        "AnimClip".into(),
        ClipNode::OUTPUT.into(),
        "Chain".into(),
        ChainNode::INPUT_1.into(),
    );
    graph.add_edge(
        "Flip LR".into(),
        FlipLRNode::OUTPUT.into(),
        "Chain".into(),
        ChainNode::INPUT_2.into(),
    );
    graph.add_edge(
        "Chain".into(),
        ChainNode::OUTPUT.into(),
        "Speed".into(),
        SpeedNode::INPUT.into(),
    );
    graph.add_edge(
        "Speed".into(),
        SpeedNode::OUTPUT.into(),
        "Loop".into(),
        LoopNode::INPUT.into(),
    );

    graph.duration_pass();
    processed_graphs.0.push(graphs.add(graph));
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<ProcessedGraphs>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    if animations.0.len() > 0 {
        for mut player in &mut players {
            player.start(animations.0[0].clone());
        }
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<ProcessedGraphs>,
) {
    for mut player in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }
        if keyboard_input.just_pressed(KeyCode::R) {
            player.reset();
        }

        if keyboard_input.just_pressed(KeyCode::Key1) {
            player.start(animations.0[0].clone());
        }
    }
}
