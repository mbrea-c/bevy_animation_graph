use crate::{
    core::{skeleton::Skeleton, state_machine::high_level::StateMachine},
    prelude::{AnimationGraph, GraphClip},
};
use bevy::{
    asset::Assets,
    ecs::{prelude::*, system::SystemParam},
    transform::prelude::*,
};

/// Contains temprary data such as references to assets, gizmos, etc.
#[derive(SystemParam)]
pub struct SystemResources<'w, 's> {
    pub graph_clip_assets: Res<'w, Assets<GraphClip>>,
    pub animation_graph_assets: Res<'w, Assets<AnimationGraph>>,
    pub state_machine_assets: Res<'w, Assets<StateMachine>>,
    pub skeleton_assets: Res<'w, Assets<Skeleton>>,
    // HACK: The mutable transform access is needed due to the query being reused by the apply_pose
    // function. This is due to bevy's restriction against conflicting system parameters
    pub transform_query: Query<'w, 's, (&'static mut Transform, &'static GlobalTransform)>,
    pub names_query: Query<'w, 's, &'static Name>,
    pub children_query: Query<'w, 's, &'static Children>,
    pub parent_query: Query<'w, 's, &'static ChildOf>,
    #[cfg(feature = "physics_avian")]
    pub rigidbody_query: Query<'w, 's, &'static avian3d::prelude::RigidBody>,
}
