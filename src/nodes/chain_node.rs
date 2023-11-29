use crate::chaining::Chainable;
use crate::core::animation_graph::{EdgeSpec, EdgeValue, NodeInput, NodeOutput};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::caches::{DurationCache, EdgePathCache, ParameterCache, TimeCache};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct ChainNode {}

impl ChainNode {
    pub const INPUT_1: &'static str = "Input Pose 1";
    pub const INPUT_2: &'static str = "Input Pose 2";
    pub const OUTPUT: &'static str = "Chain Pose";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: String) -> AnimationNode {
        AnimationNode::new_from_nodetype(name, AnimationNodeType::Chain(self))
    }
}

impl NodeLike for ChainNode {
    fn parameter_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        parameters: &ParameterCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> Option<f32> {
        let source_duration_1 = *inputs.get(Self::INPUT_1.into()).unwrap();
        let source_duration_2 = *inputs.get(Self::INPUT_2.into()).unwrap();

        match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1 + duration_2),
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        }
    }

    fn time_pass(
        &self,
        input: f32,
        parameters: &ParameterCache,
        durations: &DurationCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeInput, f32> {
        let t1 = input;
        let mut t2 = input;

        let duration_1 = durations.inputs.get(Self::INPUT_1).unwrap();

        if let Some(duration_1) = duration_1 {
            t2 = input - duration_1;
        }

        HashMap::from([(Self::INPUT_1.into(), t1), (Self::INPUT_2.into(), t2)])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        parameters: &ParameterCache,
        durations: &DurationCache,
        time: &TimeCache,
        _last_cache: Option<&EdgePathCache>,
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

        let time = time.input;

        let duration_1 = *durations.inputs.get(Self::INPUT_1).unwrap();
        let duration_2 = *durations.inputs.get(Self::INPUT_2).unwrap();

        let out_pose;

        if let Some(duration_1) = duration_1 {
            out_pose =
                in_pose_1.chain(&in_pose_2, duration_1, duration_2.unwrap_or(f32::MAX), time);
        } else {
            out_pose = in_pose_1;
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(out_pose))])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
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
