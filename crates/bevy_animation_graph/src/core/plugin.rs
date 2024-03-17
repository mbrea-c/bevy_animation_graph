use super::animation_clip::Interpolation;
use super::{
    animated_scene::{
        process_animated_scenes, spawn_animated_scenes, AnimatedScene, AnimatedSceneLoader,
    },
    animation_graph::loader::{AnimationGraphLoader, GraphClipLoader},
    parameters::{BoneMask, ParamSpec, ParamValue},
    pose::PoseSpec,
    systems::{animation_player, animation_player_deferred_gizmos},
};
use crate::prelude::{
    config::{FlipConfig, FlipNameMapper, PatternMapper, PatternMapperSerial},
    AbsF32, AddF32, AnimationGraph, AnimationGraphPlayer, AnimationNodeType, BlendNode, ChainNode,
    ClampF32, ClipNode, DivF32, DummyNode, ExtendSkeleton, FlipLRNode, GraphClip, GraphNode,
    IntoBoneSpaceNode, IntoCharacterSpaceNode, IntoGlobalSpaceNode, LoopNode, MulF32,
    RotationArcNode, RotationNode, SpeedNode, SubF32, TwoBoneIKNode,
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
            .add_systems(PreUpdate, (spawn_animated_scenes, process_animated_scenes))
            .add_systems(
                PostUpdate,
                (animation_player, animation_player_deferred_gizmos)
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
            .register_type::<GraphClip>()
            .register_asset_reflect::<GraphClip>()
            .register_type::<AnimatedScene>()
            .register_asset_reflect::<AnimatedScene>()
            .register_type::<Interpolation>()
            .register_type::<AnimationGraphPlayer>()
            .register_type::<EntityPath>()
            .register_type::<BoneMask>()
            .register_type::<ParamValue>()
            .register_type::<ParamSpec>()
            .register_type::<PoseSpec>()
            .register_type::<AnimationNode>()
            .register_type::<AnimationNodeType>()
            .register_type::<FlipConfig>()
            .register_type::<FlipNameMapper>()
            .register_type::<PatternMapper>()
            .register_type::<PatternMapperSerial>()
        // --- Node registrations
        // ------------------------------------------
            .register_type::<BlendNode>()
            .register_type::<ChainNode>()
            .register_type::<ClipNode>()
            .register_type::<DummyNode>()
            .register_type::<FlipLRNode>()
            .register_type::<GraphNode>()
            .register_type::<LoopNode>()
            .register_type::<RotationNode>()
            .register_type::<SpeedNode>()
            .register_type::<TwoBoneIKNode>()
            .register_type::<AbsF32>()
            .register_type::<AddF32>()
            .register_type::<ClampF32>()
            .register_type::<DivF32>()
            .register_type::<MulF32>()
            .register_type::<SubF32>()
            .register_type::<RotationArcNode>()
            .register_type::<ExtendSkeleton>()
            .register_type::<IntoBoneSpaceNode>()
            .register_type::<IntoGlobalSpaceNode>()
            .register_type::<IntoCharacterSpaceNode>()
        // ------------------------------------------
        ;
    }
}
