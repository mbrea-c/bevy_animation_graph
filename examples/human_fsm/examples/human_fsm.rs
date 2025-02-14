extern crate bevy;
extern crate bevy_animation_graph;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::core::animated_scene::AnimatedSceneInstance;
use bevy_animation_graph::core::edge_data::AnimationEvent;
use bevy_animation_graph::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        }))
        .add_plugins(AnimationGraphPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
        })
        .insert_resource(Params::default())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_animation_control)
        .run();
}

#[derive(Resource)]
struct Params {
    pub speed: f32,
    pub direction: Vec3,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            speed: 1.0,
            direction: Vec3::Z,
        }
    }
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
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3., 3., 3.).looking_at(Vec3::new(0.0, 0.875, 0.0), Vec3::Y),
    ));

    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(5., 5.)))),
        MeshMaterial3d(materials.add(Color::from(LinearRgba::rgb(0.3, 0.5, 0.3)))),
    ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 10.0,
            num_cascades: 3,
            minimum_distance: 0.3,
            maximum_distance: 100.0,
            ..default()
        }
        .build(),
    ));

    // Animated character
    commands.spawn((
        AnimatedSceneHandle(asset_server.load("animated_scenes/fsm.animscn.ron")),
        Transform::from_xyz(0., 0., 0.),
        Human,
    ));

    println!("Controls:");
    println!("\tSPACE: Play/Pause animation");
    println!("\tR: Reset animation");
    println!("\tUp: Fire 'speed_up' animation event");
    println!("\tDown: Fire 'slow_down' animation event");
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    human_character: Query<&AnimatedSceneInstance, With<Human>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    mut params: ResMut<Params>,
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
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        player.reset();
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        player.send_event(AnimationEvent::TransitionToState("run".into()));
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        player.send_event(AnimationEvent::TransitionToState("walk".into()));
    }

    if params.direction == Vec3::ZERO {
        params.direction = Vec3::Z;
    }

    player.set_input_parameter("target_speed", params.speed.into());
}
