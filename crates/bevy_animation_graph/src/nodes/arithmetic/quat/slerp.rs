use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct SlerpQuatNode;

impl SlerpQuatNode {
    pub const INPUT_A: &'static str = "a";
    pub const INPUT_B: &'static str = "b";
    pub const INPUT_FACTOR: &'static str = "factor";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for SlerpQuatNode {
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let a: Quat = ctx.data_back(Self::INPUT_A)?.as_quat()?;
        let b: Quat = ctx.data_back(Self::INPUT_B)?.as_quat()?;
        let factor: f32 = ctx.data_back(Self::INPUT_FACTOR)?.as_f32()?;

        let output = Quat::slerp(a, b, factor);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_A.into(), DataSpec::Quat),
            (Self::INPUT_B.into(), DataSpec::Quat),
            (Self::INPUT_FACTOR.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Slerp Quat".into()
    }
}
