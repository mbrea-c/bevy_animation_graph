use bevy::{
    asset::RecursiveDependencyLoadState, gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*,
};
use bevy_animation_graph::animation::AnimationPlugin;
use bevy_animation_graph::core::animation_graph::AnimationGraph;
use bevy_animation_graph::core::animation_graph::ToDot;
use bevy_animation_graph::core::animation_player::AnimationPlayer;
use bevy_animation_graph::nodes::blend_node::BlendNode;
use bevy_animation_graph::nodes::chain_node::ChainNode;
use bevy_animation_graph::nodes::clip_node::ClipNode;
use bevy_animation_graph::nodes::flip_lr_node::FlipLRNode;
use bevy_animation_graph::nodes::loop_node::LoopNode;
use bevy_animation_graph::nodes::speed_node::SpeedNode;
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
    if !processed_graphs.0.is_empty() {
        return;
    }

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
    graph.add_node(ClipNode::new(walk_clip.into(), Some(1.)).wrapped("WalkClip"));
    graph.add_node(ClipNode::new(run_clip.into(), Some(1.)).wrapped("RunClip"));
    graph.add_node(ChainNode::new().wrapped("WalkChain"));
    graph.add_node(ChainNode::new().wrapped("RunChain"));
    graph.add_node(FlipLRNode::new().wrapped("Flip LR"));
    graph.add_node(FlipLRNode::new().wrapped("Run Flip LR"));
    graph.add_node(BlendNode::new().wrapped("Blend"));
    graph.add_node(LoopNode::new().wrapped("Loop"));
    graph.add_node(SpeedNode::new().wrapped("Speed"));

    graph.set_out_edge("Speed", SpeedNode::OUTPUT);

    graph.set_parameter("Blend Alpha".into(), 0.5.into());
    graph.set_parameter("Speed factor".into(), 1.0.into());
    graph.add_parameter_edge("Blend Alpha", "Blend", BlendNode::FACTOR);
    graph.add_parameter_edge("Speed factor", "Speed", SpeedNode::SPEED);

    graph.add_edge(
        "WalkClip",
        ClipNode::OUTPUT,
        "WalkChain",
        ChainNode::INPUT_1,
    );
    graph.add_edge("WalkClip", ClipNode::OUTPUT, "Flip LR", FlipLRNode::INPUT);
    graph.add_edge(
        "Flip LR",
        FlipLRNode::OUTPUT,
        "WalkChain",
        ChainNode::INPUT_2,
    );
    graph.add_edge("RunClip", ClipNode::OUTPUT, "RunChain", ChainNode::INPUT_1);
    graph.add_edge(
        "RunClip",
        ClipNode::OUTPUT,
        "Run Flip LR",
        FlipLRNode::INPUT,
    );
    graph.add_edge(
        "Run Flip LR",
        FlipLRNode::OUTPUT,
        "RunChain",
        ChainNode::INPUT_2,
    );

    graph.add_edge("WalkChain", ChainNode::OUTPUT, "Blend", BlendNode::INPUT_1);
    graph.add_edge("RunChain", ChainNode::OUTPUT, "Blend", BlendNode::INPUT_2);
    graph.add_edge("Blend", BlendNode::OUTPUT, "Loop", LoopNode::INPUT);
    graph.add_edge("Loop", LoopNode::OUTPUT, "Speed", SpeedNode::INPUT);

    // graph.add_edge(
    //     "RunClip".into(),
    //     ClipNode::OUTPUT.into(),
    //     "RunChain".into(),
    //     ChainNode::INPUT_1.into(),
    // );
    // graph.add_edge(
    //     "RunClip".into(),
    //     ClipNode::OUTPUT.into(),
    //     "Run Flip LR".into(),
    //     FlipLRNode::INPUT.into(),
    // );
    // graph.add_edge(
    //     "Run Flip LR".into(),
    //     FlipLRNode::OUTPUT.into(),
    //     "RunChain".into(),
    //     ChainNode::INPUT_2.into(),
    // );
    // graph.add_edge(
    //     "WalkChain".into(),
    //     ChainNode::OUTPUT.into(),
    //     "Walk speed".into(),
    //     SpeedNode::INPUT.into(),
    // );
    // graph.add_edge(
    //     "RunChain".into(),
    //     ChainNode::OUTPUT.into(),
    //     "Run speed".into(),
    //     SpeedNode::INPUT.into(),
    // );
    // graph.add_edge(
    //     "Walk speed".into(),
    //     SpeedNode::OUTPUT.into(),
    //     "Blend".into(),
    //     BlendNode::INPUT_POSE_1.into(),
    // );
    // graph.add_edge(
    //     "Run speed".into(),
    //     SpeedNode::OUTPUT.into(),
    //     "Blend".into(),
    //     BlendNode::INPUT_POSE_2.into(),
    // );
    // graph.add_edge(
    //     "Blend".into(),
    //     BlendNode::OUTPUT.into(),
    //     "Loop".into(),
    //     LoopNode::INPUT.into(),
    // );

    graph.dot_to_tmp_file_and_open(None).unwrap();

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
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut velocity: Local<f32>,
    time: Res<Time>,
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

        let graph = animation_graphs.get_mut(animations.0[0].clone()).unwrap();

        let run_stride_length = 0.8;
        let walk_stride_length = 0.3;

        let blend_range = (1., 3.);

        let speed_fac_for_speed_alpha = |alpha: f32, speed: f32| {
            speed / (walk_stride_length * (1. - alpha) + run_stride_length * alpha)
        };

        let alpha_for_speed =
            |speed: f32| ((speed - blend_range.0) / (blend_range.1 - blend_range.0)).clamp(0., 1.);

        if keyboard_input.pressed(KeyCode::Up) {
            *velocity += 0.5 * time.delta_seconds();
            println!("velocity: {}", *velocity);
        }
        if keyboard_input.pressed(KeyCode::Down) {
            *velocity -= 0.5 * time.delta_seconds();
            println!("velocity: {}", *velocity);
        }

        *velocity = velocity.max(0.);

        let alpha = alpha_for_speed(*velocity);
        let k = speed_fac_for_speed_alpha(alpha, *velocity);

        graph.set_parameter("Blend Alpha".into(), alpha.into());
        graph.set_parameter("Speed factor".into(), k.into());
    }
}
