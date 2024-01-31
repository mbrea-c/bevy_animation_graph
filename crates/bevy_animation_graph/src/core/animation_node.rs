use super::{
    animation_graph::{PinId, PinMap, TimeUpdate},
    duration_data::DurationData,
    errors::GraphError,
    frame::{PoseFrame, PoseSpec},
    parameters::{OptParamSpec, ParamSpec, ParamValue},
};
use crate::{
    nodes::{
        blend_node::BlendNode, chain_node::ChainNode, clip_node::ClipNode, dummy_node::DummyNode,
        flip_lr_node::FlipLRNode, loop_node::LoopNode, speed_node::SpeedNode, AbsF32, AddF32,
        ClampF32, DivF32, ExtendSkeleton, GraphNode, IntoCharacterSpaceNode, MulF32,
        RotationArcNode, RotationNode, SubF32,
    },
    prelude::{IntoBoneSpaceNode, IntoGlobalSpaceNode, PassContext, SpecContext, TwoBoneIKNode},
};
use bevy::{reflect::prelude::*, utils::HashMap};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

pub trait NodeLike: Send + Sync + Reflect {
    fn parameter_pass(&self, _ctx: PassContext) -> Result<HashMap<PinId, ParamValue>, GraphError> {
        Ok(HashMap::new())
    }

    fn duration_pass(&self, _ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(None)
    }

    fn pose_pass(
        &self,
        _time_update: TimeUpdate,
        _ctx: PassContext,
    ) -> Result<Option<PoseFrame>, GraphError> {
        Ok(None)
    }

    fn parameter_input_spec(&self, _ctx: SpecContext) -> PinMap<OptParamSpec> {
        PinMap::new()
    }

    fn parameter_output_spec(&self, _ctx: SpecContext) -> PinMap<ParamSpec> {
        PinMap::new()
    }

    fn pose_input_spec(&self, _ctx: SpecContext) -> PinMap<PoseSpec> {
        PinMap::new()
    }

    /// Specify whether or not a node outputs a pose, and which space the pose is in
    fn pose_output_spec(&self, _ctx: SpecContext) -> Option<PoseSpec> {
        None
    }

    /// The name of this node.
    fn display_name(&self) -> String;
}

#[derive(Clone)]
pub struct CustomNode {
    pub node: Arc<Mutex<dyn NodeLike>>,
}

impl CustomNode {
    pub fn new(node: impl NodeLike + 'static) -> Self {
        Self {
            node: Arc::new(Mutex::new(node)),
        }
    }
}

impl Default for CustomNode {
    fn default() -> Self {
        Self {
            node: Arc::new(Mutex::new(DummyNode::new())),
        }
    }
}

impl std::fmt::Debug for CustomNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomNode")
    }
}

#[derive(Reflect, Clone, Debug, Default)]
pub struct AnimationNode {
    pub name: String,
    pub node: AnimationNodeType,
}

impl AnimationNode {
    pub fn new_from_nodetype(name: String, node: AnimationNodeType) -> Self {
        Self { name, node }
    }
}

impl NodeLike for AnimationNode {
    fn parameter_pass(&self, ctx: PassContext) -> Result<HashMap<PinId, ParamValue>, GraphError> {
        self.node.map(|n| n.parameter_pass(ctx))
    }

    fn duration_pass(&self, ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        self.node.map(|n| n.duration_pass(ctx))
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        ctx: PassContext,
    ) -> Result<Option<PoseFrame>, GraphError> {
        self.node.map(|n| n.pose_pass(input, ctx))
    }

    fn parameter_input_spec(&self, ctx: SpecContext) -> PinMap<OptParamSpec> {
        self.node.map(|n| n.parameter_input_spec(ctx))
    }

    fn parameter_output_spec(&self, ctx: SpecContext) -> PinMap<ParamSpec> {
        self.node.map(|n| n.parameter_output_spec(ctx))
    }

    fn pose_input_spec(&self, ctx: SpecContext) -> PinMap<PoseSpec> {
        self.node.map(|n| n.pose_input_spec(ctx))
    }

    fn pose_output_spec(&self, ctx: SpecContext) -> Option<PoseSpec> {
        self.node.map(|n| n.pose_output_spec(ctx))
    }

    fn display_name(&self) -> String {
        self.node.map(|n| n.display_name())
    }
}

#[derive(Reflect, Clone, Debug)]
#[reflect(Default)]
pub enum AnimationNodeType {
    // --- Dummy (default no-op node)
    // ------------------------------------------------
    Dummy(DummyNode),
    // ------------------------------------------------

    // --- Pose Nodes
    // ------------------------------------------------
    Clip(ClipNode),
    Blend(BlendNode),
    Chain(ChainNode),
    FlipLR(FlipLRNode),
    Loop(LoopNode),
    Speed(SpeedNode),
    Rotation(RotationNode),
    // ------------------------------------------------

    // --- Pose space conversion
    // ------------------------------------------------
    IntoBoneSpace(IntoBoneSpaceNode),
    IntoCharacterSpace(IntoCharacterSpaceNode),
    IntoGlobalSpace(IntoGlobalSpaceNode),
    ExtendSkeleton(ExtendSkeleton),
    // ------------------------------------------------

    // --- IK space conversion
    // ------------------------------------------------
    TwoBoneIK(TwoBoneIKNode),
    // ------------------------------------------------

    // --- F32 arithmetic nodes
    // ------------------------------------------------
    AddF32(AddF32),
    MulF32(MulF32),
    DivF32(DivF32),
    SubF32(SubF32),
    ClampF32(ClampF32),
    AbsF32(AbsF32),
    // ------------------------------------------------

    // --- Vec3 arithmetic nodes
    // ------------------------------------------------
    RotationArc(RotationArcNode),
    // ------------------------------------------------
    // HACK: needs to be ignored for now due to:
    // https://github.com/bevyengine/bevy/issues/8965
    // Recursive reference causes reflection to fail
    Graph(#[reflect(ignore)] GraphNode),
    Custom(#[reflect(ignore)] CustomNode),
}

impl Default for AnimationNodeType {
    fn default() -> Self {
        Self::Dummy(DummyNode::new())
    }
}

impl AnimationNodeType {
    pub fn map<O, F>(&self, f: F) -> O
    where
        F: FnOnce(&dyn NodeLike) -> O,
    {
        match self {
            AnimationNodeType::Clip(n) => f(n),
            AnimationNodeType::Blend(n) => f(n),
            AnimationNodeType::Chain(n) => f(n),
            AnimationNodeType::FlipLR(n) => f(n),
            AnimationNodeType::Loop(n) => f(n),
            AnimationNodeType::Speed(n) => f(n),
            AnimationNodeType::Rotation(n) => f(n),
            AnimationNodeType::AddF32(n) => f(n),
            AnimationNodeType::MulF32(n) => f(n),
            AnimationNodeType::DivF32(n) => f(n),
            AnimationNodeType::SubF32(n) => f(n),
            AnimationNodeType::ClampF32(n) => f(n),
            AnimationNodeType::AbsF32(n) => f(n),
            AnimationNodeType::RotationArc(n) => f(n),
            AnimationNodeType::Graph(n) => f(n),
            AnimationNodeType::IntoBoneSpace(n) => f(n),
            AnimationNodeType::IntoCharacterSpace(n) => f(n),
            AnimationNodeType::IntoGlobalSpace(n) => f(n),
            AnimationNodeType::ExtendSkeleton(n) => f(n),
            AnimationNodeType::TwoBoneIK(n) => f(n),
            AnimationNodeType::Dummy(n) => f(n),
            AnimationNodeType::Custom(n) => f(n.node.lock().unwrap().deref()),
        }
    }

    pub fn map_mut<O, F>(&mut self, mut f: F) -> O
    where
        F: FnMut(&mut dyn NodeLike) -> O,
    {
        match self {
            AnimationNodeType::Clip(n) => f(n),
            AnimationNodeType::Blend(n) => f(n),
            AnimationNodeType::Chain(n) => f(n),
            AnimationNodeType::FlipLR(n) => f(n),
            AnimationNodeType::Loop(n) => f(n),
            AnimationNodeType::Speed(n) => f(n),
            AnimationNodeType::Rotation(n) => f(n),
            AnimationNodeType::AddF32(n) => f(n),
            AnimationNodeType::MulF32(n) => f(n),
            AnimationNodeType::DivF32(n) => f(n),
            AnimationNodeType::SubF32(n) => f(n),
            AnimationNodeType::ClampF32(n) => f(n),
            AnimationNodeType::AbsF32(n) => f(n),
            AnimationNodeType::RotationArc(n) => f(n),
            AnimationNodeType::Graph(n) => f(n),
            AnimationNodeType::IntoBoneSpace(n) => f(n),
            AnimationNodeType::IntoCharacterSpace(n) => f(n),
            AnimationNodeType::IntoGlobalSpace(n) => f(n),
            AnimationNodeType::ExtendSkeleton(n) => f(n),
            AnimationNodeType::TwoBoneIK(n) => f(n),
            AnimationNodeType::Dummy(n) => f(n),
            AnimationNodeType::Custom(n) => {
                let mut nod = n.node.lock().unwrap();
                f(nod.deref_mut())
            }
        }
    }

    pub fn inner_reflect(&mut self) -> &mut dyn Reflect {
        match self {
            AnimationNodeType::Clip(n) => n,
            AnimationNodeType::Blend(n) => n,
            AnimationNodeType::Chain(n) => n,
            AnimationNodeType::FlipLR(n) => n,
            AnimationNodeType::Loop(n) => n,
            AnimationNodeType::Speed(n) => n,
            AnimationNodeType::Rotation(n) => n,
            AnimationNodeType::IntoBoneSpace(n) => n,
            AnimationNodeType::IntoCharacterSpace(n) => n,
            AnimationNodeType::IntoGlobalSpace(n) => n,
            AnimationNodeType::ExtendSkeleton(n) => n,
            AnimationNodeType::TwoBoneIK(n) => n,
            AnimationNodeType::AddF32(n) => n,
            AnimationNodeType::MulF32(n) => n,
            AnimationNodeType::DivF32(n) => n,
            AnimationNodeType::SubF32(n) => n,
            AnimationNodeType::ClampF32(n) => n,
            AnimationNodeType::AbsF32(n) => n,
            AnimationNodeType::RotationArc(n) => n,
            AnimationNodeType::Graph(n) => n,
            AnimationNodeType::Dummy(n) => n,
            AnimationNodeType::Custom(_) => todo!(),
        }
    }
}
