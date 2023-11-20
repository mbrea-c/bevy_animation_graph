use crate::animation::{AnimationNode, EdgeSpec, EdgeValue, NodeInput, NodeOutput, NodeWrapper};
use bevy::utils::HashMap;

pub struct SpeedNode {
    speed: f32,
}

impl SpeedNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }
}

impl AnimationNode for SpeedNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        if self.speed == 0. {
            None
        } else {
            let duration = input_durations.get(Self::INPUT).unwrap();
            if let Some(duration) = duration {
                Some(duration / self.speed)
            } else {
                None
            }
        }
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        HashMap::from([(Self::INPUT.into(), time * self.speed)])
    }

    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let mut in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        if self.speed != 0. {
            in_pose_frame.prev_timestamp /= self.speed;
            in_pose_frame.next_timestamp /= self.speed;
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
