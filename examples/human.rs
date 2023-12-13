use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::core::animated_scene::AnimatedSceneBundle;
use bevy_animation_graph::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AnimationGraphPlugin)
        .add_plugins(bevy_egui_editor::EguiEditorPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_animation_control)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // Animated character
    commands.spawn(AnimatedSceneBundle {
        animated_scene: asset_server.load("animated_scenes/character.animscn.ron"),
        transform: Transform::from_xyz(0.0, 2.4, 0.0),
        ..default()
    });
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
