//! Loads animations from a skinned glTF, spawns many of them, and plays the
//! animation to stress test skinned meshes.
//!
//! Adapted from: https://github.com/bevyengine/bevy/blob/73970d0c1215ee2f290513da7aed2e619a77bd7b/examples/stress_tests/many_foxes.rs

use std::f32::consts::PI;

use argh::FromArgs;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    window::{PresentMode, WindowResolution},
    winit::{UpdateMode, WinitSettings},
};
use bevy_animation_graph::prelude::{AnimatedSceneHandle, AnimationGraphPlugin};

#[derive(FromArgs, Resource)]
/// `many_foxes` stress test
struct Args {
    /// whether all foxes run in sync.
    #[argh(switch)]
    sync: bool,

    /// total number of foxes.
    #[argh(option, default = "1000")]
    count: usize,
}

#[derive(Resource)]
struct Foxes {
    count: usize,
    speed: f32,
    moving: bool,
    sync: bool,
}

fn main() {
    // `from_env` panics on the web
    #[cfg(not(target_arch = "wasm32"))]
    let args: Args = argh::from_env();
    #[cfg(target_arch = "wasm32")]
    let args = Args::from_args(&[], &[]).unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "🦊🦊🦊 Many Foxes! 🦊🦊🦊".into(),
                        present_mode: PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(1920.0, 1080.0)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(AnimationGraphPlugin::default())
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .insert_resource(Foxes {
            count: args.count,
            speed: 2.0,
            moving: true,
            sync: args.sync,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup_scene_once_loaded,
                keyboard_animation_control,
                update_fox_rings.after(keyboard_animation_control),
            ),
        )
        .run();
}

#[derive(Resource)]
struct Animations {
    node_indices: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

const RING_SPACING: f32 = 2.0;
const FOX_SPACING: f32 = 2.0;

#[derive(Component, Clone, Copy)]
enum RotationDirection {
    CounterClockwise,
    Clockwise,
}

impl RotationDirection {
    fn sign(&self) -> f32 {
        match self {
            RotationDirection::CounterClockwise => 1.0,
            RotationDirection::Clockwise => -1.0,
        }
    }
}

#[derive(Component)]
struct Ring {
    radius: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    foxes: Res<Foxes>,
) {
    warn!(include_str!("warning_string.txt"));

    // Foxes
    // Concentric rings of foxes, running in opposite directions. The rings are spaced at 2m radius intervals.
    // The foxes in each ring are spaced at least 2m apart around its circumference.'

    // NOTE: This fox model faces +z
    let fox_handle = asset_server.load("animated_scenes/fox.animscn.ron");

    let ring_directions = [
        (
            Quat::from_rotation_y(PI),
            RotationDirection::CounterClockwise,
        ),
        (Quat::IDENTITY, RotationDirection::Clockwise),
    ];

    let mut ring_index = 0;
    let mut radius = RING_SPACING;
    let mut foxes_remaining = foxes.count;

    info!("Spawning {} foxes...", foxes.count);

    while foxes_remaining > 0 {
        let (base_rotation, ring_direction) = ring_directions[ring_index % 2];
        let ring_parent = commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                ring_direction,
                Ring { radius },
            ))
            .id();

        let circumference = PI * 2. * radius;
        let foxes_in_ring = ((circumference / FOX_SPACING) as usize).min(foxes_remaining);
        let fox_spacing_angle = circumference / (foxes_in_ring as f32 * radius);

        for fox_i in 0..foxes_in_ring {
            let fox_angle = fox_i as f32 * fox_spacing_angle;
            let (s, c) = ops::sin_cos(fox_angle);
            let (x, z) = (radius * c, radius * s);

            commands.entity(ring_parent).with_children(|builder| {
                builder.spawn((
                    AnimatedSceneHandle::new(fox_handle.clone()),
                    Transform::from_xyz(x, 0.0, z)
                        .with_scale(Vec3::splat(0.01))
                        .with_rotation(base_rotation * Quat::from_rotation_y(-fox_angle)),
                ));
            });
        }

        foxes_remaining -= foxes_in_ring;
        radius += RING_SPACING;
        ring_index += 1;
    }

    // Camera
    let zoom = 0.8;
    let translation = Vec3::new(
        radius * 1.25 * zoom,
        radius * 0.5 * zoom,
        radius * 1.5 * zoom,
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(translation)
            .looking_at(0.2 * Vec3::new(translation.x, 0.0, translation.z), Vec3::Y),
    ));

    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5000.0, 5000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 0.9 * radius,
            maximum_distance: 2.8 * radius,
            ..default()
        }
        .build(),
    ));

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - return: change animation");
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    foxes: Res<Foxes>,
    mut commands: Commands,
    mut player: Query<(Entity, &mut AnimationPlayer)>,
    mut done: Local<bool>,
) {
    if !*done && player.iter().len() == foxes.count {
        for (entity, mut player) in &mut player {
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(animations.graph.clone()))
                .insert(AnimationTransitions::new());

            let playing_animation = player.play(animations.node_indices[0]).repeat();
            if !foxes.sync {
                playing_animation.seek_to(entity.index() as f32 / 10.0);
            }
        }
        *done = true;
    }
}

fn update_fox_rings(
    time: Res<Time>,
    foxes: Res<Foxes>,
    mut rings: Query<(&Ring, &RotationDirection, &mut Transform)>,
) {
    if !foxes.moving {
        return;
    }

    let dt = time.delta_secs();
    for (ring, rotation_direction, mut transform) in &mut rings {
        let angular_velocity = foxes.speed / ring.radius;
        transform.rotate_y(rotation_direction.sign() * angular_velocity * dt);
    }
}

fn keyboard_animation_control(keyboard_input: Res<ButtonInput<KeyCode>>, mut foxes: ResMut<Foxes>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        foxes.moving = !foxes.moving;
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        foxes.speed *= 1.25;
    }

    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        foxes.speed *= 0.8;
    }

    // if keyboard_input.just_pressed(KeyCode::Enter) {
    //     *current_animation = (*current_animation + 1) % animations.node_indices.len();
    // }

    // for (mut player, mut transitions) in &mut animation_player {
    //     if keyboard_input.just_pressed(KeyCode::Space) {
    //         if player.all_paused() {
    //             player.resume_all();
    //         } else {
    //             player.pause_all();
    //         }
    //     }

    //     if keyboard_input.just_pressed(KeyCode::ArrowUp) {
    //         player.adjust_speeds(1.25);
    //     }

    //     if keyboard_input.just_pressed(KeyCode::ArrowDown) {
    //         player.adjust_speeds(0.8);
    //     }

    //     if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
    //         player.seek_all_by(-0.1);
    //     }

    //     if keyboard_input.just_pressed(KeyCode::ArrowRight) {
    //         player.seek_all_by(0.1);
    //     }

    //     if keyboard_input.just_pressed(KeyCode::Enter) {
    //         transitions
    //             .play(
    //                 &mut player,
    //                 animations.node_indices[*current_animation],
    //                 Duration::from_millis(250),
    //             )
    //             .repeat();
    //     }
    // }
}
