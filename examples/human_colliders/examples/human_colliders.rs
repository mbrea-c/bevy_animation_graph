extern crate bevy;
extern crate bevy_animation_graph;

use avian3d::PhysicsPlugins;
use avian3d::prelude::{PhysicsDebugPlugin, SpatialQuery, SpatialQueryFilter};
use bevy::window::PrimaryWindow;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::core::animated_scene::AnimatedSceneInstance;
use bevy_animation_graph::core::colliders::core::ColliderLabel;
use bevy_animation_graph::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(AnimationGraphPlugin::default())
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
            ..default()
        })
        .insert_resource(Params::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_animation_control, raycast_ui))
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
        AnimatedSceneHandle::new(asset_server.load("animated_scenes/human_colliders.animscn.ron")),
        Transform::from_xyz(0., 0., 0.),
        Human,
    ));

    // Collider label text
    commands.spawn(Text::new(""));

    println!("Controls:");
    println!("\tSPACE: Play/Pause animation");
    println!("\tR: Reset animation");
    println!("\tUp/Down: Increase/decrease movement speed");
    println!("\tLeft/Right: Rotate character");
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    human_character: Query<&AnimatedSceneInstance, With<Human>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    mut params: ResMut<Params>,
    time: Res<Time>,
) {
    let Ok(player_entity) = human_character.single().map(|i| i.player_entity()) else {
        return;
    };

    let Ok(mut player) = animation_players.get_mut(player_entity) else {
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
        params.speed += 0.5 * time.delta_secs();
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        params.speed -= 0.5 * time.delta_secs();
    }

    if params.direction == Vec3::ZERO {
        params.direction = Vec3::Z;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        params.direction =
            (Quat::from_rotation_y(1. * time.delta_secs()) * params.direction).normalize();
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        params.direction =
            (Quat::from_rotation_y(-1. * time.delta_secs()) * params.direction).normalize();
    }

    player.set_input_parameter("target_speed", params.speed.into());
    player.set_input_parameter("target_direction", params.direction.into());
}

fn raycast_ui(
    spatial_query: SpatialQuery,
    main_camera: Single<(&Camera, &GlobalTransform)>,
    main_window: Single<&Window, With<PrimaryWindow>>,
    collider_label_query: Query<&ColliderLabel>,
    debug_text: Single<(&mut Text, &mut Node)>,
) {
    let (camera, camera_transform) = main_camera.into_inner();
    let cursor = main_window.cursor_position().unwrap_or_default();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };

    let (mut debug_text, mut debug_node) = debug_text.into_inner();

    if let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        100000.,
        true,
        &SpatialQueryFilter::DEFAULT,
    ) && let Ok(label) = collider_label_query.get(hit.entity)
    {
        debug_text.0 = label.0.clone();
        debug_node.position_type = PositionType::Absolute;
        debug_node.top = Val::Px(cursor.y - 5.);
        debug_node.left = Val::Px(cursor.x + 5.);
    } else {
        debug_text.0 = "".into();
    }
}
