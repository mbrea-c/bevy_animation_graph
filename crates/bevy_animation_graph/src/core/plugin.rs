use super::animation_clip::Interpolation;
use super::edge_data::{AnimationEvent, EventQueue, SampledEvent};
use super::pose::Pose;
use super::prelude::{locate_animated_scene_player, GraphClip};
use super::skeleton::loader::SkeletonLoader;
use super::skeleton::Skeleton;
use super::state_machine::high_level::GlobalTransition;
use super::systems::apply_animation_to_targets;
use super::{
    animated_scene::{spawn_animated_scenes, AnimatedScene, AnimatedSceneLoader},
    animation_graph::loader::{AnimationGraphLoader, GraphClipLoader},
    edge_data::{BoneMask, DataSpec, DataValue},
    state_machine::high_level::{loader::StateMachineLoader, StateMachine},
    systems::{animation_player, animation_player_deferred_gizmos},
};
use crate::nodes::blend_space_node::BlendSpaceNode;
use crate::nodes::{
    AbsF32, AddF32, BlendMode, BlendNode, BlendSyncMode, ChainNode, ClampF32, ClipNode, CompareF32,
    DivF32, DummyNode, FSMNode, FireEventNode, FlipLRNode, GraphNode, LoopNode, MulF32,
    PaddingNode, RotationArcNode, RotationNode, SpeedNode, SubF32, TwoBoneIKNode,
};
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
        self.register_assets(app);
        self.register_types(app);
        self.register_nodes(app);

        app //
            .add_systems(PreUpdate, spawn_animated_scenes)
            .add_systems(
                PostUpdate,
                (
                    animation_player,
                    apply_animation_to_targets,
                    animation_player_deferred_gizmos,
                )
                    .chain()
                    .before(TransformSystem::TransformPropagate),
            )
            .add_observer(locate_animated_scene_player);
    }
}

impl AnimationGraphPlugin {
    /// Registers asset types and their loaders
    fn register_assets(&self, app: &mut App) {
        app.init_asset::<GraphClip>()
            .init_asset_loader::<GraphClipLoader>()
            .register_asset_reflect::<GraphClip>();
        app.init_asset::<AnimationGraph>()
            .init_asset_loader::<AnimationGraphLoader>()
            .register_asset_reflect::<AnimationGraph>();
        app.init_asset::<AnimatedScene>()
            .init_asset_loader::<AnimatedSceneLoader>()
            .register_asset_reflect::<AnimatedScene>();
        app.init_asset::<StateMachine>()
            .init_asset_loader::<StateMachineLoader>()
            .register_asset_reflect::<StateMachine>();
        app.init_asset::<Skeleton>()
            .init_asset_loader::<SkeletonLoader>()
            .register_asset_reflect::<Skeleton>();
    }

    /// Registers built-in animation node implementations
    fn register_nodes(&self, app: &mut App) {
        app //
            .register_type::<ClipNode>()
            .register_type::<DummyNode>()
            .register_type::<ChainNode>()
            .register_type::<BlendNode>()
            .register_type::<BlendSpaceNode>()
            .register_type::<FlipLRNode>()
            .register_type::<GraphNode>()
            .register_type::<LoopNode>()
            .register_type::<PaddingNode>()
            .register_type::<RotationNode>()
            .register_type::<SpeedNode>()
            .register_type::<TwoBoneIKNode>()
            .register_type::<AbsF32>()
            .register_type::<AddF32>()
            .register_type::<ClampF32>()
            .register_type::<DivF32>()
            .register_type::<MulF32>()
            .register_type::<SubF32>()
            .register_type::<CompareF32>()
            .register_type::<FireEventNode>()
            .register_type::<RotationArcNode>()
            .register_type::<FSMNode>();
        // .register_type::<ExtendSkeleton>()
        // .register_type::<IntoBoneSpaceNode>()
        // .register_type::<IntoGlobalSpaceNode>()
        // .register_type::<IntoCharacterSpaceNode>()
    }

    /// "Other" reflect registrations
    fn register_types(&self, app: &mut App) {
        app //
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
            .register_type::<BlendMode>()
            .register_type::<BlendSyncMode>()
            .register_type::<GlobalTransition>()
            .register_type::<()>()
            .register_type_data::<(), ReflectDefault>();
    }
}
