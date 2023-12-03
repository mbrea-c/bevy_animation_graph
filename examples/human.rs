use bevy::utils::HashMap;
use bevy::{gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::core::systems::replace_animation_players;
use bevy_animation_graph::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AnimationGraphPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
        })
        .insert_resource(ProcessedGraphs(vec![]))
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, replace_animation_players)
        .add_systems(
            Update,
            (setup_scene_once_loaded, keyboard_animation_control),
        )
        .run();
}

#[derive(Resource)]
struct RootGltf(Handle<Gltf>);

#[derive(Resource)]
struct GraphClips(HashMap<String, Handle<GraphClip>>);

#[derive(Resource)]
struct ProcessedGraphs(Vec<Handle<AnimationGraph>>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::new(0.0, 2.5, 0.0), Vec3::Y),
        ..default()
    });

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

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut players: Query<&mut AnimationGraphPlayer, Added<AnimationGraphPlayer>>,
    asset_server: Res<AssetServer>,
) {
    for mut player in &mut players {
        player.start(asset_server.load("animation_graphs/locomotion.animgraph.ron"));
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
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

        let Some(graph_handle) = player.get_animation_graph() else {
            continue;
        };

        let Some(graph) = animation_graphs.get_mut(graph_handle) else {
            continue;
        };

        let mut velocity_changed = false;
        if keyboard_input.pressed(KeyCode::Up) {
            *velocity += 0.5 * time.delta_seconds();
            velocity_changed = true;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            *velocity -= 0.5 * time.delta_seconds();
            velocity_changed = true;
        }

        *velocity = velocity.max(0.);

        if velocity_changed {
            println!("velocity: {}", *velocity);
        }

        graph.set_input_parameter("Target Speed", (*velocity).into());
    }
}
