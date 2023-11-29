use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::graph_context::GraphContext;
use crate::flipping::FlipXBySuffix;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct FlipLRNode {}

impl FlipLRNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Flip Pose";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::FlipLR(self))
    }
}

impl NodeLike for FlipLRNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
    ) -> Option<f32> {
        *inputs.get(Self::INPUT.into()).unwrap()
    }

    fn time_pass(
        &self,
        input: TimeState,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
    ) -> HashMap<NodeInput, TimeUpdate> {
        // Propagate the time update without modification
        HashMap::from([(Self::INPUT.into(), input.update)])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        let flipped_pose_frame = in_pose_frame.flipped_by_suffix(" R".into(), " L".into());

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(flipped_pose_frame),
        )])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::from([(Self::INPUT.into(), ())])
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::INPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
