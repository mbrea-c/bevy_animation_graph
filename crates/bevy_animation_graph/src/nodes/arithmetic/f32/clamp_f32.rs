use crate::core::animation_graph::PinId;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::prelude::{OptParamSpec, ParamSpec, ParamValue, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
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
    fn parameter_pass(&self, mut ctx: PassContext) -> HashMap<PinId, ParamValue> {
        let input = ctx.parameter_back(Self::INPUT).unwrap_f32();
        let min = ctx.parameter_back(Self::CLAMP_MIN).unwrap_f32();
        let max = ctx.parameter_back(Self::CLAMP_MAX).unwrap_f32();

        HashMap::from([(Self::OUTPUT.into(), ParamValue::F32(input.clamp(min, max)))])
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
    fn display_name(&self) -> String {
        "Clamp".into()
    }
}
