use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
pub struct ClampF32 {}

impl ClampF32 {
    pub const INPUT: &'static str = "F32 In";
    pub const CLAMP_MIN: &'static str = "Min";
    pub const CLAMP_MAX: &'static str = "Max";
    pub const OUTPUT: &'static str = "F32 Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::ClampF32(self))
    }
}

impl NodeLike for ClampF32 {
    fn parameter_pass(
        &self,
        inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        let input = inputs.get(Self::INPUT).unwrap().clone().unwrap_f32();
        let min = inputs.get(Self::CLAMP_MIN).unwrap().clone().unwrap_f32();
        let max = inputs.get(Self::CLAMP_MAX).unwrap().clone().unwrap_f32();

        HashMap::from([(Self::OUTPUT.into(), ParamValue::F32(input.clamp(min, max)))])
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<Option<f32>> {
        None
    }

    fn time_pass(&self, _input: TimeState, _: PassContext) -> HashMap<PinId, TimeUpdate> {
        HashMap::new()
    }

    fn time_dependent_pass(
        &self,
        _inputs: HashMap<PinId, PoseFrame>,
        _: PassContext,
    ) -> Option<PoseFrame> {
        None
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::from([
            (Self::INPUT.into(), ParamSpec::F32.into()),
            (Self::CLAMP_MIN.into(), ParamSpec::F32.into()),
            (Self::CLAMP_MAX.into(), ParamSpec::F32.into()),
        ])
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::from([(Self::OUTPUT.into(), ParamSpec::F32)])
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::new()
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        false
    }

    fn display_name(&self) -> String {
        "Clamp".into()
    }
}
