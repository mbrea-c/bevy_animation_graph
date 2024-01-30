use crate::core::animation_graph::{PinId, PinMap};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::prelude::{OptParamSpec, ParamSpec, ParamValue, PassContext, SpecContext};
use crate::utils::unwrap::Unwrap;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct RotationArcNode {}

impl RotationArcNode {
    pub const INPUT_1: &'static str = "Vec3 In 1";
    pub const INPUT_2: &'static str = "Vec3 In 2";
    pub const OUTPUT: &'static str = "Quat Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::RotationArc(self))
    }
}

impl NodeLike for RotationArcNode {
    fn parameter_pass(&self, mut ctx: PassContext) -> HashMap<PinId, ParamValue> {
        let input_1: Vec3 = ctx.parameter_back(Self::INPUT_1).unwrap();
        let input_2: Vec3 = ctx.parameter_back(Self::INPUT_2).unwrap();

        HashMap::from([(
            Self::OUTPUT.into(),
            ParamValue::Quat(Quat::from_rotation_arc(input_1, input_2)),
        )])
    }

    fn parameter_input_spec(&self, _: SpecContext) -> PinMap<OptParamSpec> {
        [
            (Self::INPUT_1.into(), ParamSpec::Vec3.into()),
            (Self::INPUT_2.into(), ParamSpec::Vec3.into()),
        ]
        .into()
    }

    fn parameter_output_spec(&self, _: SpecContext) -> PinMap<ParamSpec> {
        [(Self::OUTPUT.into(), ParamSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Rotation Arc".into()
    }
}
