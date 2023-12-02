use super::{
    animation_graph::loader::{AnimationGraphLoader, GraphClipLoader},
    systems::animation_player,
};
use crate::prelude::{AnimationGraph, AnimationGraphPlayer, GraphClip};
use bevy::{prelude::*, transform::TransformSystem};

/// Adds animation support to an app
#[derive(Default)]
pub struct AnimationGraphPlugin;

impl Plugin for AnimationGraphPlugin {
    fn build(&self, app: &mut App) {
        app //
            .register_type::<AnimationGraph>()
            .register_asset_reflect::<AnimationGraph>()
            .register_type::<GraphClip>()
            .register_asset_reflect::<GraphClip>()
            .register_type::<AnimationGraphPlayer>()
            .init_asset::<GraphClip>()
            .init_asset_loader::<GraphClipLoader>()
            .init_asset::<AnimationGraph>()
            .init_asset_loader::<AnimationGraphLoader>()
            // .add_systems(PreUpdate, replace_animation_players)
            .add_systems(
                PostUpdate,
                animation_player.before(TransformSystem::TransformPropagate),
            );
    }
}
