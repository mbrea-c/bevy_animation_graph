use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
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
    fn display_name(&self) -> String {
        "× Multiply".into()
    }

    fn duration(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input_1 = ctx.data_back(Self::INPUT_1)?.unwrap_f32();
        let input_2 = ctx.data_back(Self::INPUT_2)?.unwrap_f32();

        ctx.set_data_fwd(Self::OUTPUT, input_1 * input_2);
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_1.into(), DataSpec::F32),
            (Self::INPUT_2.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
