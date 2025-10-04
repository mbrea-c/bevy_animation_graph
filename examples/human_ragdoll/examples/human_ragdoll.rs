extern crate bevy;
extern crate bevy_animation_graph;

use avian3d::PhysicsPlugins;
use avian3d::prelude::{Collider, RigidBody};
use bevy::color::palettes::css::GREEN;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::core::animated_scene::AnimatedSceneInstance;
use bevy_animation_graph::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(avian3d::prelude::PhysicsDebugPlugin::default())
        .add_plugins(AnimationGraphPlugin)
        // .add_plugins((
        //     bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        //     bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        // ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
            ..default()
        })
        .insert_resource(Params::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (find_target, update_params, update_animation_player).chain(),
        )
        .run();
}

#[derive(Resource)]
struct Params {
    pub speed: f32,
    pub real_speed: f32,
    pub target_angle: f32,
    pub angle: f32,
    pub target_position: Vec3,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            speed: 3.0,
            angle: 0.,
            target_angle: 0.,
            target_position: Vec3::ZERO,
            position: Vec3::ZERO,
            real_speed: 0.,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
struct Human;

#[derive(Component)]
struct Label;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3., 9., 3.).looking_at(Vec3::new(0.0, 0.875, 0.0), Vec3::Y),
    ));

    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(20., 20.)))),
        MeshMaterial3d(materials.add(Color::from(LinearRgba::rgb(0.3, 0.5, 0.3)))),
        Collider::cuboid(40., 0.1, 40.),
        RigidBody::Static,
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
        AnimatedSceneHandle::new(asset_server.load("animated_scenes/human_ragdoll.animscn.ron")),
        Transform::from_xyz(0., 0., 0.),
        Human,
    ));

    commands.spawn((
        Text("test".into()),
        TextFont {
            font_size: 55.,
            ..default()
        },
        Label,
    ));

    println!("Controls:");
    println!("\tSPACE: Play/Pause animation");
    println!("\tR: Reset animation");
    println!("\tUp/Down: Increase/decrease movement speed");
    println!("\tLeft/Right: Rotate character");
}

fn update_animation_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut human_character: Query<(&AnimatedSceneInstance, &mut Transform), With<Human>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    mut params: ResMut<Params>,
    mut text: Single<&mut Text, With<Label>>,
    time: Res<Time>,
) {
    let Ok((instance, mut transform)) = human_character.single_mut() else {
        return;
    };

    transform.translation = params.position;

    let Ok(mut player) = animation_players.get_mut(instance.player_entity()) else {
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

    if keyboard_input.just_pressed(KeyCode::KeyT) {
        player.ragdoll_enabled = !player.ragdoll_enabled;
    }

    if player.ragdoll_enabled {
        text.0 = "Right arm: Dynamic (physics on)".into();
    } else {
        text.0 = "Right arm: Kinematic (follows pose)".into();
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        params.speed += 1.5 * time.delta_secs();
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        params.speed -= 1.5 * time.delta_secs();
    }

    player.set_input_parameter("target_speed", params.real_speed.into());
    player.set_input_parameter(
        "target_direction",
        (Quat::from_rotation_y(-params.angle) * Vec3::X).into(),
    );
}

fn find_target(
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut gizmos: Gizmos,
    mut params: ResMut<Params>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK

    let Ok((camera, camera_transform)) = q_camera.single() else {
        warn!("Cannot get mouse pos, there isn't a single camera!");
        return;
    };

    if let Ok(window) = q_window.single() {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .and_then(|ray| {
                Some(ray.get_point(ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?))
            })
        {
            params.target_position = world_position;
            gizmos.sphere(Isometry3d::from_translation(world_position), 0.25, GREEN);
        }
    }
}

fn update_params(mut params: ResMut<Params>, time: Res<Time>) {
    let delta = params.target_position - params.position;
    let direction = delta.normalize_or_zero();

    let target_velocity = (direction * params.speed) * delta.length().clamp(0., 2.) / 2.;

    let velocity_delta = target_velocity - params.velocity;
    let velocity_delta_length = velocity_delta.length();
    let velocity_delta_dir = velocity_delta.normalize_or_zero();

    params.velocity = params.velocity
        + (velocity_delta_dir * 6. * time.delta_secs()).clamp_length_max(velocity_delta_length);

    let direction = params.velocity.normalize_or_zero();
    params.target_angle = direction.z.atan2(direction.x);
    params.real_speed = params.velocity.length();
    params.position = params.position + params.velocity * time.delta_secs();

    //let delta_angle = (PI + params.target_angle - params.angle).rem_euclid(2. * PI) - PI;
    let delta_angle = (params.target_angle - params.angle)
        .sin()
        .atan2((params.target_angle - params.angle).cos());
    let angle_sign = if delta_angle < 0. { -1. } else { 1. };
    params.angle = params.angle
        + (angle_sign * 4. * time.delta_secs()).clamp(-delta_angle.abs(), delta_angle.abs());
    params.angle = (params.angle + PI).rem_euclid(2. * PI) - PI;
}
