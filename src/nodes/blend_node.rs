use crate::{
    animation::{AnimationNode, EdgeSpec, EdgeValue, NodeInput, NodeLike, NodeOutput},
    interpolation::linear::InterpolateLinear,
};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct BlendNode;

impl BlendNode {
    pub const INPUT_POSE_1: &'static str = "Input Pose 1";
    pub const INPUT_POSE_2: &'static str = "Input Pose 2";
    pub const FACTOR: &'static str = "Factor";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self) -> AnimationNode {
        AnimationNode::Blend(self)
    }
}

impl NodeLike for BlendNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        let duration_1 = *input_durations.get(Self::INPUT_POSE_1.into()).unwrap();
        let duration_2 = *input_durations.get(Self::INPUT_POSE_2.into()).unwrap();

        match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        }
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        HashMap::from([
            (Self::INPUT_POSE_1.into(), time),
            (Self::INPUT_POSE_2.into(), time),
            (Self::FACTOR.into(), 0.),
        ])
    }

    fn backward(
        &self,
        _time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
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
        let alpha = inputs.get(Self::FACTOR).unwrap().clone().unwrap_f32();

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(in_frame_1.interpolate_linear(&in_frame_2, alpha)),
        )])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([
            (Self::INPUT_POSE_1.into(), EdgeSpec::PoseFrame),
            (Self::INPUT_POSE_2.into(), EdgeSpec::PoseFrame),
            (Self::FACTOR.into(), EdgeSpec::F32),
        ])
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
