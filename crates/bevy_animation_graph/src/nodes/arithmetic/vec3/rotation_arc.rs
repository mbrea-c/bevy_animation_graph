use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

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
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input_1: Vec3 = ctx.data_back(Self::INPUT_1)?.val();
        let input_2: Vec3 = ctx.data_back(Self::INPUT_2)?.val();

        ctx.set_data_fwd(Self::OUTPUT, Quat::from_rotation_arc(input_1, input_2));

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_1.into(), DataSpec::Vec3),
            (Self::INPUT_2.into(), DataSpec::Vec3),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Rotation Arc".into()
    }
}
