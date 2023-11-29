use crate::core::animation_graph::{EdgeSpec, EdgeValue, NodeInput, NodeOutput};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::caches::{DurationCache, EdgePathCache, ParameterCache, TimeCache};
use bevy::{reflect::Reflect, utils::HashMap};

#[derive(Reflect, Clone, Debug)]
pub struct SpeedNode;

impl SpeedNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Speed Pose";
    pub const SPEED: &'static str = "Speed";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self) -> AnimationNode {
        AnimationNode::new_from_nodetype(AnimationNodeType::Speed(self))
    }
}

impl NodeLike for SpeedNode {
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
        parameters: &ParameterCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> Option<f32> {
        let speed = parameters
            .inputs
            .get(Self::SPEED)
            .unwrap()
            .clone()
            .unwrap_f32();

        if speed == 0. {
            None
        } else {
            let duration = inputs.get(Self::INPUT).unwrap();
            if let Some(duration) = duration {
                Some(duration / speed)
            } else {
                None
            }
        }
    }

    fn time_pass(
        &self,
        input: f32,
        parameters: &ParameterCache,
        _durations: &DurationCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeInput, f32> {
        let speed = parameters
            .inputs
            .get(Self::SPEED)
            .unwrap()
            .clone()
            .unwrap_f32();

        HashMap::from([(Self::INPUT.into(), input * speed)])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        parameters: &ParameterCache,
        _durations: &DurationCache,
        _time: &TimeCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let mut in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        let speed = parameters
            .inputs
            .get(Self::SPEED)
            .unwrap()
            .clone()
            .unwrap_f32();

        if speed != 0. {
            in_pose_frame.map_ts(|t| t / speed);
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(in_pose_frame))])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::SPEED.into(), EdgeSpec::F32)])
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
