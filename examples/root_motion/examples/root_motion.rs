//! # Root Motion Locomotion Example
//!
//! Demonstrates how to use root motion with an FSM-driven animation graph.
//!
//! The animation graph contains four states (idle, walk, run, jump) connected via
//! an FSM with blended transitions. The walk and run clips use `GroundPlane` root
//! motion extraction, which zeroes the root bone's XZ translation in the visual
//! pose and exposes it as a per-frame delta via [`RootMotionOutput`].
//!
//! The plugin's `extract_root_motion` system populates [`RootMotionOutput`] but does
//! **not** move the entity. This example's [`apply_root_motion`] system reads the
//! delta and applies it to the character's [`Transform`], showing how to integrate
//! root motion with your own movement logic.
//!
//! ## Setup
//!
//! This example requires Mixamo assets that cannot be redistributed. See
//! `examples/root_motion/README.md` for download and conversion instructions.

use std::f32::consts::PI;

use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use bevy_animation_graph::{
    AnimationGraphPlugin,
    core::{
        animated_scene::{AnimatedSceneHandle, AnimatedSceneInstance},
        animation_graph_player::AnimationGraphPlayer,
        edge_data::events::AnimationEvent,
        plugin::AnimationGraphSet,
        systems::RootMotionOutput,
    },
};

const SCENE: &str = "animated_scenes/locomotion.animscn.ron";
const TURN_SPEED: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "assets".to_string(),
            ..default()
        }))
        .add_plugins(AnimationGraphPlugin::default())
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
            ..default()
        })
        .add_systems(Startup, setup)
        // Run root motion application in the same schedule as the animation system
        // (FixedPostUpdate) to avoid jitter from frame-rate vs fixed-tick mismatches.
        .add_systems(
            FixedPostUpdate,
            apply_root_motion.in_set(AnimationGraphSet::PrePhysics),
        )
        .add_systems(Update, (locomotion_control, keyboard_control))
        .run();
}

/// Marker component for our character entity.
#[derive(Component)]
struct Character;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3., 2., -4.).looking_at(Vec3::new(0.0, 0.5, 5.0), Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(50., 50.)))),
        MeshMaterial3d(materials.add(Color::from(LinearRgba::rgb(0.3, 0.5, 0.3)))),
    ));

    // Directional light
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

    // Spawn the animated character.
    // `RootMotionOutput` tells the plugin to extract root motion deltas each frame.
    // The `AnimatedSceneHandle` loads the scene, spawns the model, and wires up
    // the animation graph. Once ready, `AnimatedSceneInstance` is added automatically.
    commands.spawn((
        AnimatedSceneHandle::new(asset_server.load(SCENE)),
        Transform::from_xyz(0., 0., 0.),
        Character,
        RootMotionOutput::default(),
    ));

    println!("Root Motion Locomotion Example");
    println!("Controls:");
    println!("\tW: Walk");
    println!("\tShift+W: Run");
    println!("\tA/D: Turn left/right");
    println!("\tSpace: Jump");
    println!("\tR: Reset");
}

/// Reads the root motion delta from [`RootMotionOutput`] and applies it to the
/// entity's [`Transform`]. The delta is in entity-local space, so we rotate it
/// by the entity's current facing direction to get world-space movement.
///
/// This is where you would swap in physics impulses or character controller
/// movement instead of direct transform manipulation.
fn apply_root_motion(mut query: Query<(&RootMotionOutput, &mut Transform), With<Character>>) {
    for (rm, mut transform) in &mut query {
        let world_delta = transform.rotation * rm.translation_delta;
        transform.translation += world_delta;
        transform.rotation *= rm.rotation_delta;
    }
}

/// Sends FSM transition events based on keyboard input.
///
/// The FSM handles blend durations and the jump-to-idle auto-transition internally
/// (via `MapEventsNode` converting `AnimationClipFinished` into a state transition).
/// This system just declares the desired state each frame.
///
/// Note: `AnimatedSceneInstance` is added asynchronously after the scene loads, so
/// we use `Query::single_mut()` which gracefully returns `Err` on early frames.
fn locomotion_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut character: Query<(&AnimatedSceneInstance, &mut Transform), With<Character>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
) {
    let Ok((scene_instance, mut transform)) = character.single_mut() else {
        return;
    };
    let Ok(mut player) = animation_players.get_mut(scene_instance.player_entity()) else {
        return;
    };

    // Jump: the FSM auto-returns to idle when the clip finishes
    if keyboard_input.just_pressed(KeyCode::Space) {
        player.send_event(AnimationEvent::TransitionToStateLabel("jump".into()));
    }

    // Locomotion: send the desired state every frame. The FSM ignores
    // transitions to the current state and handles blending for changes.
    let forward = keyboard_input.pressed(KeyCode::KeyW);
    let running = forward && keyboard_input.pressed(KeyCode::ShiftLeft);

    let target_state = if running {
        "run"
    } else if forward {
        "walk"
    } else {
        "idle"
    };
    player.send_event(AnimationEvent::TransitionToStateLabel(target_state.into()));

    // Turning is not driven by root motion; we rotate the entity directly.
    let dt = time.delta_secs();
    let turn = if keyboard_input.pressed(KeyCode::KeyA) {
        TURN_SPEED * dt
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        -TURN_SPEED * dt
    } else {
        0.0
    };
    if turn != 0.0 {
        transform.rotation *= Quat::from_rotation_y(turn);
    }
}

/// Reset the character's position and animation state.
fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    character: Query<&AnimatedSceneInstance, With<Character>>,
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    mut transforms: Query<&mut Transform, With<Character>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if let Ok(si) = character.single() {
            if let Ok(mut player) = animation_players.get_mut(si.player_entity()) {
                player.reset();
            }
        }
        if let Ok(mut transform) = transforms.single_mut() {
            transform.translation = Vec3::ZERO;
            transform.rotation = Quat::IDENTITY;
        }
    }
}
