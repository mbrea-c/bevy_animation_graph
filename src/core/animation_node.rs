use super::{
    animation_graph::{OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate},
    frame::PoseFrame,
};
use crate::{
    nodes::{
        add_f32::AddF32, blend_node::BlendNode, chain_node::ChainNode, clamp_f32::ClampF32,
        clip_node::ClipNode, dummy_node::DummyNode, flip_lr_node::FlipLRNode, loop_node::LoopNode,
        speed_node::SpeedNode, sub_f32::SubF32, DivF32, GraphNode, MulF32,
    },
    prelude::{PassContext, SpecContext},
};
use bevy::{
    reflect::prelude::*,
    utils::{HashMap, HashSet},
};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

pub type DurationData = Option<f32>;

pub trait NodeLike: Send + Sync {
    fn parameter_pass(
        &self,
        inputs: HashMap<PinId, ParamValue>,
        ctx: PassContext,
    ) -> HashMap<PinId, ParamValue>;
    fn duration_pass(
        &self,
        inputs: HashMap<PinId, Option<f32>>,
        ctx: PassContext,
    ) -> Option<DurationData>;
    fn time_pass(&self, input: TimeState, ctx: PassContext) -> HashMap<PinId, TimeUpdate>;
    fn time_dependent_pass(
        &self,
        inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame>;

    fn parameter_input_spec(&self, ctx: SpecContext) -> HashMap<PinId, OptParamSpec>;
    fn parameter_output_spec(&self, ctx: SpecContext) -> HashMap<PinId, ParamSpec>;
    fn pose_input_spec(&self, ctx: SpecContext) -> HashSet<PinId>;
    fn pose_output_spec(&self, ctx: SpecContext) -> bool;

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

#[derive(Reflect, Clone, Debug)]
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
    fn parameter_pass(
        &self,
        inputs: HashMap<PinId, ParamValue>,
        ctx: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        self.node.map(|n| n.parameter_pass(inputs, ctx))
    }

    fn duration_pass(
        &self,
        inputs: HashMap<PinId, Option<f32>>,
        ctx: PassContext,
    ) -> Option<Option<f32>> {
        self.node.map(|n| n.duration_pass(inputs, ctx))
    }

    fn time_pass(&self, input: TimeState, ctx: PassContext) -> HashMap<PinId, TimeUpdate> {
        self.node.map(|n| n.time_pass(input, ctx))
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame> {
        self.node.map(|n| n.time_dependent_pass(inputs, ctx))
    }

    fn parameter_input_spec(&self, ctx: SpecContext) -> HashMap<PinId, OptParamSpec> {
        self.node.map(|n| n.parameter_input_spec(ctx))
    }

    fn parameter_output_spec(&self, ctx: SpecContext) -> HashMap<PinId, ParamSpec> {
        self.node.map(|n| n.parameter_output_spec(ctx))
    }

    fn pose_input_spec(&self, ctx: SpecContext) -> HashSet<PinId> {
        self.node.map(|n| n.pose_input_spec(ctx))
    }

    fn pose_output_spec(&self, ctx: SpecContext) -> bool {
        self.node.map(|n| n.pose_output_spec(ctx))
    }

    fn display_name(&self) -> String {
        self.node.map(|n| n.display_name())
    }
}

#[derive(Reflect, Clone, Debug)]
pub enum AnimationNodeType {
    Clip(ClipNode),
    Blend(BlendNode),
    Chain(ChainNode),
    FlipLR(FlipLRNode),
    Loop(LoopNode),
    Speed(SpeedNode),
    AddF32(AddF32),
    MulF32(MulF32),
    DivF32(DivF32),
    SubF32(SubF32),
    ClampF32(ClampF32),
    // HACK: needs to be ignored for now due to:
    // https://github.com/bevyengine/bevy/issues/8965
    // Recursive reference causes reflection to fail
    Graph(#[reflect(ignore)] GraphNode),
    Custom(#[reflect(ignore)] CustomNode),
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
            AnimationNodeType::AddF32(n) => f(n),
            AnimationNodeType::MulF32(n) => f(n),
            AnimationNodeType::DivF32(n) => f(n),
            AnimationNodeType::SubF32(n) => f(n),
            AnimationNodeType::ClampF32(n) => f(n),
            AnimationNodeType::Graph(n) => f(n),
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
            AnimationNodeType::AddF32(n) => f(n),
            AnimationNodeType::MulF32(n) => f(n),
            AnimationNodeType::DivF32(n) => f(n),
            AnimationNodeType::SubF32(n) => f(n),
            AnimationNodeType::ClampF32(n) => f(n),
            AnimationNodeType::Graph(n) => f(n),
            AnimationNodeType::Custom(n) => {
                let mut nod = n.node.lock().unwrap();
                f(nod.deref_mut())
            }
        }
    }
}
