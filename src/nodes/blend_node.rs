use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::interpolation::linear::InterpolateLinear;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
pub struct BlendNode;

impl BlendNode {
    pub const INPUT_1: &'static str = "Pose In 1";
    pub const INPUT_2: &'static str = "Pose In 2";
    pub const FACTOR: &'static str = "Factor";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Blend(self))
    }
}

impl NodeLike for BlendNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<DurationData> {
        let duration_1 = *inputs.get(Self::INPUT_1).unwrap();
        let duration_2 = *inputs.get(Self::INPUT_2).unwrap();

        let out_duration = match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        };

        Some(out_duration)
    }

    fn time_pass(&self, input: TimeState, _: PassContext) -> HashMap<PinId, TimeUpdate> {
        HashMap::from([
            (Self::INPUT_1.into(), input.update),
            (Self::INPUT_2.into(), input.update),
        ])
    }

    fn time_dependent_pass(
        &self,
        mut inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame> {
        let in_frame_1 = inputs.remove(Self::INPUT_1).unwrap();
        let in_frame_2 = inputs.remove(Self::INPUT_2).unwrap();
        let alpha = ctx.parameter_back(Self::FACTOR).unwrap_f32();

        Some(in_frame_1.interpolate_linear(&in_frame_2, alpha))
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::from([(Self::FACTOR.into(), ParamSpec::F32.into())])
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::new()
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT_1.into(), Self::INPUT_2.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        true
    }

    fn display_name(&self) -> String {
        "󰳫 Blend".into()
    }
}
