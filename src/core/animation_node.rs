use super::{
    animation_graph::{
        EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
    },
    graph_context::{GraphContext, GraphContextTmp},
};
use crate::nodes::{
    blend_node::BlendNode, chain_node::ChainNode, clip_node::ClipNode, dummy_node::DummyNode,
    flip_lr_node::FlipLRNode, loop_node::LoopNode, speed_node::SpeedNode,
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
    ) -> Option<f32>;
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

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec>;
    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec>;

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()>;

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec>;
    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec>;
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

    pub fn node_type_str(&self) -> String {
        match self.node {
            AnimationNodeType::Parameter(_) => "Parameter".into(),
            AnimationNodeType::Clip(_) => "Animation Clip".into(),
            AnimationNodeType::Blend(_) => "Blend".into(),
            AnimationNodeType::Chain(_) => "Chain".into(),
            AnimationNodeType::FlipLR(_) => "Flip LR".into(),
            AnimationNodeType::Loop(_) => "Loop".into(),
            AnimationNodeType::Speed(_) => "Speed".into(),
            AnimationNodeType::Custom(_) => "Custom".into(),
        }
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
    ) -> Option<f32> {
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

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        self.node.map(|n| n.parameter_input_spec())
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        self.node.map(|n| n.parameter_output_spec())
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        self.node.map(|n| n.duration_input_spec())
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        self.node.map(|n| n.time_dependent_input_spec())
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        self.node.map(|n| n.time_dependent_output_spec())
    }
}

#[derive(Reflect, Clone, Debug)]
pub enum AnimationNodeType {
    Parameter(ParameterNode),
    Clip(ClipNode),
    Blend(BlendNode),
    Chain(ChainNode),
    FlipLR(FlipLRNode),
    Loop(LoopNode),
    Speed(SpeedNode),
    // HACK: needs to be ignored for now due to:
    // https://github.com/bevyengine/bevy/issues/8965
    // Recursive reference causes reflection to fail
    // Graph(#[reflect(ignore)] AnimationGraph),
    Custom(#[reflect(ignore)] CustomNode),
}

impl AnimationNodeType {
    pub fn map<O, F>(&self, f: F) -> O
    where
        F: FnOnce(&dyn NodeLike) -> O,
    {
        match self {
            AnimationNodeType::Parameter(n) => f(n),
            AnimationNodeType::Clip(n) => f(n),
            AnimationNodeType::Blend(n) => f(n),
            AnimationNodeType::Chain(n) => f(n),
            AnimationNodeType::FlipLR(n) => f(n),
            AnimationNodeType::Loop(n) => f(n),
            AnimationNodeType::Speed(n) => f(n),
            // AnimationNodeType::Graph(n) => f(n),
            AnimationNodeType::Custom(n) => f(n.node.lock().unwrap().deref()),
        }
    }

    pub fn map_mut<O, F>(&mut self, mut f: F) -> O
    where
        F: FnMut(&mut dyn NodeLike) -> O,
    {
        match self {
            AnimationNodeType::Parameter(n) => f(n),
            AnimationNodeType::Clip(n) => f(n),
            AnimationNodeType::Blend(n) => f(n),
            AnimationNodeType::Chain(n) => f(n),
            AnimationNodeType::FlipLR(n) => f(n),
            AnimationNodeType::Loop(n) => f(n),
            AnimationNodeType::Speed(n) => f(n),
            // AnimationNodeType::Graph(n) => f(n),
            AnimationNodeType::Custom(n) => {
                let mut nod = n.node.lock().unwrap();
                f(nod.deref_mut())
            }
        }
    }

    pub fn unwrap_parameter(&self) -> &ParameterNode {
        match self {
            AnimationNodeType::Parameter(n) => n,
            _ => panic!("Node is not a parameter node"),
        }
    }

    pub fn unwrap_parameter_mut(&mut self) -> &mut ParameterNode {
        match self {
            AnimationNodeType::Parameter(n) => n,
            _ => panic!("Node is not a parameter node"),
        }
    }
}

#[derive(Reflect, Default, Clone, Debug)]
pub struct ParameterNode {
    pub parameters: HashMap<String, EdgeValue>,
}

impl ParameterNode {
    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Parameter(self))
    }
}

impl NodeLike for ParameterNode {
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
    ) -> Option<f32> {
        None
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

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        self.parameters
            .iter()
            .map(|(k, v)| (k.clone(), EdgeSpec::from(v)))
            .collect()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::new()
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }
}
