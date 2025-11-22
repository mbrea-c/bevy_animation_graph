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
pub struct SelectF32;

impl SelectF32 {
    pub const INPUT_BOOL: &'static str = "bool";
    pub const INPUT_FALSE: &'static str = "if_false";
    pub const INPUT_TRUE: &'static str = "if_true";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for SelectF32 {
    fn display_name(&self) -> String {
        "Select F32".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let bool: bool = ctx.data_back(Self::INPUT_BOOL)?.as_bool()?;
        let if_false: f32 = ctx.data_back(Self::INPUT_FALSE)?.as_f32()?;
        let if_true: f32 = ctx.data_back(Self::INPUT_TRUE)?.as_f32()?;

        let output = if bool { if_true } else { if_false };

        ctx.set_data_fwd(Self::OUTPUT, output);
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_BOOL.into(), DataSpec::Bool),
            (Self::INPUT_FALSE.into(), DataSpec::F32),
            (Self::INPUT_TRUE.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
