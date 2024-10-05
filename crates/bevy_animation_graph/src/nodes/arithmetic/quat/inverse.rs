use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct InvertQuatNode;

impl InvertQuatNode {
    pub const INPUT: &'static str = "quat";
    pub const OUTPUT: &'static str = "inverse";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for InvertQuatNode {
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input: Quat = ctx.data_back(Self::INPUT)?.val();
        let output: Quat = input.inverse();

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::Quat)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Invert Quat".into()
    }
}
