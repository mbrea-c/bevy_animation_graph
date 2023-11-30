use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, CustomNode, NodeLike};
use crate::core::graph_context::{GraphContext, GraphContextTmp};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
pub struct DummyNode {}

impl DummyNode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: String) -> AnimationNode {
        AnimationNode::new_from_nodetype(name, AnimationNodeType::Custom(CustomNode::new(self)))
    }
}

impl NodeLike for DummyNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
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
        HashMap::new()
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
