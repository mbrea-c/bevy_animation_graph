use crate::core::animation_graph::PinMap;
use crate::core::animation_node::NodeLike;
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct MulQuatNode {}

impl MulQuatNode {
    pub const INPUT_A: &'static str = "a";
    pub const INPUT_B: &'static str = "b";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self {}
    }
}

impl NodeLike for MulQuatNode {
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let a: Quat = ctx.data_back(Self::INPUT_A)?.val();
        let b: Quat = ctx.data_back(Self::INPUT_B)?.val();

        let output = a * b;

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_A.into(), DataSpec::Quat),
            (Self::INPUT_B.into(), DataSpec::Quat),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "× Multiply Quat".into()
    }
}
