use bevy::utils::HashMap;

use crate::{
    animation::{
        AnimationNode, EdgeSpec, EdgeValue, EntityPath, NodeInput, NodeOutput, NodeWrapper, Pose,
        PoseFrame,
    },
    flipping::FlipXBySuffix,
};

pub struct FlipLRNode {}

impl FlipLRNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }
}

impl AnimationNode for FlipLRNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        *input_durations.get(Self::INPUT.into()).unwrap()
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        HashMap::from([(Self::INPUT.into(), time)])
    }

    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        let flipped_pose_frame = in_pose_frame.flipped_by_suffix(" R".into(), " L".into());

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(flipped_pose_frame),
        )])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::INPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
