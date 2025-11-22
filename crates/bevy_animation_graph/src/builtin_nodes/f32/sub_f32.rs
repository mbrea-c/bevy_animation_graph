use bevy::prelude::*;

use crate::core::{
    animation_graph::PinMap,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct SubF32;

impl SubF32 {
    pub const INPUT_1: &'static str = "in_a";
    pub const INPUT_2: &'static str = "in_b";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for SubF32 {
    fn display_name(&self) -> String {
        "- Subtract".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input_1 = ctx.data_back(Self::INPUT_1)?.as_f32()?;
        let input_2 = ctx.data_back(Self::INPUT_2)?.as_f32()?;

        ctx.set_data_fwd(Self::OUTPUT, input_1 - input_2);
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
