extern crate bevy;
extern crate bevy_animation_graph;

use std::f32::consts::PI;

use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::{AnimationGraphPlugin, core::animated_scene::AnimatedSceneHandle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        }))
        .add_plugins(AnimationGraphPlugin::default())
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
            ..default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(9., 9., 9.).looking_at(Vec3::new(0.0, 3., 0.0), Vec3::Y),
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

    // Animated scene without retargeting required
    commands.spawn((
        AnimatedSceneHandle::new(asset_server.load("animated_scenes/snake_a.animscn.ron")),
        Transform::from_xyz(-2., 0., 0.),
    ));

    // Animated scene with retargeting applied
    // The retargeting is configured in the scene asset file
    commands.spawn((
        AnimatedSceneHandle::new(asset_server.load("animated_scenes/snake_b.animscn.ron")),
        Transform::from_xyz(2., 0., 0.),
    ));
}
