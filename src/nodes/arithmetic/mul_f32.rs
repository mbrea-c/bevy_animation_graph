use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
pub struct MulF32 {}

impl MulF32 {
    pub const INPUT_1: &'static str = "F32 In 1";
    pub const INPUT_2: &'static str = "F32 In 2";
    pub const OUTPUT: &'static str = "F32 Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::MulF32(self))
    }
}

impl NodeLike for MulF32 {
    fn parameter_pass(
        &self,
        inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        let input_1 = inputs.get(Self::INPUT_1).unwrap().clone().unwrap_f32();
        let input_2 = inputs.get(Self::INPUT_2).unwrap().clone().unwrap_f32();

        HashMap::from([(Self::OUTPUT.into(), ParamValue::F32(input_1 * input_2))])
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<DurationData> {
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
            (Self::INPUT_1.into(), ParamSpec::F32.into()),
            (Self::INPUT_2.into(), ParamSpec::F32.into()),
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
        "× Multiply".into()
    }
}
