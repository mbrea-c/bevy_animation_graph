use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::graph_context::{GraphContext, GraphContextTmp};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
pub struct ClampF32 {}

impl ClampF32 {
    pub const INPUT: &'static str = "F32 In";
    pub const CLAMP_MIN: &'static str = "Min";
    pub const CLAMP_MAX: &'static str = "Max";
    pub const OUTPUT: &'static str = "F32 Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::ClampF32(self))
    }
}

impl NodeLike for ClampF32 {
    fn parameter_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let input = inputs.get(Self::INPUT).unwrap().clone().unwrap_f32();
        let min = inputs.get(Self::CLAMP_MIN).unwrap().clone().unwrap_f32();
        let max = inputs.get(Self::CLAMP_MAX).unwrap().clone().unwrap_f32();

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::F32(input.clamp(min, max)))])
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<NodeInput, Option<f32>>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        HashMap::new()
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
        HashMap::from([
            (Self::INPUT.into(), EdgeSpec::F32),
            (Self::CLAMP_MIN.into(), EdgeSpec::F32),
            (Self::CLAMP_MAX.into(), EdgeSpec::F32),
        ])
    }

    fn parameter_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::F32)])
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
        HashMap::new()
    }

    fn display_name(&self) -> String {
        "Clamp".into()
    }
}
