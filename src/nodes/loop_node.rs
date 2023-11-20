use bevy::utils::HashMap;

use crate::animation::{AnimationNode, EdgeSpec, EdgeValue, NodeInput, NodeOutput, NodeWrapper};

pub struct LoopNode {
    source_duration: Option<f32>,
}

impl LoopNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new() -> Self {
        Self {
            source_duration: None,
        }
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }
}

impl AnimationNode for LoopNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        self.source_duration = *input_durations.get(Self::INPUT.into()).unwrap();

        None
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        let mut t = time;

        if let Some(duration) = self.source_duration {
            while t > duration {
                t -= duration;
            }
        }

        HashMap::from([(Self::INPUT.into(), t)])
    }

    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let mut in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();

        if let Some(duration) = self.source_duration {
            let t_extra = (time / duration).floor() * duration;
            in_pose_frame.prev_timestamp += t_extra;
            in_pose_frame.next_timestamp += t_extra;
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(in_pose_frame))])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::INPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
