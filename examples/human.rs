use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::core::animated_scene::{AnimatedSceneBundle, AnimatedSceneInstance};
use bevy_animation_graph::prelude::*;
use bevy_egui_editor::{EditorCamera, EguiEditorPlugin};
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AnimationGraphPlugin)
        .add_plugins(EguiEditorPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_animation_control)
        .run();
}

#[derive(Component)]
struct Human;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(3., 3., 3.)
                .looking_at(Vec3::new(0.0, 0.875, 0.0), Vec3::Y),
            ..default()
        })
        .insert(EditorCamera);

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
            first_cascade_far_bound: 10.0,
            num_cascades: 3,
            minimum_distance: 0.3,
            maximum_distance: 100.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // Animated character
    commands.spawn((
        AnimatedSceneBundle {
            animated_scene: asset_server.load("animated_scenes/human.animscn.ron"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Human,
    ));

    println!("Controls:");
    println!("\tSPACE: Play/Pause animation");
    println!("\tR: Reset animation");
    println!("\tUp/Down: Increase/decrease movement speed");
}

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    human_character: Query<&AnimatedSceneInstance, With<Human>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    mut velocity: Local<f32>,
    mut direction: Local<Vec3>,
    time: Res<Time>,
) {
    let Ok(AnimatedSceneInstance { player_entity }) = human_character.get_single() else {
        return;
    };

    let Ok(mut player) = animation_players.get_mut(*player_entity) else {
        return;
    };

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

    if keyboard_input.pressed(KeyCode::Up) {
        *velocity += 0.5 * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::Down) {
        *velocity -= 0.5 * time.delta_seconds();
    }

    if *direction == Vec3::ZERO {
        *direction = Vec3::Z;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        *direction = (Quat::from_rotation_y(1. * time.delta_seconds()) * *direction).normalize();
    }
    if keyboard_input.pressed(KeyCode::Left) {
        *direction = (Quat::from_rotation_y(-1. * time.delta_seconds()) * *direction).normalize();
    }

    player.set_input_parameter("Target Speed", (*velocity).into());
    player.set_input_parameter("Target Direction", (*direction).into());
}
