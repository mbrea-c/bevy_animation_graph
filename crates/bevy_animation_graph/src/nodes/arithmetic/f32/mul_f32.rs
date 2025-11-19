use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct MulF32;

impl MulF32 {
    pub const INPUT_1: &'static str = "in_a";
    pub const INPUT_2: &'static str = "in_b";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for MulF32 {
    fn display_name(&self) -> String {
        "× Multiply F32".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input_1 = ctx.data_back(Self::INPUT_1)?.as_f32()?;
        let input_2 = ctx.data_back(Self::INPUT_2)?.as_f32()?;

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
