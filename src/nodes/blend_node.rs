use crate::core::animation_graph::{EdgeSpec, EdgeValue, NodeInput, NodeOutput};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::caches::{DurationCache, EdgePathCache, ParameterCache, TimeCache};
use crate::interpolation::linear::InterpolateLinear;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct BlendNode;

impl BlendNode {
    pub const INPUT_POSE_1: &'static str = "Input Pose 1";
    pub const INPUT_POSE_2: &'static str = "Input Pose 2";
    pub const FACTOR: &'static str = "Factor";
    pub const OUTPUT: &'static str = "Blend Pose";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self, name: String) -> AnimationNode {
        AnimationNode::new_from_nodetype(name, AnimationNodeType::Blend(self))
    }
}

impl NodeLike for BlendNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        _parameters: &ParameterCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> Option<f32> {
        let duration_1 = *inputs.get(Self::INPUT_POSE_1.into()).unwrap();
        let duration_2 = *inputs.get(Self::INPUT_POSE_2.into()).unwrap();

        match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        }
    }

    fn time_pass(
        &self,
        input: f32,
        _parameters: &ParameterCache,
        _durations: &DurationCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeInput, f32> {
        HashMap::from([
            (Self::INPUT_POSE_1.into(), input),
            (Self::INPUT_POSE_2.into(), input),
        ])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        parameters: &ParameterCache,
        _durations: &DurationCache,
        _time: &TimeCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let _ = _durations;
        let in_frame_1 = inputs
            .get(Self::INPUT_POSE_1)
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let in_frame_2 = inputs
            .get(Self::INPUT_POSE_2)
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let alpha = parameters
            .inputs
            .get(Self::FACTOR)
            .unwrap()
            .clone()
            .unwrap_f32();

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
        HashMap::from([
            (Self::INPUT_POSE_1.into(), ()),
            (Self::INPUT_POSE_2.into(), ()),
        ])
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([
            (Self::INPUT_POSE_1.into(), EdgeSpec::PoseFrame),
            (Self::INPUT_POSE_2.into(), EdgeSpec::PoseFrame),
        ])
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
