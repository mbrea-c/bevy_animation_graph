use crate::core::animation_graph::{EdgeSpec, EdgeValue, NodeInput, NodeOutput};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::caches::{DurationCache, EdgePathCache, ParameterCache, TimeCache};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
pub struct LoopNode {}

impl LoopNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Loop Pose";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self) -> AnimationNode {
        AnimationNode::new_from_nodetype(AnimationNodeType::Loop(self))
    }
}

impl NodeLike for LoopNode {
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
        None
    }

    fn time_pass(
        &self,
        input: f32,
        parameters: &ParameterCache,
        durations: &DurationCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeInput, f32> {
        let mut t = input;

        let duration = *durations.inputs.get(Self::INPUT).unwrap();
        if let Some(duration) = duration {
            t = t.rem_euclid(duration);
        }

        HashMap::from([(Self::INPUT.into(), t)])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        parameters: &ParameterCache,
        durations: &DurationCache,
        time: &TimeCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let mut in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        let time = time.input;
        let duration = durations.inputs.get(Self::INPUT).unwrap();

        if let Some(duration) = duration {
            let t_extra = time.div_euclid(*duration) * duration;
            in_pose_frame.map_ts(|t| t + t_extra);
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(in_pose_frame))])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::from([(Self::INPUT.into(), ())])
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::INPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
