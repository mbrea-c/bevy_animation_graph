use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct ClampF32;

impl ClampF32 {
    pub const INPUT: &'static str = "in";
    pub const CLAMP_MIN: &'static str = "min";
    pub const CLAMP_MAX: &'static str = "max";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for ClampF32 {
    fn display_name(&self) -> String {
        "Clamp".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.data_back(Self::INPUT)?.unwrap_f32();
        let min = ctx.data_back(Self::CLAMP_MIN)?.unwrap_f32();
        let max = ctx.data_back(Self::CLAMP_MAX)?.unwrap_f32();
        ctx.set_data_fwd(Self::OUTPUT, input.clamp(min, max));
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT.into(), DataSpec::F32),
            (Self::CLAMP_MIN.into(), DataSpec::F32),
            (Self::CLAMP_MAX.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
