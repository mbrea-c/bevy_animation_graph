use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct LerpVec3Node {}

impl LerpVec3Node {
    pub const INPUT_A: &'static str = "a";
    pub const INPUT_B: &'static str = "b";
    pub const INPUT_FACTOR: &'static str = "factor";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::LerpVec3(self))
    }
}

impl NodeLike for LerpVec3Node {
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let a: Vec3 = ctx.data_back(Self::INPUT_A)?.val();
        let b: Vec3 = ctx.data_back(Self::INPUT_B)?.val();
        let factor: f32 = ctx.data_back(Self::INPUT_FACTOR)?.val();

        let output = Vec3::lerp(a, b, factor);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_A.into(), DataSpec::Vec3),
            (Self::INPUT_B.into(), DataSpec::Vec3),
            (Self::INPUT_FACTOR.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Vec3)].into()
    }

    fn display_name(&self) -> String {
        "Lerp Vec3".into()
    }
}
