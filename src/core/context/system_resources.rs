use crate::prelude::{AnimationGraph, GraphClip};
use bevy::{
    asset::Assets,
    core::Name,
    ecs::{prelude::*, system::SystemParam},
    gizmos::gizmos::Gizmos,
    hierarchy::{Children, Parent},
    transform::prelude::*,
};

/// Contains temprary data such as references to assets, gizmos, etc.
#[derive(SystemParam)]
pub struct SystemResources<'w, 's> {
    pub graph_clip_assets: Res<'w, Assets<GraphClip>>,
    pub animation_graph_assets: Res<'w, Assets<AnimationGraph>>,
    // HACK: The mutable transform access is needed due to the query being reused by the apply_pose
    // function. This is due to bevy's restriction against conflicting system parameters
    pub transform_query: Query<'w, 's, (&'static mut Transform, &'static GlobalTransform)>,
    pub names_query: Query<'w, 's, &'static Name>,
    pub children_query: Query<'w, 's, &'static Children>,
    pub parent_query: Query<'w, 's, &'static Parent>,
    pub gizmos: Gizmos<'s>,
}
