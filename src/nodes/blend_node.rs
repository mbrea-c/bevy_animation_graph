use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::graph_context::GraphContext;
use crate::interpolation::linear::InterpolateLinear;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct BlendNode;

impl BlendNode {
    pub const INPUT_1: &'static str = "Input Pose 1";
    pub const INPUT_2: &'static str = "Input Pose 2";
    pub const FACTOR: &'static str = "Factor";
    pub const OUTPUT: &'static str = "Blend Pose";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Blend(self))
    }
}

impl NodeLike for BlendNode {
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
        let duration_1 = *inputs.get(Self::INPUT_1.into()).unwrap();
        let duration_2 = *inputs.get(Self::INPUT_2.into()).unwrap();

        match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        }
    }

    fn time_pass(
        &self,
        input: TimeState,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
    ) -> HashMap<NodeInput, TimeUpdate> {
        HashMap::from([
            (Self::INPUT_1.into(), input.update),
            (Self::INPUT_2.into(), input.update),
        ])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        _path: &EdgePath,
        context: &mut GraphContext,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let in_frame_1 = inputs
            .get(Self::INPUT_1)
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let in_frame_2 = inputs
            .get(Self::INPUT_2)
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let alpha = context
            .get_parameters(name)
            .unwrap()
            .upstream
            .get(Self::FACTOR)
            .unwrap()
            .clone()
            .unwrap_f32();

        let time = context.get_times(name, _path).unwrap().downstream.time;

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(in_frame_1.interpolate_linear(&in_frame_2, alpha)),
        )])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::FACTOR.into(), EdgeSpec::F32)])
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::from([(Self::INPUT_1.into(), ()), (Self::INPUT_2.into(), ())])
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([
            (Self::INPUT_1.into(), EdgeSpec::PoseFrame),
            (Self::INPUT_2.into(), EdgeSpec::PoseFrame),
        ])
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
