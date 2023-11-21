use bevy::utils::HashMap;

use crate::{
    animation::{
        AnimationNode, EdgeSpec, EdgeValue, EntityPath, NodeInput, NodeOutput, NodeWrapper, Pose,
        PoseFrame,
    },
    interpolation::linear::InterpolateLinear,
};

pub struct BlendNode {
    alpha: f32,
}

impl BlendNode {
    pub const INPUT_1: &'static str = "Input Pose 1";
    pub const INPUT_2: &'static str = "Input Pose 2";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new(alpha: f32) -> Self {
        Self { alpha }
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }
}

impl AnimationNode for BlendNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        let duration_1 = *input_durations.get(Self::INPUT_1.into()).unwrap();
        let duration_2 = *input_durations.get(Self::INPUT_2.into()).unwrap();

        match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        }
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        HashMap::from([(Self::INPUT_1.into(), time), (Self::INPUT_2.into(), time)])
    }

    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
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

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(in_frame_1.interpolate_linear(&in_frame_2, self.alpha)),
        )])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([
            (Self::INPUT_1.into(), EdgeSpec::PoseFrame),
            (Self::INPUT_2.into(), EdgeSpec::PoseFrame),
        ])
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
