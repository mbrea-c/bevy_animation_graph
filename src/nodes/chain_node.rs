use crate::animation::{
    AnimationNode, EdgeSpec, EdgeValue, NodeInput, NodeOutput, NodeWrapper, PoseFrame,
};
use bevy::utils::HashMap;

pub struct ChainNode {
    source_duration_1: Option<f32>,
    source_duration_2: Option<f32>,
}

impl ChainNode {
    pub const INPUT_1: &'static str = "Input Pose 1";
    pub const INPUT_2: &'static str = "Input Pose 2";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new() -> Self {
        Self {
            source_duration_1: None,
            source_duration_2: None,
        }
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }
}

impl AnimationNode for ChainNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        self.source_duration_1 = *input_durations.get(Self::INPUT_1.into()).unwrap();
        self.source_duration_2 = *input_durations.get(Self::INPUT_2.into()).unwrap();

        match (self.source_duration_1, self.source_duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1 + duration_2),
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        }
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        let t1 = time;
        let mut t2 = time;

        if let Some(duration_1) = self.source_duration_1 {
            if time > duration_1 {
                t2 = time - duration_1;
            }
        }

        HashMap::from([(Self::INPUT_1.into(), t1), (Self::INPUT_2.into(), t2)])
    }

    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let in_pose_1 = inputs
            .get(Self::INPUT_1.into())
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let in_pose_2 = inputs
            .get(Self::INPUT_2.into())
            .unwrap()
            .clone()
            .unwrap_pose_frame();

        let mut out_pose;

        match self.source_duration_1 {
            Some(duration_1) if time > duration_1 => {
                out_pose = in_pose_2;
                out_pose.prev_timestamp += duration_1;
                out_pose.next_timestamp += duration_1;
                if out_pose.next_is_wrapped {
                    out_pose.next = in_pose_1.next;
                    out_pose.next_timestamp = in_pose_1.next_timestamp + duration_1;
                }
            }
            Some(_) if in_pose_1.next_is_wrapped => {
                out_pose = PoseFrame {
                    prev: in_pose_1.prev,
                    prev_timestamp: in_pose_1.prev_timestamp,
                    next: in_pose_2.next,
                    next_timestamp: in_pose_2.next_timestamp,
                    next_is_wrapped: false,
                };
            }
            Some(_) => {
                out_pose = in_pose_1;
            }
            None => {
                out_pose = in_pose_1;
            }
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(out_pose))])
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
