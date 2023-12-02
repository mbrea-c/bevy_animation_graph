use super::{
    animation_graph::{
        EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
    },
    graph_context::{GraphContext, GraphContextTmp},
};
use crate::nodes::{
    add_f32::AddF32, blend_node::BlendNode, chain_node::ChainNode, clamp_f32::ClampF32,
    clip_node::ClipNode, dummy_node::DummyNode, flip_lr_node::FlipLRNode, loop_node::LoopNode,
    speed_node::SpeedNode, sub_f32::SubF32, DivF32, GraphNode, MulF32,
};
use bevy::{reflect::prelude::*, utils::HashMap};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

pub trait NodeLike: Send + Sync {
    fn parameter_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue>;
    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>>;
    fn time_pass(
        &self,
        input: TimeState,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate>;
    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue>;

    fn parameter_input_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec>;
    fn parameter_output_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec>;

    fn time_dependent_input_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec>;
    fn time_dependent_output_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec>;

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
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        self.node
            .map(|n| n.parameter_pass(inputs, name, path, context, context_tmp))
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        self.node
            .map(|n| n.duration_pass(inputs, name, path, context, context_tmp))
    }

    fn time_pass(
        &self,
        input: TimeState,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        self.node
            .map(|n| n.time_pass(input, name, path, context, context_tmp))
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        self.node
            .map(|n| n.time_dependent_pass(inputs, name, path, context, context_tmp))
    }

    fn parameter_input_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        self.node
            .map(|n| n.parameter_input_spec(context, context_tmp))
    }

    fn parameter_output_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        self.node
            .map(|n| n.parameter_output_spec(context, context_tmp))
    }

    fn time_dependent_input_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        self.node
            .map(|n| n.time_dependent_input_spec(context, context_tmp))
    }

    fn time_dependent_output_spec(
        &self,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        self.node
            .map(|n| n.time_dependent_output_spec(context, context_tmp))
    }

    fn display_name(&self) -> String {
        self.node.map(|n| n.display_name())
    }
}

#[derive(Reflect, Clone, Debug)]
pub enum AnimationNodeType {
    GraphInput(GraphInputNode),
    GraphOutput(GraphOutputNode),

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
            AnimationNodeType::GraphInput(n) => f(n),
            AnimationNodeType::GraphOutput(n) => f(n),
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
            AnimationNodeType::GraphInput(n) => f(n),
            AnimationNodeType::GraphOutput(n) => f(n),
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

    pub fn unwrap_input(&self) -> &GraphInputNode {
        match self {
            AnimationNodeType::GraphInput(n) => n,
            _ => panic!("Node is not a parameter node"),
        }
    }

    pub fn unwrap_input_mut(&mut self) -> &mut GraphInputNode {
        match self {
            AnimationNodeType::GraphInput(n) => n,
            _ => panic!("Node is not a parameter node"),
        }
    }

    pub fn unwrap_output(&self) -> &GraphOutputNode {
        match self {
            AnimationNodeType::GraphOutput(n) => n,
            _ => panic!("Node is not a parameter node"),
        }
    }

    pub fn unwrap_output_mut(&mut self) -> &mut GraphOutputNode {
        match self {
            AnimationNodeType::GraphOutput(n) => n,
            _ => panic!("Node is not a parameter node"),
        }
    }
}

#[derive(Reflect, Default, Clone, Debug)]
pub struct GraphInputNode {
    pub parameters: HashMap<String, EdgeValue>,
    pub time_dependent_spec: HashMap<String, EdgeSpec>,
    pub time_dependent: HashMap<String, EdgeValue>,
    pub durations: HashMap<String, Option<f32>>,
}

#[derive(Reflect, Default, Clone, Debug)]
pub struct GraphOutputNode {
    pub parameters: HashMap<String, EdgeSpec>,
    pub time_dependent: HashMap<String, EdgeSpec>,
}

impl GraphInputNode {
    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::GraphInput(self))
    }
}

impl GraphOutputNode {
    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::GraphOutput(self))
    }
}

impl NodeLike for GraphInputNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        self.parameters.clone()
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<NodeInput, Option<f32>>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        self.durations.clone()
    }

    fn time_pass(
        &self,
        _input: TimeState,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        HashMap::new()
    }

    fn time_dependent_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn parameter_input_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        self.parameters
            .iter()
            .map(|(k, v)| (k.clone(), EdgeSpec::from(v)))
            .collect()
    }

    fn time_dependent_input_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn time_dependent_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        self.time_dependent_spec.clone()
    }

    fn display_name(&self) -> String {
        "󰿄 Input".into()
    }
}

impl NodeLike for GraphOutputNode {
    fn parameter_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        inputs.clone()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        inputs.clone()
    }

    fn time_pass(
        &self,
        input: TimeState,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        self.time_dependent
            .iter()
            .map(|(k, _)| (k.clone(), input.update))
            .collect()
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        inputs.clone()
    }

    fn parameter_input_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        self.parameters.clone()
    }

    fn parameter_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        self.parameters.clone()
    }

    fn time_dependent_input_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        self.time_dependent.clone()
    }

    fn time_dependent_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        self.time_dependent.clone()
    }

    fn display_name(&self) -> String {
        "󰿅 Output".into()
    }
}
