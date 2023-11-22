use crate::{
    animation::{AnimationNode, EdgeSpec, EdgeValue, NodeInput, NodeLike, NodeOutput},
    chaining::Chainable,
};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
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

    pub fn wrapped(self) -> AnimationNode {
        AnimationNode::Chain(self)
    }
}

impl NodeLike for ChainNode {
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
            t2 = time - duration_1;
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

        let out_pose;

        if let Some(duration_1) = self.source_duration_1 {
            out_pose = in_pose_1.chain(
                &in_pose_2,
                duration_1,
                self.source_duration_2.unwrap_or(f32::MAX),
                time,
            );
        } else {
            out_pose = in_pose_1;
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
