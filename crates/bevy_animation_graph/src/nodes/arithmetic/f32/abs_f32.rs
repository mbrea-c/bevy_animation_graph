use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct AbsF32;

impl AbsF32 {
    pub const INPUT: &'static str = "in";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for AbsF32 {
    fn display_name(&self) -> String {
        "|_| Absolute val".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.data_back(Self::INPUT)?.as_f32()?;
        ctx.set_data_fwd(Self::OUTPUT, input.abs());
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::F32)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
