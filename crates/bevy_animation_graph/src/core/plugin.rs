use super::animation_clip::Interpolation;
use super::edge_data::{AnimationEvent, EventQueue, SampledEvent};
use super::pose::Pose;
use super::prelude::GraphClip;
use super::skeleton::loader::SkeletonLoader;
use super::skeleton::Skeleton;
use super::systems::apply_animation_to_targets;
use super::{
    animated_scene::{
        process_animated_scenes, spawn_animated_scenes, AnimatedScene, AnimatedSceneLoader,
    },
    animation_graph::loader::{AnimationGraphLoader, GraphClipLoader},
    edge_data::{BoneMask, DataSpec, DataValue},
    state_machine::high_level::{loader::StateMachineLoader, StateMachine},
    systems::{animation_player, animation_player_deferred_gizmos},
};
use crate::node;
use crate::prelude::{
    config::{FlipConfig, FlipNameMapper, PatternMapper, PatternMapperSerial},
    AnimationGraph, AnimationGraphPlayer,
};
use crate::{core::animation_clip::EntityPath, prelude::AnimationNode};
use bevy::{prelude::*, transform::TransformSystem};

/// Adds animation support to an app
#[derive(Default)]
pub struct AnimationGraphPlugin;

impl Plugin for AnimationGraphPlugin {
    fn build(&self, app: &mut App) {
        self.register_types(app);
        app //
            .init_asset::<GraphClip>()
            .init_asset_loader::<GraphClipLoader>()
            .init_asset::<AnimationGraph>()
            .init_asset_loader::<AnimationGraphLoader>()
            .init_asset::<AnimatedScene>()
            .init_asset_loader::<AnimatedSceneLoader>()
            .init_asset::<StateMachine>()
            .init_asset_loader::<StateMachineLoader>()
            .init_asset::<Skeleton>()
            .init_asset_loader::<SkeletonLoader>()
            .add_systems(PreUpdate, (spawn_animated_scenes, process_animated_scenes))
            .add_systems(
                PostUpdate,
                (
                    animation_player,
                    apply_animation_to_targets,
                    animation_player_deferred_gizmos,
                )
                    .chain()
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

impl AnimationGraphPlugin {
    fn register_types(&self, app: &mut App) {
        app //
            .register_type::<AnimationGraph>()
            .register_asset_reflect::<AnimationGraph>()
            .register_type::<StateMachine>()
            .register_asset_reflect::<StateMachine>()
            .register_type::<GraphClip>()
            .register_asset_reflect::<GraphClip>()
            .register_type::<AnimatedScene>()
            .register_asset_reflect::<AnimatedScene>()
            .register_type::<Interpolation>()
            .register_type::<AnimationGraphPlayer>()
            .register_type::<EntityPath>()
            .register_type::<BoneMask>()
            .register_type::<Pose>()
            .register_type::<AnimationEvent>()
            .register_type::<SampledEvent>()
            .register_type::<EventQueue>()
            .register_type::<AnimationEvent>()
            .register_type::<SampledEvent>()
            .register_type::<DataValue>()
            .register_type::<DataSpec>()
            .register_type::<AnimationNode>()
            .register_type::<FlipConfig>()
            .register_type::<FlipNameMapper<PatternMapper>>()
            .register_type::<FlipNameMapper<PatternMapperSerial>>()
            .register_type::<PatternMapper>()
            .register_type::<PatternMapperSerial>()
            .register_type::<()>()
            .register_type_data::<(), ReflectDefault>();
        node::register_types(app);
    }
}
